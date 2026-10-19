//! Exact captured plan details and allocation formatting shared with VM inspection.
use hyper_gpu_support::{
    model::{Allocation, Resource},
    workflow::Plan,
};

pub(in crate::gui) fn review_text(plan: &Plan) -> String {
    let requested = allocation_description(
        plan.desired.vram.as_ref(),
        hyper_gpu_support::ui_text::operations::apply::review::vram_unspecified(),
    );
    let planned = allocation_description(
        plan.preview.allocation.as_ref(),
        hyper_gpu_support::ui_text::operations::apply::review::vram_no_write(),
    );
    let extended = [
        (Resource::Compute, plan.preview.compute.as_ref()),
        (Resource::Encode, plan.preview.encode.as_ref()),
        (Resource::Decode, plan.preview.decode.as_ref()),
    ]
    .into_iter()
    .map(|(resource, planned)| {
        hyper_gpu_support::ui_text::operations::apply::review::category(
            resource.name(),
            &allocation_description(
                plan.desired.allocation(resource),
                hyper_gpu_support::ui_text::operations::apply::review::category_unspecified(),
            ),
            &allocation_description(
                planned,
                hyper_gpu_support::ui_text::operations::apply::review::category_no_write(),
            ),
        )
        .into_string()
    })
    .collect::<Vec<_>>()
    .join("\n");
    hyper_gpu_support::ui_text::operations::apply::review::body(
        &plan.observed.name,
        &plan.desired.vm_id,
        &plan.desired.gpu_interface,
        if plan.desired.enabled {
            hyper_gpu_support::ui_text::common::support::enabled()
        } else {
            hyper_gpu_support::ui_text::common::support::disabled()
        },
        &requested,
        &planned,
        &extended,
        &plan.preview.summary.join("\n"),
        &plan.allocation_units.to_string(),
    )
    .into_string()
}

pub(in crate::gui) fn allocation_description(value: Option<&Allocation>, absent: &str) -> String {
    value.map_or_else(
        || absent.to_owned(),
        |v| format!("{} / {} / {}", v.minimum, v.optimal, v.maximum),
    )
}
