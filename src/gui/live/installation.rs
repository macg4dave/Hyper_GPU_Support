//! Credential acquisition order for an explicitly reviewed Apply operation.
pub(super) fn credentials<T>(
    installs_drivers: bool,
    needs_credentials: bool,
    preflight: impl FnOnce() -> Result<(), String>,
    acquire: impl FnOnce() -> Result<T, String>,
) -> Result<Option<T>, String> {
    if installs_drivers {
        preflight()?;
    }
    if needs_credentials {
        acquire().map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn blocked_installation_never_reads_the_vault_or_opens_the_prompt() {
        let result = credentials(
            true,
            true,
            || Err("unsupported guest".into()),
            || -> Result<(), String> { panic!("credential access before OS approval") },
        );
        assert_eq!(result, Err("unsupported guest".into()));
    }

    #[test]
    fn installation_checks_first_and_management_skips_the_os_check() {
        let events = RefCell::new(Vec::new());
        assert_eq!(
            credentials(
                true,
                true,
                || {
                    events.borrow_mut().push("os");
                    Ok(())
                },
                || {
                    events.borrow_mut().push("credentials");
                    Ok(7)
                }
            )
            .unwrap(),
            Some(7)
        );
        assert_eq!(*events.borrow(), ["os", "credentials"]);
        assert_eq!(
            credentials(
                false,
                false,
                || panic!("management OS query"),
                || -> Result<(), String> { panic!("management credentials") }
            )
            .unwrap(),
            None
        );
        assert_eq!(
            credentials(false, true, || panic!("verification OS query"), || Ok(9)).unwrap(),
            Some(9)
        );
    }
}
