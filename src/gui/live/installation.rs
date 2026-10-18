//! Credentials are acquired only for explicitly reviewed guest work.
/// Name the current host operation independently of enrollment/driver history.
pub(super) fn apply_title(enabled: bool, attached: bool) -> &'static str {
    use hyper_gpu_support::ui_text::operations::apply;
    if enabled && !attached {
        apply::SETUP_TITLE
    } else if !enabled {
        apply::REMOVE_TITLE
    } else {
        apply::PROGRESS_TITLE
    }
}

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
    fn setup_and_removal_titles_follow_host_attachment_without_inferring_guest_health() {
        assert_eq!(
            apply_title(true, false),
            hyper_gpu_support::ui_text::operations::apply::SETUP_TITLE
        );
        assert_eq!(
            apply_title(true, true),
            hyper_gpu_support::ui_text::operations::apply::PROGRESS_TITLE
        );
        for attached in [false, true] {
            assert_eq!(
                apply_title(false, attached),
                hyper_gpu_support::ui_text::operations::apply::REMOVE_TITLE
            );
        }
    }

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
