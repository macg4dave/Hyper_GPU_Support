//! Optional, untrusted guest OS metadata; never used for operation admission.
use windows::{
    Data::Xml::Dom::{XmlDocument, XmlLoadSettings},
    core::HSTRING,
};

fn property(document: &XmlDocument, name: &str) -> Option<String> {
    // CIM class/property identifiers are case-insensitive, including XML
    // serialised by integration services from different guest implementations.
    let nodes = document.SelectNodes(&HSTRING::from(format!(
        "/INSTANCE[translate(@CLASSNAME,'ABCDEFGHIJKLMNOPQRSTUVWXYZ','abcdefghijklmnopqrstuvwxyz')='msvm_kvpexchangedataitem']/PROPERTY[translate(@NAME,'ABCDEFGHIJKLMNOPQRSTUVWXYZ','abcdefghijklmnopqrstuvwxyz')='{name}' and @TYPE='string']/VALUE"
        , name = name.to_ascii_lowercase()
    ))).ok()?;
    if nodes.Length().ok()? != 1 {
        return None;
    }
    Some(nodes.Item(0).ok()?.InnerText().ok()?.to_string())
}

/// Intrinsic KVP entries are CIM XML. Missing or ambiguous OSName stays unknown.
pub(super) fn name(items: &[String]) -> Option<String> {
    let settings = XmlLoadSettings::new().ok()?;
    settings.SetProhibitDtd(true).ok()?;
    settings.SetResolveExternals(false).ok()?;
    settings.SetMaxElementDepth(16).ok()?;
    let mut found = None;
    for xml in items {
        if xml.len() > 65536 {
            return None;
        }
        let document = XmlDocument::new().ok()?;
        if document
            .LoadXmlWithSettings(&HSTRING::from(xml), &settings)
            .is_err()
        {
            // One unrelated broken KVP entry must not hide a valid OSName.
            continue;
        }
        if property(&document, "Name").is_some_and(|key| key.eq_ignore_ascii_case("OSName")) {
            let value = property(&document, "Data")?;
            if value.trim().is_empty() || value.len() > 1024 || found.is_some() {
                return None;
            }
            found = Some(value);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: &str, value: &str) -> String {
        format!(
            r#"<INSTANCE CLASSNAME="Msvm_KvpExchangeDataItem"><PROPERTY NAME="Name" TYPE="string"><VALUE>{key}</VALUE></PROPERTY><PROPERTY NAME="Data" TYPE="string"><VALUE>{value}</VALUE></PROPERTY></INSTANCE>"#
        )
    }

    #[test]
    fn intrinsic_metadata_parses_without_provider_or_guest_access() {
        let _apartment = crate::windows_com::Apartment::initialize().unwrap();
        for os in ["Microsoft Windows 11 Pro", "Ubuntu 24.04", "FreeBSD"] {
            assert_eq!(name(&[entry("OSName", os)]).as_deref(), Some(os));
        }
        assert_eq!(
            name(&[entry("OSName", "Debian GNU/Linux &amp; tools")]).as_deref(),
            Some("Debian GNU/Linux & tools")
        );
        assert!(name(&[]).is_none());
        assert!(name(&[entry("ComputerName", "Windows VM")]).is_none());
        assert!(name(&[entry("OSName", "")]).is_none());
        assert!(name(&[entry("OSName", "Windows"), entry("OSName", "Linux")]).is_none());
        assert!(name(&["invalid XML".into()]).is_none());
        assert!(name(&["x".repeat(65537)]).is_none());
        assert!(
            name(&[format!(
                "<!DOCTYPE INSTANCE [<!ENTITY os SYSTEM 'file:///missing'>]>{}",
                entry("OSName", "&os;")
            )])
            .is_none()
        );
    }

    #[test]
    fn independent_kvp_entries_and_cim_identifier_case_do_not_hide_the_os() {
        let _apartment = crate::windows_com::Apartment::initialize().unwrap();
        let os = entry("osname", "Ubuntu 24.04")
            .replace("Msvm_KvpExchangeDataItem", "Msvm_KVPExchangeDataItem")
            .replace("NAME=\"Data\"", "NAME=\"data\"");
        for items in [
            vec!["invalid unrelated entry".into(), os.clone()],
            vec![os.clone(), "invalid unrelated entry".into()],
            vec![entry("ComputerName", "Windows VM"), os.clone()],
        ] {
            assert_eq!(name(&items).as_deref(), Some("Ubuntu 24.04"));
        }
        assert!(name(&[os, entry("OSName", "Windows")]).is_none());
    }
}
