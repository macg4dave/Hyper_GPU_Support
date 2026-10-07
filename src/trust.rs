//! Native payload trust: catalog membership or embedded signatures, never catalog presence alone.
use crate::payload::Manifest;
use std::{fs::File, os::windows::io::AsRawHandle, path::Path};
use windows::{
    Win32::{
        Foundation::{HANDLE, HWND},
        Security::{
            Cryptography::Catalog::{
                CryptCATAdminAcquireContext2, CryptCATAdminCalcHashFromFileHandle2,
                CryptCATAdminReleaseContext,
            },
            WinTrust::*,
        },
    },
    core::{PCWSTR, w},
};
struct CatalogContext(isize);
#[allow(unsafe_code)]
impl Drop for CatalogContext {
    fn drop(&mut self) {
        // SAFETY: solely owned successful catalog context.
        unsafe {
            let _ = CryptCATAdminReleaseContext(self.0, 0);
        }
    }
}
fn wide(path: &Path) -> Vec<u16> {
    path.to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect()
}
#[allow(unsafe_code)]
fn check(data: &mut WINTRUST_DATA) -> bool {
    let mut action = WINTRUST_ACTION_GENERIC_VERIFY_V2;
    data.dwStateAction = WTD_STATEACTION_VERIFY;
    // SAFETY: caller selected and initialized the matching live FILE/CATALOG union.
    let result = unsafe {
        WinVerifyTrust(
            HWND::default(),
            &mut action,
            (data as *mut WINTRUST_DATA).cast(),
        )
    };
    data.dwStateAction = WTD_STATEACTION_CLOSE;
    // SAFETY: release state from exactly the preceding trust invocation.
    unsafe {
        WinVerifyTrust(
            HWND::default(),
            &mut action,
            (data as *mut WINTRUST_DATA).cast(),
        );
    }
    result == 0
}
fn embedded(path: &Path) -> bool {
    let path = wide(path);
    let mut file = WINTRUST_FILE_INFO {
        cbStruct: std::mem::size_of::<WINTRUST_FILE_INFO>() as u32,
        pcwszFilePath: PCWSTR(path.as_ptr()),
        ..Default::default()
    };
    let mut data = WINTRUST_DATA {
        cbStruct: std::mem::size_of::<WINTRUST_DATA>() as u32,
        dwUIChoice: WTD_UI_NONE,
        dwUnionChoice: WTD_CHOICE_FILE,
        ..Default::default()
    };
    data.Anonymous.pFile = &mut file;
    check(&mut data)
}

/// Require every discovered source to authenticate to a trusted signature/catalog.
#[allow(unsafe_code)]
pub(crate) fn validate(manifest: &Manifest) -> Result<(), String> {
    if manifest.catalogs.is_empty() {
        return Err("driver catalogs are missing".into());
    }
    for catalog in &manifest.catalogs {
        if !embedded(catalog) {
            return Err("driver catalog signature is invalid".into());
        }
    }
    let mut context = CatalogContext(0);
    // SAFETY: writable context handle; standard SHA256 catalog hashing provider.
    unsafe { CryptCATAdminAcquireContext2(&mut context.0, None, w!("SHA256"), None, None) }
        .map_err(|e| e.to_string())?;
    let mut authenticated = std::collections::BTreeSet::new();
    let mut caches = Vec::new();
    for source in &manifest.files {
        crate::payload::no_reparse(&source.source)?;
        crate::security::verify_system(&source.source)?;
        if source
            .source
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pnf"))
        {
            caches.push(&source.source);
            continue;
        }
        if embedded(&source.source) {
            authenticated.insert(source.source.to_string_lossy().to_ascii_lowercase());
            continue;
        }
        let file = File::open(&source.source).map_err(|e| e.to_string())?;
        let handle = HANDLE(file.as_raw_handle());
        let mut size = 0;
        // SAFETY: live owned file and catalog context, writable hash length.
        unsafe { CryptCATAdminCalcHashFromFileHandle2(context.0, handle, &mut size, None, None) }
            .map_err(|e| e.to_string())?;
        if size == 0 || size > 128 {
            return Err("invalid catalog hash length".into());
        }
        let mut hash = vec![0; size as usize];
        // SAFETY: sized writable hash buffer and live source handle.
        unsafe {
            CryptCATAdminCalcHashFromFileHandle2(
                context.0,
                handle,
                &mut size,
                Some(hash.as_mut_ptr()),
                None,
            )
        }
        .map_err(|e| e.to_string())?;
        let tag: Vec<_> = hash
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<String>()
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let member = wide(&source.source);
        let mut verified = false;
        for catalog in &manifest.catalogs {
            let path = wide(catalog);
            let mut catalog = WINTRUST_CATALOG_INFO {
                cbStruct: std::mem::size_of::<WINTRUST_CATALOG_INFO>() as u32,
                pcwszCatalogFilePath: PCWSTR(path.as_ptr()),
                pcwszMemberTag: PCWSTR(tag.as_ptr()),
                pcwszMemberFilePath: PCWSTR(member.as_ptr()),
                hMemberFile: handle,
                pbCalculatedFileHash: hash.as_mut_ptr(),
                cbCalculatedFileHash: size,
                hCatAdmin: context.0,
                ..Default::default()
            };
            let mut data = WINTRUST_DATA {
                cbStruct: std::mem::size_of::<WINTRUST_DATA>() as u32,
                dwUIChoice: WTD_UI_NONE,
                dwUnionChoice: WTD_CHOICE_CATALOG,
                ..Default::default()
            };
            data.Anonymous.pCatalog = &mut catalog;
            if check(&mut data) {
                verified = true;
                break;
            }
        }
        if !verified {
            return Err(format!(
                "driver source is neither signed nor authenticated by its catalogs: {}",
                source.destination
            ));
        }
        authenticated.insert(source.source.to_string_lossy().to_ascii_lowercase());
    }
    // Windows generates precompiled INF caches after installation; those bytes
    // are not catalog members. Accept data-only caches only alongside an
    // authenticated INF in the same protected installed package. Never apply
    // this exception to executable/runtime payloads or unrelated metadata.
    for cache in caches {
        let windows = crate::windows_paths::windows_directory().map_err(|e| e.to_string())?;
        if !authenticated_cache(cache, &windows, &authenticated) {
            return Err(
                "precompiled INF cache lacks an authenticated installed-package INF".into(),
            );
        }
    }
    Ok(())
}
fn authenticated_cache(
    cache: &Path,
    windows: &Path,
    authenticated: &std::collections::BTreeSet<String>,
) -> bool {
    let inf = cache.with_extension("inf");
    let prefix = format!(
        "{}\\System32\\DriverStore\\FileRepository\\",
        windows.to_string_lossy()
    );
    let text = inf.to_string_lossy();
    cache
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pnf"))
        && text
            .get(..prefix.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(&prefix))
        && authenticated.contains(&text.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    #[test]
    fn generated_cache_exception_requires_same_authenticated_installed_inf() {
        use std::{collections::BTreeSet, path::Path};
        let root = Path::new(r"D:\OperatingSystem");
        let inf = r"d:\operatingsystem\system32\driverstore\filerepository\new.inf_dynamic\new.inf";
        let signed = BTreeSet::from([inf.to_owned()]);
        let cache = Path::new(
            r"D:\OperatingSystem\System32\DriverStore\FileRepository\new.inf_dynamic\new.pnf",
        );
        assert!(!super::authenticated_cache(cache, root, &BTreeSet::new()));
        assert!(super::authenticated_cache(
            Path::new(
                &inf.replace(".inf", ".pnf")
                    .replace("new.pnf_dynamic", "new.inf_dynamic")
            ),
            root,
            &signed
        ));
        assert!(!super::authenticated_cache(
            Path::new(r"D:\OperatingSystem\System32\DriverStore\FileRepository\other\new.pnf"),
            root,
            &signed
        ));
        assert!(!super::authenticated_cache(
            Path::new(r"D:\Other\new.pnf"),
            root,
            &signed
        ));
        assert!(!super::authenticated_cache(Path::new(inf), root, &signed));
    }
    #[test]
    fn unsigned_bytes_are_not_authenticated_as_a_driver() {
        let path =
            std::env::temp_dir().join(format!("hyper-gpu-unsigned-{}.bin", std::process::id()));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        use std::io::Write;
        file.write_all(b"unsigned driver fixture").unwrap();
        drop(file);
        let verified = super::embedded(&path);
        std::fs::remove_file(&path).unwrap();
        assert!(!verified);
    }
}
