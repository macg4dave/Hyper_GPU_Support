use super::*;

#[test]
fn plan_details_preserve_exact_requests_and_distinguish_omitted_writes() {
    let mut plan = review_plan();
    let original = review_text(&plan);
    assert!(original.contains("GPU memory (minimum / optimal / maximum): 10 / 50 / 100"));
    assert!(original.contains("GPU memory to set: 10 / 50 / 100"));
    plan.desired.vram = Some(Allocation {
        minimum: 1,
        optimal: 500_000_001,
        maximum: 900_000_003,
    });
    plan.preview.allocation = plan.desired.vram.clone();
    assert_ne!(original, review_text(&plan));
    assert!(review_text(&plan).contains("1 / 500000001 / 900000003"));
    plan.preview.allocation = None;
    assert!(review_text(&plan).contains("GPU memory to set: Not changed"));
    plan.desired.vram = None;
    assert!(review_text(&plan).contains("GPU memory (minimum / optimal / maximum): Not set"));
}

#[test]
fn verify_details_keep_identity_and_payload_without_claiming_current_health() {
    let apply = review_plan();
    let verify = hyper_gpu_support::workflow::VerificationPlan {
        target: apply.desired,
        observed: apply.observed,
        gpu: apply.gpu,
        managed: apply.managed,
        payload_digest: "exact-payload".into(),
    };
    let details = crate::gui::wording::verify_details(&verify);
    for fact in [
        &verify.target.vm_id,
        &verify.target.gpu_interface,
        &verify.gpu.driver_version,
        &verify.payload_digest,
    ] {
        assert!(details.contains(fact));
    }
}
