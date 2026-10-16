//! Optional, untrusted OS metadata for display only.
use windows::{
    Data::Xml::Dom::{XmlDocument, XmlLoadSettings},
    Win32::System::WinRT::{
        IActivationFactory, RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize,
    },
    core::{HSTRING, Interface, RuntimeName},
};

struct XmlRuntime;
#[allow(unsafe_code)]
impl XmlRuntime {
    fn initialize() -> windows::core::Result<Self> {
        // SAFETY: metadata parsing uses the calling WMI MTA thread. WinRT has
        // its own per-thread initialisation contract in addition to COM; retain
        // it until all XML objects and factories in this call have been released.
        unsafe { RoInitialize(RO_INIT_MULTITHREADED) }?;
        Ok(Self)
    }
}
#[allow(unsafe_code)]
impl Drop for XmlRuntime {
    fn drop(&mut self) {
        // SAFETY: balances this thread's successful RoInitialize, including S_FALSE.
        unsafe { RoUninitialize() };
    }
}

#[allow(unsafe_code)]
fn activate<T: Interface + RuntimeName>() -> windows::core::Result<T> {
    // Generated `new()` caches agile factories across calls. Our WMI apartments
    // are short-lived; after the last MTA exits that cache can reference unloaded
    // XML implementation code. Own a fresh factory only for this activation.
    let factory = windows::core::factory::<T, IActivationFactory>()?;
    // SAFETY: the factory was obtained for T's exact registered runtime class;
    // both the factory and instance are released within the caller's apartment.
    unsafe { factory.ActivateInstance() }?.cast()
}

fn property(document: &XmlDocument, name: &str) -> Option<String> {
    let instance = document.DocumentElement().ok()?;
    if !instance
        .GetAttribute(&HSTRING::from("CLASSNAME"))
        .ok()?
        .to_string()
        .eq_ignore_ascii_case("Msvm_KvpExchangeDataItem")
    {
        return None;
    }
    let properties = instance.ChildNodes().ok()?;
    let mut found = None;
    for index in 0..properties.Length().ok()? {
        let node = properties.Item(index).ok()?;
        if !node
            .NodeName()
            .ok()?
            .to_string()
            .eq_ignore_ascii_case("PROPERTY")
        {
            continue;
        }
        let attributes = node.Attributes().ok()?;
        let key = attributes
            .GetNamedItem(&HSTRING::from("NAME"))
            .ok()?
            .InnerText()
            .ok()?
            .to_string();
        if !key.eq_ignore_ascii_case(name) {
            continue;
        }
        let kind = attributes
            .GetNamedItem(&HSTRING::from("TYPE"))
            .ok()?
            .InnerText()
            .ok()?
            .to_string();
        if !kind.eq_ignore_ascii_case("string") {
            return None;
        }
        let children = node.ChildNodes().ok()?;
        for child in 0..children.Length().ok()? {
            let value = children.Item(child).ok()?;
            if value
                .NodeName()
                .ok()?
                .to_string()
                .eq_ignore_ascii_case("VALUE")
            {
                if found.is_some() {
                    return None;
                }
                found = Some(value.InnerText().ok()?.to_string());
            }
        }
    }
    found
}

/// Intrinsic KVP entries are CIM XML. Missing or ambiguous OSName stays unknown.
pub(super) fn name(items: &[String]) -> Option<String> {
    let _runtime = XmlRuntime::initialize().ok()?;
    let settings = activate::<XmlLoadSettings>().ok()?;
    settings.SetProhibitDtd(true).ok()?;
    settings.SetResolveExternals(false).ok()?;
    settings.SetMaxElementDepth(16).ok()?;
    let mut found = None;
    for xml in items {
        if xml.len() > 65536 {
            return None;
        }
        let document = activate::<XmlDocument>().ok()?;
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
