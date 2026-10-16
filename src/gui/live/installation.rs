//! Credentials are acquired only for explicitly reviewed guest work.
pub(super) fn credentials<T>(
    needs_credentials: bool,
    acquire: impl FnOnce() -> Result<T, String>,
) -> Result<Option<T>, String> {
    if needs_credentials {
        acquire().map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_reviewed_guest_work_acquires_credentials_and_cancellation_stops_submission() {
        assert_eq!(credentials(true, || Ok(7)).unwrap(), Some(7));
        assert_eq!(
            credentials(false, || -> Result<(), String> {
                panic!("management credentials")
            })
            .unwrap(),
            None
        );
        assert_eq!(
            credentials::<()>(true, || Err("cancelled".into())),
            Err("cancelled".into())
        );
    }
}
