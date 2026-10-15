//! Guest-reported OS metadata: optional display and installation-only fallback.
use crate::guest_compatibility::{WindowsIdentity, unknown};
use windows::{
    Data::Xml::Dom::{XmlDocument, XmlLoadSettings},
    core::HSTRING,
};

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

/// Admission uses complete, internally consistent intrinsic version/client data,
/// never a VM name, frontend cache or OSName alone. Reports can be historical;
/// the current running OS is still rechecked before guest installation writes.
pub(super) fn installation_identity(items: &[String]) -> Result<WindowsIdentity, String> {
    if items.len() > 256 {
        return Err(unknown());
    }
    let settings = XmlLoadSettings::new().map_err(|_| unknown())?;
    settings.SetProhibitDtd(true).map_err(|_| unknown())?;
    settings.SetResolveExternals(false).map_err(|_| unknown())?;
    settings.SetMaxElementDepth(16).map_err(|_| unknown())?;
    let mut values = std::collections::BTreeMap::new();
    for xml in items {
        if xml.len() > 65536 {
            return Err(unknown());
        }
        let document = XmlDocument::new().map_err(|_| unknown())?;
        document
            .LoadXmlWithSettings(&HSTRING::from(xml), &settings)
            .map_err(|_| unknown())?;
        let key = property(&document, "Name")
            .ok_or_else(unknown)?
            .to_ascii_lowercase();
        if !matches!(
            key.as_str(),
            "osname"
                | "osversion"
                | "osbuildnumber"
                | "osmajorversion"
                | "osminorversion"
                | "osplatformid"
                | "producttype"
        ) {
            continue;
        }
        let data = property(&document, "Data").ok_or_else(unknown)?;
        if data.is_empty() || data.len() > 1024 || values.insert(key, data).is_some() {
            return Err(unknown());
        }
    }
    let get = |key: &str| values.get(key).map(String::as_str).ok_or_else(unknown);
    let number = |text: &str| {
        if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
            return Err(unknown());
        }
        text.parse::<u32>().map_err(|_| unknown())
    };
    let version: Vec<_> = get("osversion")?
        .split('.')
        .map(number)
        .collect::<Result<_, _>>()?;
    let [major, minor, build] = version.as_slice() else {
        return Err(unknown());
    };
    if number(get("osbuildnumber")?)? != *build {
        return Err(unknown());
    }
    for (key, expected) in [
        ("osmajorversion", *major),
        ("osminorversion", *minor),
        ("osplatformid", 2),
    ] {
        if let Some(value) = values.get(key)
            && number(value)? != expected
        {
            return Err(unknown());
        }
    }
    // OSName is registry ProductName on Windows and can still say Windows 10
    // on Windows 11. The numeric release policy makes the actual distinction.
    let name = get("osname")?.to_ascii_lowercase();
    let product = name
        .strip_prefix("microsoft windows ")
        .ok_or_else(unknown)?;
    let windows_client_name = ["10", "11"]
        .iter()
        .any(|version| product == *version || product.starts_with(&format!("{version} ")));
    Ok(WindowsIdentity {
        major: *major,
        minor: *minor,
        build: *build,
        workstation: number(get("producttype")?)? == 1
            && windows_client_name
            && !name.contains("server"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: &str, value: &str) -> String {
        format!(
            r#"<INSTANCE CLASSNAME="Msvm_KvpExchangeDataItem"><PROPERTY NAME="Name" TYPE="string"><VALUE>{key}</VALUE></PROPERTY><PROPERTY NAME="Data" TYPE="string"><VALUE>{value}</VALUE></PROPERTY></INSTANCE>"#
        )
    }

    fn reported_windows(build: &str, product_type: &str) -> Vec<String> {
        vec![
            entry("OSName", "Microsoft Windows 11 Pro"),
            entry("OSVersion", &format!("10.0.{build}")),
            entry("OSBuildNumber", build),
            entry("ProductType", product_type),
        ]
    }

    #[test]
    fn installation_fallback_requires_consistent_windows_11_client_facts() {
        let _apartment = crate::windows_com::Apartment::initialize().unwrap();
        for name in ["Microsoft Windows 11 Pro", "Microsoft Windows 10 Pro"] {
            let mut items = reported_windows("26100", "1");
            items[0] = entry("OSName", name);
            let identity = installation_identity(&items).unwrap();
            assert!(crate::guest_compatibility::require_supported(&identity).is_ok());
        }
        for (build, product_type) in [
            ("19045", "1"),
            ("26100", "3"),
            ("26100", "2"),
            ("99999", "1"),
        ] {
            let identity = installation_identity(&reported_windows(build, product_type)).unwrap();
            assert!(crate::guest_compatibility::require_supported(&identity).is_err());
        }
    }

    #[test]
    fn missing_conflicting_or_malformed_reports_cannot_approve_installation() {
        let _apartment = crate::windows_com::Apartment::initialize().unwrap();
        assert!(installation_identity(&[]).is_err());
        assert!(installation_identity(&[entry("OSName", "Microsoft Windows 11 Pro")]).is_err());
        for scenario in 0..10 {
            let mut items = reported_windows("26100", "1");
            match scenario {
                0 => {
                    items.pop();
                }
                1 => items.push(entry("producttype", "1")),
                2 => items[2] = entry("OSBuildNumber", "19045"),
                3 => items[1] = entry("OSVersion", "10.0.26100.1"),
                4 => items.push(entry("OSMajorVersion", "6")),
                5 => items.push(entry("OSPlatformId", "0")),
                6 => items.push("invalid XML".into()),
                7 => items[3] = entry("ProductType", "workstation"),
                8 => items.push(entry("OSName", "Microsoft Windows 11 Pro")),
                _ => items[1] = entry("OSVersion", "10.0.+26100"),
            }
            assert!(installation_identity(&items).is_err(), "report {scenario}");
        }
        let mut conflicting = reported_windows("26100", "1");
        conflicting[0] = entry("OSName", "Ubuntu 24.04");
        assert!(
            installation_identity(&conflicting)
                .and_then(|identity| crate::guest_compatibility::require_supported(&identity))
                .is_err()
        );
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
        let serialised = entry("OSName", "Microsoft Windows 11 Pro")
            .replace("TYPE=\"string\"", "TYPE=\"STRING\"")
            .replace("<PROPERTY", "\n<PROPERTY");
        assert_eq!(
            name(&[serialised]).as_deref(),
            Some("Microsoft Windows 11 Pro")
        );
        let duplicate = entry("OSName", "Linux").replace(
            "</INSTANCE>",
            "<PROPERTY NAME=\"Data\" TYPE=\"string\"><VALUE>Windows</VALUE></PROPERTY></INSTANCE>",
        );
        assert!(name(&[duplicate]).is_none());
    }
}
