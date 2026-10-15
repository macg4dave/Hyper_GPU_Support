//! Artifact replacement preserves authority and rebases pending policy receipts atomically.
use super::*;
use crate::enrollment::PendingPair;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema: u32,
    before: Option<Enrollment>,
    after: Enrollment,
    pair_before: Option<PendingPair>,
    pair_after: Option<PendingPair>,
}

trait Publication {
    fn policy(&mut self) -> Result<Enrollment, String>;
    fn pair(&mut self) -> Result<Option<PendingPair>, String>;
    fn write_policy(&mut self, policy: &Enrollment) -> Result<(), String>;
    fn write_pair(&mut self, pair: &PendingPair) -> Result<(), String>;
}

fn publish_receipt(pending: &Receipt, store: &mut impl Publication) -> Result<(), String> {
    let policy = store.policy()?;
    let pair = store.pair()?;
    let old = same(&Some(&policy), &pending.before)?;
    let new = same(&policy, &pending.after)?;
    if !((old && same(&pair, &pending.pair_before)?)
        || (new && (same(&pair, &pending.pair_before)? || same(&pair, &pending.pair_after)?)))
    {
        return Err("Reinstall publication binding changed; retain recovery".into());
    }
    if !new {
        store.write_policy(&pending.after)?;
    }
    if !same(&store.policy()?, &pending.after)? {
        return Err("Reinstall policy readback failed".into());
    }
    if !same(&store.pair()?, &pending.pair_after)? {
        store.write_pair(
            pending
                .pair_after
                .as_ref()
                .ok_or("Unexpected pair recovery appeared")?,
        )?;
    }
    if !same(&store.pair()?, &pending.pair_after)? {
        return Err("Reinstall pair readback failed".into());
    }
    Ok(())
}

struct ProtectedPublication;
impl Publication for ProtectedPublication {
    fn policy(&mut self) -> Result<Enrollment, String> {
        read_enrollment()
    }
    fn pair(&mut self) -> Result<Option<PendingPair>, String> {
        pending_pair()
    }
    fn write_policy(&mut self, policy: &Enrollment) -> Result<(), String> {
        atomic_json(&data_directory()?.join("enrollment.json"), policy)
    }
    fn write_pair(&mut self, pair: &PendingPair) -> Result<(), String> {
        atomic_json(&pair_pending_path()?, pair)
    }
}

fn same(a: &impl Serialize, b: &impl Serialize) -> Result<bool, String> {
    Ok(serde_json::to_value(a).map_err(|e| e.to_string())?
        == serde_json::to_value(b).map_err(|e| e.to_string())?)
}

fn scope(current: &Enrollment, configuration: &Configuration, client: &str) -> Result<(), String> {
    let pairs = |targets: &[Target]| {
        targets
            .iter()
            .map(|t| (t.vm_id.to_ascii_lowercase(), t.gpu_interface.clone()))
            .collect::<std::collections::BTreeSet<_>>()
    };
    if current.client_sid != client || pairs(&current.targets) != pairs(&configuration.targets) {
        return Err("Artifact reinstall must preserve the enrolled operator and exact VM/GPU pairs. Use the separate reviewed enrollment/removal operations.".into());
    }
    Ok(())
}

fn receipt() -> Result<Option<Receipt>, String> {
    let path = data_directory()?.join("installation-pending.json");
    match fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
        Ok(_) => crate::security::verify(&path)?,
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(FRAME_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > FRAME_LIMIT {
        return Err("Installation receipt exceeds input limit".into());
    }
    if serde_json::from_slice::<serde_json::Value>(&bytes).ok()
        == Some(serde_json::json!({"schema": 1}))
    {
        // Legacy CLI marker recorded no source package. Admission still preserves
        // any existing exact authority before upgrading to the bound receipt.
        return Ok(None);
    }
    let value: Receipt = serde_json::from_slice(&bytes).map_err(
        |_| "Installation receipt has no artifact-reinstall binding; retain setup recovery",
    )?;
    if value.schema != 2 {
        return Err("Unsupported reinstall receipt".into());
    }
    Ok(Some(value))
}

// Must run under the common lock, before scheduler/ACL publication.
pub(super) fn admit(
    configuration: &Configuration,
    client: &str,
    bootstrap: bool,
) -> Result<Option<Enrollment>, String> {
    let path = data_directory()?.join("enrollment.json");
    let current = match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.to_string()),
        Ok(_) => Some(read_enrollment()?),
    };
    if bootstrap {
        return Ok(current);
    }
    if let Some(current) = &current {
        scope(current, configuration, client)?;
    }
    if let Some(pending) = receipt()? {
        scope(&pending.after, configuration, client)?;
        if !(same(&current, &pending.before)? || same(&current, &Some(&pending.after))?)
            || !(same(&pending_pair()?, &pending.pair_before)?
                || same(&pending_pair()?, &pending.pair_after)?)
        {
            return Err(
                "Reinstall authority or recovery changed; retain installation recovery".into(),
            );
        }
        return Ok(pending.before.or(Some(pending.after)));
    }
    if current.is_none() {
        require_no_recovery()?;
        crate::windows_wmi::require_no_pending_operation()?;
    }
    Ok(current)
}

fn rebase(
    before: &Enrollment,
    after: &Enrollment,
    pending: &PendingPair,
) -> Result<PendingPair, String> {
    let mut other = before.clone();
    let actual = enrollment_revision(before)?;
    if actual == pending.review.expected {
        if pending.removal.is_some() {
            other.targets = pending
                .review
                .remove(&before.targets, &actual, &before.client_sid)?;
        } else {
            other.targets = pending
                .review
                .append(&before.targets, &actual, &before.client_sid)?;
        }
        if enrollment_revision(&other)? != pending.resulting {
            return Err("Pending resulting authority changed".into());
        }
    } else if actual == pending.resulting {
        // The old policy is historical once the authority write completed.
        // Preserve its exact reviewed token; never reconstruct/replay it from
        // identity-only intent (legacy CLI authority may include other fields).
        let mut updated = pending.clone();
        updated.resulting = enrollment_revision(after)?;
        return Ok(updated);
    } else {
        return Err("Pending authority has neither recorded revision".into());
    }
    other.artifacts = after.artifacts.clone();
    let mut updated = pending.clone();
    updated.review.expected = enrollment_revision(after)?;
    updated.resulting = enrollment_revision(&other)?;
    if let Some(removal) = &mut updated.removal {
        removal.approved.pair = updated.review.clone();
    }
    Ok(updated)
}

pub(super) fn begin(
    before: Option<&Enrollment>,
    configuration: &Configuration,
    client: &str,
    source: &Path,
) -> Result<(), String> {
    let mut artifacts = std::collections::BTreeMap::new();
    for name in [
        "hyper-gpu-runner.exe",
        "hyper-gpu-guest.exe",
        "d3d11-probe.exe",
        "hyper-gpu-support.exe",
    ] {
        let path = source.join(name);
        payload::no_reparse(&path)?;
        artifacts.insert(name.into(), payload::hash_file(&path)?);
    }
    if let Some(pending) = receipt()? {
        if pending.after.artifacts != artifacts {
            return Err("Resume installation using its originally bound artifacts".into());
        }
        return Ok(());
    }
    let after = Enrollment {
        schema: 2,
        client_sid: client.into(),
        targets: before.map_or_else(|| configuration.targets.clone(), |p| p.targets.clone()),
        artifacts,
    };
    let pair_before = pending_pair()?;
    let pair_after = match (&pair_before, before) {
        (Some(pair), Some(before)) => Some(rebase(before, &after, pair)?),
        (None, _) => None,
        _ => return Err("Pending pair requires existing authority".into()),
    };
    atomic_json(
        &data_directory()?.join("installation-pending.json"),
        &Receipt {
            schema: 2,
            before: before.cloned(),
            after,
            pair_before,
            pair_after,
        },
    )
}

pub(super) fn publish(enrollment: &Enrollment) -> Result<(), String> {
    let pending = receipt()?.ok_or("Missing reinstall receipt")?;
    if !same(enrollment, &pending.after)? {
        return Err("Reinstall artifacts changed after admission".into());
    }
    if pending.before.is_none() {
        // First CLI installation has no prior policy; the marker excludes operations.
        atomic_json(&data_directory()?.join("enrollment.json"), &pending.after)
    } else {
        publish_receipt(&pending, &mut ProtectedPublication)
    }
}

pub(super) fn finish(client: &str) -> Result<(), String> {
    let pending = receipt()?.ok_or("Missing reinstall receipt")?;
    if !same(&read_enrollment()?, &pending.after)? || !same(&pending_pair()?, &pending.pair_after)?
    {
        return Err("Reinstall authority readback failed; retain installation recovery".into());
    }
    verify_artifacts(&pending.after)?;
    let audit = data_directory()?.join("audit");
    match fs::symlink_metadata(&audit) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            crate::security::create_directory(&audit, client)?
        }
        Err(e) => return Err(e.to_string()),
        Ok(_) => crate::security::verify(&audit)?,
    }
    let crate::configuration_store::Revision::Content(token) =
        crate::configuration_store::Revision::of(
            &serde_json::to_vec(&pending).map_err(|e| e.to_string())?,
        )
    else {
        return Err("Missing installation revision".into());
    };
    atomic_json(&audit.join(format!("reinstall-{token}.json")), &pending)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{configuration_store::Revision, enrollment::PairReview};

    fn policy() -> Enrollment {
        Enrollment {
            schema: 2,
            client_sid: "operator".into(),
            targets: vec![target(1), target(2)],
            artifacts: std::collections::BTreeMap::from([("runner".into(), "old".into())]),
        }
    }
    fn target(n: u8) -> Target {
        Target {
            vm_id: format!("{n:08}-1111-1111-1111-111111111111"),
            gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#test\GPUPARAV".into(),
            enabled: false,
            vram: None,
            compute: None,
            encode: None,
            decode: None,
        }
    }
    fn configuration(policy: &Enrollment) -> Configuration {
        Configuration {
            schema: 2,
            targets: policy.targets.clone(),
        }
    }

    struct FakePublication {
        policy: Enrollment,
        pair: Option<PendingPair>,
        retained: std::collections::BTreeMap<&'static str, Vec<u8>>,
        fail_after: Option<&'static str>,
    }
    impl Publication for FakePublication {
        fn policy(&mut self) -> Result<Enrollment, String> {
            Ok(self.policy.clone())
        }
        fn pair(&mut self) -> Result<Option<PendingPair>, String> {
            Ok(self.pair.clone())
        }
        fn write_policy(&mut self, policy: &Enrollment) -> Result<(), String> {
            self.policy = policy.clone();
            if self.fail_after == Some("policy") {
                return Err("Interrupted policy publication".into());
            }
            Ok(())
        }
        fn write_pair(&mut self, pair: &PendingPair) -> Result<(), String> {
            self.pair = Some(pair.clone());
            if self.fail_after == Some("pair") {
                return Err("Interrupted recovery publication".into());
            }
            Ok(())
        }
    }
    #[test]
    fn interrupted_publication_preserves_all_other_holds_and_resumes_bound_revisions() {
        let before = policy();
        let mut after = before.clone();
        after.artifacts.insert("runner".into(), "new".into());
        let review = PairReview {
            vm_id: target(3).vm_id,
            gpu_interface: target(3).gpu_interface,
            expected: enrollment_revision(&before).unwrap(),
            operator_sid: before.client_sid.clone(),
        };
        let mut resulting = before.clone();
        resulting.targets.push(review.target().unwrap());
        let pair = PendingPair {
            operation_id: "0123456789abcdef0123456789abcdef".into(),
            review,
            resulting: enrollment_revision(&resulting).unwrap(),
            removal: None,
        };
        let receipt = Receipt {
            schema: 2,
            before: Some(before.clone()),
            after,
            pair_before: Some(pair.clone()),
            pair_after: Some(
                rebase(
                    &before,
                    &{
                        let mut after = before.clone();
                        after.artifacts.insert("runner".into(), "new".into());
                        after
                    },
                    &pair,
                )
                .unwrap(),
            ),
        };
        let retained = std::collections::BTreeMap::from([
            ("saved", b"saved intent".to_vec()),
            ("managed", b"managed history".to_vec()),
            ("effect", b"effect hold".to_vec()),
            ("save", b"save hold".to_vec()),
            ("import", b"import hold".to_vec()),
            ("native", b"native hold".to_vec()),
        ]);
        for failure in ["policy", "pair"] {
            let mut store = FakePublication {
                policy: before.clone(),
                pair: Some(pair.clone()),
                retained: retained.clone(),
                fail_after: Some(failure),
            };
            assert!(publish_receipt(&receipt, &mut store).is_err());
            assert_eq!(store.retained, retained);
            store.fail_after = None;
            publish_receipt(&receipt, &mut store).unwrap();
            assert!(same(&store.policy, &receipt.after).unwrap());
            assert!(same(&store.pair, &receipt.pair_after).unwrap());
            assert_eq!(store.retained, retained);
            publish_receipt(&receipt, &mut store).unwrap();
            assert_eq!(store.retained, retained);
        }
    }
    #[test]
    fn reinstall_rejects_subset_gpu_or_operator_replacement_before_publication() {
        let old = policy();
        for case in 0..4 {
            let mut requested = configuration(&old);
            let mut operator = old.client_sid.as_str();
            match case {
                0 => {
                    requested.targets.pop();
                }
                1 => requested.targets[0].gpu_interface.push_str("changed"),
                2 => operator = "different-operator",
                _ => requested.targets.push(target(3)),
            }
            let original = serde_json::to_vec(&old).unwrap();
            assert!(scope(&old, &requested, operator).is_err());
            assert_eq!(serde_json::to_vec(&old).unwrap(), original);
        }
        let mut requested = configuration(&old);
        requested.targets.reverse();
        requested.targets[0].enabled = true;
        assert!(scope(&old, &requested, &old.client_sid).is_ok());
        assert!(!old.targets[0].enabled); // desired settings never replace authority
    }
    #[test]
    fn pending_enrollment_preserves_both_revisions_across_artifact_reinstall() {
        let before = policy();
        let review = PairReview {
            vm_id: target(3).vm_id,
            gpu_interface: target(3).gpu_interface,
            expected: enrollment_revision(&before).unwrap(),
            operator_sid: before.client_sid.clone(),
        };
        let mut resulting = before.clone();
        resulting.targets.push(review.target().unwrap());
        let pending = PendingPair {
            operation_id: "0123456789abcdef0123456789abcdef".into(),
            review,
            resulting: enrollment_revision(&resulting).unwrap(),
            removal: None,
        };
        for original in [&before, &resulting] {
            let mut updated = original.clone();
            updated.artifacts.insert("runner".into(), "new".into());
            let rebased = rebase(original, &updated, &pending).unwrap();
            let actual = enrollment_revision(&updated).unwrap();
            assert!(actual == rebased.review.expected || actual == rebased.resulting);
            assert_eq!(rebased.review.vm_id, pending.review.vm_id);
            assert_eq!(rebased.operation_id, pending.operation_id);
            // A durable reinstall receipt can resume either side of publication.
            let receipt = Receipt {
                schema: 2,
                before: Some(original.clone()),
                after: updated,
                pair_before: Some(pending.clone()),
                pair_after: Some(rebased),
            };
            let recovered: Receipt =
                serde_json::from_slice(&serde_json::to_vec(&receipt).unwrap()).unwrap();
            assert!(same(&recovered.before, &Some(original)).unwrap());
            assert!(same(&recovered.pair_before, &Some(&pending)).unwrap());
        }
    }
    #[test]
    fn pending_removal_retains_snapshots_and_progress_on_both_sides_of_policy_write() {
        let before = policy();
        let review = PairReview {
            vm_id: target(1).vm_id,
            gpu_interface: target(1).gpu_interface,
            expected: enrollment_revision(&before).unwrap(),
            operator_sid: before.client_sid.clone(),
        };
        let mut resulting = before.clone();
        resulting.targets.remove(0);
        let snapshot =
            std::collections::BTreeMap::from([(review.vm_id.clone(), Revision::of(b"retained"))]);
        let pending = PendingPair {
            operation_id: "0123456789abcdef0123456789abcdef".into(),
            review: review.clone(),
            resulting: enrollment_revision(&resulting).unwrap(),
            removal: Some(crate::enrollment::RemovalState {
                approved: crate::enrollment::RemovalReview {
                    pair: review,
                    configurations: snapshot.clone(),
                    journals: snapshot.clone(),
                },
                configuration_removed: true,
                journal_removed: true,
            }),
        };
        for original in [&before, &resulting] {
            let mut updated = original.clone();
            updated.artifacts.insert("runner".into(), "new".into());
            let rebased = rebase(original, &updated, &pending).unwrap();
            let removal = rebased.removal.unwrap();
            assert_eq!(removal.approved.pair, rebased.review);
            assert_eq!(removal.approved.configurations, snapshot);
            assert_eq!(removal.approved.journals, snapshot);
            assert!(removal.configuration_removed && removal.journal_removed);
        }
    }
}
