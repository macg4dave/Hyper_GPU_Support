//! Exact captured plan details and allocation formatting shared with VM inspection.
use hyper_gpu_support::{
    model::{Allocation, Resource},
    workflow::Plan,
};

pub(in crate::gui) fn review_text(plan: &Plan) -> String {
    let requested = allocation_description(
        plan.desired.vram.as_ref(),
        hyper_gpu_support::ui_text::operations::apply::review::VRAM_UNSPECIFIED,
    );
    let planned = allocation_description(
        plan.preview.allocation.as_ref(),
        hyper_gpu_support::ui_text::operations::apply::review::VRAM_NO_WRITE,
    );
    let extended = [
        (Resource::Compute, plan.preview.compute.as_ref()),
        (Resource::Encode, plan.preview.encode.as_ref()),
        (Resource::Decode, plan.preview.decode.as_ref()),
    ]
    .into_iter()
    .map(|(resource, planned)| {
        format!(
            "{} draft (Minimum / Optimal / Maximum): {}\nPlanned {} write: {}",
            resource.name(),
            allocation_description(
                plan.desired.allocation(resource),
                hyper_gpu_support::ui_text::operations::apply::review::CATEGORY_UNSPECIFIED
            ),
            resource.name(),
            allocation_description(
                planned,
                hyper_gpu_support::ui_text::operations::apply::review::CATEGORY_NO_WRITE
            ),
        )
    })
    .collect::<Vec<_>>()
    .join("\n");
    format!(
        "VM: {}\nVM ID: {}\nGPU interface: {}\nDesired GPU support: {}\nVRAM draft (Minimum / Optimal / Maximum): {requested}\nPlanned allocation write: {planned}\n{extended}\n\n{}\n\nAllocation units: {}\nConfiguration publication: schema 3, retaining schema-2 input compatibility.\n\nPreview from the shared CLI planner. No VM or guest changes have been performed during review.",
        plan.observed.name,
        plan.desired.vm_id,
        plan.desired.gpu_interface,
        if plan.desired.enabled {
            hyper_gpu_support::ui_text::common::support::ENABLED
        } else {
            hyper_gpu_support::ui_text::common::support::DISABLED
        },
        plan.preview.summary.join("\n"),
        plan.allocation_units,
    )
}

pub(in crate::gui) fn allocation_description(value: Option<&Allocation>, absent: &str) -> String {
    value.map_or_else(
        || absent.to_owned(),
        |v| format!("{} / {} / {}", v.minimum, v.optimal, v.maximum),
    )
}
