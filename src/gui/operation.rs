//! Local presentation identity, independent of editable titles and worker dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Read,
    StatusRefresh,
    Plan,
    VerifyPlan,
    Apply,
    Verify,
    Reconcile,
    SaveOnly,
    Credentials,
    Setup,
    Remove,
    Forget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GuestPurpose {
    Apply,
    Verify,
    Reconcile,
    Credentials,
    Reviewed,
}

#[derive(Clone, Copy)]
pub(super) struct Operation<'a> {
    pub kind: Kind,
    pub title: &'a str,
}

impl<'a> Operation<'a> {
    pub fn new(kind: Kind, title: &'a str) -> Self {
        Self { kind, title }
    }
}

impl Kind {
    pub fn operation_dialog(self) -> bool {
        matches!(
            self,
            Self::Apply
                | Self::Verify
                | Self::Reconcile
                | Self::SaveOnly
                | Self::Setup
                | Self::Remove
                | Self::Forget
        )
    }
    pub fn review_preparation(self) -> bool {
        matches!(self, Self::Plan | Self::VerifyPlan)
    }
    pub fn silent(self) -> bool {
        self == Self::StatusRefresh
    }

    pub fn guest_purpose(self) -> GuestPurpose {
        match self {
            Self::Apply => GuestPurpose::Apply,
            Self::Verify => GuestPurpose::Verify,
            Self::Reconcile => GuestPurpose::Reconcile,
            Self::Credentials => GuestPurpose::Credentials,
            _ => GuestPurpose::Reviewed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edited_titles_do_not_select_purpose_or_silent_reads() {
        let title: String = ['{', '}', '\n', 'λ'].into_iter().collect();
        for (kind, purpose) in [
            (Kind::Apply, GuestPurpose::Apply),
            (Kind::Verify, GuestPurpose::Verify),
            (Kind::Reconcile, GuestPurpose::Reconcile),
            (Kind::Credentials, GuestPurpose::Credentials),
            (Kind::Read, GuestPurpose::Reviewed),
            (Kind::Plan, GuestPurpose::Reviewed),
            (Kind::VerifyPlan, GuestPurpose::Reviewed),
            (Kind::SaveOnly, GuestPurpose::Reviewed),
            (Kind::StatusRefresh, GuestPurpose::Reviewed),
        ] {
            let operation = Operation::new(kind, &title);
            assert_eq!(operation.kind.guest_purpose(), purpose);
            assert_eq!(operation.kind.silent(), kind == Kind::StatusRefresh);
            assert_eq!(
                operation.kind.review_preparation(),
                matches!(kind, Kind::Plan | Kind::VerifyPlan)
            );
        }
    }
}
