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

pub(crate) fn unknown() -> String {
    ComponentFailure {
        stage: None,
        component: Component::GuestCompatibility,
        cause: FailureCause::Unavailable,
        code: None,
    }
    .into_error()
}

/// Consult host-read guest reports only when primary inspection was unavailable.
/// A positive rejection must never be overridden by cached metadata.
pub(crate) fn with_metadata_fallback(
    primary: Result<(), String>,
    fallback: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    match primary {
        Err(error)
            if matches!(
                ComponentFailure::decode(&error),
                Some(ComponentFailure {
                    component: Component::GuestCompatibility,
                    cause: FailureCause::Unavailable,
                    ..
                })
            ) =>
        {
            fallback()
        }
        outcome => outcome,
    }
}

pub(crate) fn require_supported(identity: &WindowsIdentity) -> Result<(), String> {
    if identity.major == 10
        && identity.minor == 0
        && identity.workstation
        && WINDOWS_11_BUILDS.contains(&identity.build)
    {
        Ok(())
    } else {
        Err(ComponentFailure {
            stage: None,
            component: Component::GuestCompatibility,
            cause: FailureCause::UnsupportedGuest,
            code: None,
        }
        .into_error())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_is_a_lazy_fallback_and_cannot_override_a_positive_rejection() {
        assert!(with_metadata_fallback(Ok(()), || panic!("unnecessary fallback")).is_ok());
        let unsupported = require_supported(&WindowsIdentity {
            major: 10,
            minor: 0,
            build: 19045,
            workstation: true,
        })
        .unwrap_err();
        assert_eq!(
            with_metadata_fallback(Err(unsupported.clone()), || panic!("unsupported override")),
            Err(unsupported),
        );
        assert!(with_metadata_fallback(Err(unknown()), || Ok(())).is_ok());
        assert!(with_metadata_fallback(Err(unknown()), || Err(unknown())).is_err());
        let changed = ComponentFailure {
            stage: None,
            component: Component::GuestCompatibility,
            cause: FailureCause::ReadbackMismatch,
            code: None,
        }
        .into_error();
        assert_eq!(
            with_metadata_fallback(Err(changed.clone()), || panic!("changed binding override")),
            Err(changed)
        );
        assert_eq!(
            with_metadata_fallback(Err("other failure".into()), || panic!(
                "unclassified override"
            )),
            Err("other failure".into())
        );
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
