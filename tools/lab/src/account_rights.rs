//! Fixed local-account rights for the least-privilege runner principal.

use std::collections::BTreeSet;

/// Rights required or denied for the dedicated runner account.
pub const RUNNER_RIGHTS: [&str; 5] = [
    "SeBatchLogonRight",
    "SeDenyInteractiveLogonRight",
    "SeDenyNetworkLogonRight",
    "SeDenyRemoteInteractiveLogonRight",
    "SeDenyServiceLogonRight",
];

/// Return the exact expected right set after a grant or revoke operation.
#[must_use]
pub fn expected_rights(current: &BTreeSet<String>, grant: bool) -> BTreeSet<String> {
    let mut expected = current.clone();
    for right in RUNNER_RIGHTS {
        if grant {
            expected.insert(right.to_owned());
        } else {
            expected.remove(right);
        }
    }
    expected
}

/// Return whether the fixed LSA mutation is necessary.
///
/// A revoke against an SID that has no LSA account-right record is already
/// complete and must remain a successful, idempotent recovery operation.
#[must_use]
pub fn update_required(current: &BTreeSet<String>, grant: bool) -> bool {
    RUNNER_RIGHTS
        .iter()
        .any(|right| current.contains(*right) != grant)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{RUNNER_RIGHTS, expected_rights, update_required};

    #[test]
    fn grant_and_revoke_preserve_unrelated_rights() {
        let current = BTreeSet::from(["SeChangeNotifyPrivilege".to_owned()]);
        let granted = expected_rights(&current, true);
        assert_eq!(granted.len(), RUNNER_RIGHTS.len() + 1);
        assert!(RUNNER_RIGHTS.iter().all(|right| granted.contains(*right)));
        assert_eq!(expected_rights(&granted, false), current);
    }

    #[test]
    fn pre_grant_and_repeated_revoke_need_no_native_update() {
        let no_rights = BTreeSet::new();
        assert!(!update_required(&no_rights, false));

        let unrelated_only = BTreeSet::from(["SeChangeNotifyPrivilege".to_owned()]);
        assert!(!update_required(&unrelated_only, false));
        assert_eq!(expected_rights(&unrelated_only, false), unrelated_only);

        let partial = BTreeSet::from([
            "SeBatchLogonRight".to_owned(),
            "SeChangeNotifyPrivilege".to_owned(),
        ]);
        assert!(update_required(&partial, false));
        assert_eq!(expected_rights(&partial, false), unrelated_only);
    }
}
