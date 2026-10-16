//! Installation eligibility, separate from optional discovery/display metadata.
use crate::reporting::{Component, ComponentFailure, FailureCause};

// Explicit releases, never an open-ended build >= comparison that could admit a
// future Windows product. Server shares build numbers and must be excluded too.
// Microsoft release-health Windows 11 catalogue, reviewed 15 October 2026.
pub(crate) const WINDOWS_11_BUILDS: &[u32] = &[22000, 22621, 22631, 26100, 26200, 26300, 28000];

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WindowsIdentity {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
    pub workstation: bool,
}

impl WindowsIdentity {
    /// WMI version and build are independent fields and must describe one OS.
    /// Product names and cached presentation metadata never enter this boundary.
    pub(crate) fn from_facts(
        version: &str,
        build: &str,
        product_type: u64,
        installation_type: &str,
    ) -> Result<Self, String> {
        let number = |s: &str| {
            if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
                return Err(unknown());
            }
            s.parse::<u32>().map_err(|_| unknown())
        };
        let parts = version
            .split('.')
            .map(number)
            .collect::<Result<Vec<_>, _>>()?;
        let [major, minor, version_build] = parts.as_slice() else {
            return Err(unknown());
        };
        let build = number(build)?;
        if build != *version_build
            || !matches!(product_type, 1..=3)
            || installation_type.trim().is_empty()
        {
            return Err(unknown());
        }
        Ok(Self {
            major: *major,
            minor: *minor,
            build,
            workstation: product_type == 1 && installation_type == "Client",
        })
    }
}

pub(crate) fn unknown() -> String {
    ComponentFailure {
        stage: None,
        component: Component::GuestCompatibility,
        cause: FailureCause::Unavailable,
        code: None,
    }
    .into_error()
}

pub(crate) fn unsupported() -> String {
    ComponentFailure {
        stage: None,
        component: Component::GuestCompatibility,
        cause: FailureCause::UnsupportedGuest,
        code: None,
    }
    .into_error()
}

/// Mark refusal of a running guest before power/GPU/installation effects.
pub(crate) fn preflight(result: Result<(), String>) -> Result<(), String> {
    result.map_err(|error| {
        let mut failure = ComponentFailure::decode(&error)
            .filter(|failure| matches!(failure.component, Component::GuestCompatibility))
            .unwrap_or(ComponentFailure {
                stage: None,
                component: Component::GuestCompatibility,
                cause: FailureCause::Unavailable,
                code: None,
            });
        failure.stage = Some(crate::reporting::FailureStage::GuestOsPreflight);
        failure.into_error()
    })
}

pub(crate) fn require_supported(identity: &WindowsIdentity) -> Result<(), String> {
    if identity.major == 10
        && identity.minor == 0
        && identity.workstation
        && WINDOWS_11_BUILDS.contains(&identity.build)
    {
        Ok(())
    } else {
        Err(unsupported())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_windows_facts_require_matching_build_and_actual_client_type() {
        let client = WindowsIdentity::from_facts("10.0.26100", "26100", 1, "Client").unwrap();
        assert!(require_supported(&client).is_ok());
        for (version, build, product_type, kind) in [
            ("10.0.26100", "22631", 1, "Client"),
            ("10.0", "26100", 1, "Client"),
            ("10.0.26100.1", "26100", 1, "Client"),
            ("10.0.invalid", "26100", 1, "Client"),
            ("10.0.26100", "+26100", 1, "Client"),
            ("10.0.26100", "26100", 0, "Client"),
            ("10.0.26100", "26100", 1, ""),
        ] {
            assert!(WindowsIdentity::from_facts(version, build, product_type, kind).is_err());
        }
        for (product_type, kind) in [(3, "Server"), (2, "Server"), (1, "WindowsPE")] {
            let identity =
                WindowsIdentity::from_facts("10.0.26100", "26100", product_type, kind).unwrap();
            assert!(require_supported(&identity).is_err());
        }
    }

    #[test]
    fn only_explicit_windows_11_client_releases_are_eligible() {
        for build in WINDOWS_11_BUILDS {
            let mut identity = WindowsIdentity {
                major: 10,
                minor: 0,
                build: *build,
                workstation: true,
            };
            assert!(require_supported(&identity).is_ok());
            identity.workstation = false;
            assert!(
                require_supported(&identity).is_err(),
                "Server build {build}"
            );
        }
        for (major, minor, build) in [
            (10, 0, 19045),
            (10, 0, 99999),
            (11, 0, 26100),
            (10, 1, 26100),
            (0, 0, 0),
        ] {
            assert!(
                require_supported(&WindowsIdentity {
                    major,
                    minor,
                    build,
                    workstation: true
                })
                .is_err()
            );
        }
    }

    #[test]
    fn blocked_installation_has_safe_actionable_public_explanation() {
        let text = crate::reporting::operator_error("Apply", &unknown());
        assert!(text.contains("Windows 11"));
        assert!(text.contains("BitLocker"));
        assert!(text.contains("blocked"));
        assert!(text.contains("guest credentials"));
        assert!(text.contains("PowerShell Direct"));
        let unsupported = crate::reporting::operator_error("Apply", &unsupported());
        assert!(!unsupported.contains("start it"));
        let changed = ComponentFailure {
            stage: None,
            component: Component::GuestCompatibility,
            cause: FailureCause::ReadbackMismatch,
            code: None,
        }
        .into_error();
        assert!(!crate::reporting::operator_error("Apply", &changed).contains("start it"));
    }

    #[test]
    fn missing_or_malformed_identity_cannot_be_promoted_by_a_windows_name() {
        for json in [
            r#"{}"#,
            r#"{"name":"Windows 11"}"#,
            r#"{"major":10,"minor":0,"build":26100}"#,
            r#"{"major":10,"minor":0,"build":26100,"workstation":"true"}"#,
            r#"{"major":10,"minor":0,"build":26100,"workstation":true,"name":"Windows 11"}"#,
        ] {
            assert!(serde_json::from_str::<WindowsIdentity>(json).is_err());
        }
    }
}
