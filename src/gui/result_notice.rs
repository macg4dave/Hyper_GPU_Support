//! Retained operation result for a status-only retry; never retained effect approval.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResultNotice {
    pub vm_id: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub details: String,
}

impl ResultNotice {
    pub fn matches_selection(&self, vm_id: &str) -> bool {
        self.vm_id == vm_id
    }
    pub fn unavailable_body(&self, error: &str) -> String {
        format!(
            "{}\n\nCurrent VM status is unavailable: {error}\nRetry the status update before another review.",
            self.body
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_failures_preserve_original_result_without_accumulating_retry_text() {
        let notice = ResultNotice {
            vm_id: "vm-1".into(),
            kind: "permission-saved".into(),
            title: "Permission saved".into(),
            body: "Permission saved; setup not confirmed.".into(),
            details: "reviewed pair".into(),
        };
        let original = notice.clone();
        let first = notice.unavailable_body("first read failed");
        let second = notice.unavailable_body("second read failed");
        assert!(first.contains(&notice.body));
        assert!(!second.contains("first read failed"));
        assert!(second.contains("second read failed"));
        assert_eq!(notice, original);
        assert!(notice.matches_selection("vm-1"));
        assert!(!notice.matches_selection("vm-2"));
    }
}
