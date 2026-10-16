//! Readable effect-plan summaries and exact allocation details.
use crate::gui::memory;
use hyper_gpu_support::{
    model::{Allocation, Resource},
    workflow::Plan,
};

/// Show readable VRAM requests; exact bytes remain in Technical Details.
pub(in crate::gui) fn memory_review(plan: &Plan) -> String {
    let readable = |allocation: Option<&Allocation>| {
        allocation.map_or_else(
            || hyper_gpu_support::ui_text::common::state::UNKNOWN.to_owned(),
            |value| {
                if value.minimum == value.optimal && value.optimal == value.maximum {
                    memory::format_gb(value.optimal)
                } else {
                    format!(
                        "{} / {} / {} (minimum / optimal / maximum)",
                        memory::format_gb(value.minimum),
                        memory::format_gb(value.optimal),
                        memory::format_gb(value.maximum)
                    )
                }
            },
        )
    };
    if let Some(write) = &plan.preview.allocation {
        format!(
            "GPU memory request: {} → {}. Exact bytes are in Technical Details.\n",
            readable(plan.observed.vram.as_ref()),
            readable(Some(write)),
        )
    } else if let Some(request) = &plan.desired.vram {
        format!(
            "GPU memory request: {}. No VRAM write is planned.\n",
            readable(Some(request)),
        )
    } else {
        hyper_gpu_support::ui_text::operations::apply::review::MEMORY_ABSENT.into()
    }
}

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

/// User-facing before/after changes, derived from the same approved effect plan.
pub(in crate::gui) fn review_summary(plan: &Plan) -> String {
    use hyper_gpu_support::workflow::PreviewAction;
    let mut lines = vec![
        hyper_gpu_support::ui_text::operations::apply::review::vm_heading(&(plan.observed.name))
            .into_string(),
        format!(
            "GPU support: {} → {}",
            if plan.observed.gpus.is_empty() {
                hyper_gpu_support::ui_text::common::support::DISABLED
            } else {
                hyper_gpu_support::ui_text::common::support::ENABLED
            },
            if plan.desired.enabled {
                hyper_gpu_support::ui_text::common::support::ENABLED
            } else {
                hyper_gpu_support::ui_text::common::support::DISABLED
            }
        ),
    ];
    if plan.reinstall {
        lines.insert(
            1,
            hyper_gpu_support::ui_text::operations::repair::review::BODY.into(),
        );
    }
    if !plan.desired.enabled {
        lines.insert(
            1,
            hyper_gpu_support::ui_text::operations::removal::review::BODY.into(),
        );
    }
    if plan.initial_setup && !plan.reinstall {
        lines.insert(
            1,
            hyper_gpu_support::ui_text::operations::setup::review::BODY.into(),
        );
    }
    if !plan.initial_setup
        && plan.preview.credentials_required
        && !plan.reinstall
        && plan.preview.actions.contains(&PreviewAction::Prepare)
    {
        lines.insert(
            1,
            hyper_gpu_support::ui_text::operations::setup::update_review::BODY.into(),
        );
    } else if !plan.initial_setup && plan.preview.credentials_required && !plan.reinstall {
        lines.insert(
            1,
            hyper_gpu_support::ui_text::operations::verify::apply_review::BODY.into(),
        );
    } else if !plan.preview.credentials_required {
        lines.insert(
            1,
            hyper_gpu_support::ui_text::operations::apply::ordinary_review::BODY.into(),
        );
    }
    if plan.authentication_deferred {
        lines.insert(
            1,
            hyper_gpu_support::ui_text::operations::plan::PRELIMINARY_NOTICE.into(),
        );
    }
    for (resource, write) in [
        (Resource::Vram, plan.preview.allocation.as_ref()),
        (Resource::Compute, plan.preview.compute.as_ref()),
        (Resource::Encode, plan.preview.encode.as_ref()),
        (Resource::Decode, plan.preview.decode.as_ref()),
    ] {
        // VRAM is summarised once in GB by memory_review; retain raw triples
        // for the other provider categories and in Technical Details.
        if resource == Resource::Vram {
            continue;
        }
        if let Some(write) = write {
            lines.push(format!(
                "{} (Minimum / Optimal / Maximum, raw provider values): {} → {}",
                resource.name(),
                allocation_description(
                    plan.observed.allocation(resource),
                    hyper_gpu_support::ui_text::common::state::UNKNOWN
                ),
                allocation_description(
                    Some(write),
                    hyper_gpu_support::ui_text::common::state::UNKNOWN
                )
            ));
        }
    }
    if let Some(settings) = &plan.preview.settings {
        for (name, before, after) in [
            (
                hyper_gpu_support::ui_text::common::compatibility::LOW_MMIO,
                settings.before.low_mmio.to_string(),
                settings.after.low_mmio.to_string(),
            ),
            (
                hyper_gpu_support::ui_text::common::compatibility::HIGH_MMIO,
                settings.before.high_mmio.to_string(),
                settings.after.high_mmio.to_string(),
            ),
            (
                hyper_gpu_support::ui_text::common::compatibility::CACHE_TYPES,
                settings.before.cache_types.to_string(),
                settings.after.cache_types.to_string(),
            ),
            (
                hyper_gpu_support::ui_text::common::compatibility::CHECKPOINTS,
                settings.before.automatic_checkpoints.to_string(),
                settings.after.automatic_checkpoints.to_string(),
            ),
        ] {
            if before != after {
                lines.push(format!("{name}: {before} → {after}"));
            }
        }
    }
    lines.push(hyper_gpu_support::ui_text::operations::apply::review::STEPS_HEADING.into());
    if plan.preview.actions.is_empty() {
        lines.push(hyper_gpu_support::ui_text::operations::apply::review::NO_EFFECTS.into());
    }
    lines.extend(
        plan.preview
            .actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let step = match action {
                    PreviewAction::Shutdown => {
                        hyper_gpu_support::ui_text::operations::apply::review::step::SHUTDOWN
                    }
                    PreviewAction::Start => {
                        hyper_gpu_support::ui_text::operations::apply::review::step::START
                    }
                    PreviewAction::Detach => {
                        hyper_gpu_support::ui_text::operations::apply::review::step::DETACH
                    }
                    PreviewAction::Prepare => {
                        hyper_gpu_support::ui_text::operations::apply::review::step::PREPARE
                    }
                    PreviewAction::Settings => {
                        hyper_gpu_support::ui_text::operations::apply::review::step::COMPATIBILITY
                    }
                    PreviewAction::Attach => {
                        hyper_gpu_support::ui_text::operations::apply::review::step::ATTACH
                    }
                    PreviewAction::Allocation => {
                        hyper_gpu_support::ui_text::operations::apply::review::step::ALLOCATION
                    }
                    PreviewAction::Verify => {
                        hyper_gpu_support::ui_text::operations::apply::review::step::VERIFY
                    }
                };
                format!("{}. {step}", index + 1)
            }),
    );
    lines.push(format!(
        "\nExpected final VM power after successful completion: {}.",
        match plan.preview.restore_power {
            hyper_gpu_support::model::Power::Running =>
                hyper_gpu_support::ui_text::common::power::RUNNING,
            hyper_gpu_support::model::Power::Off =>
                hyper_gpu_support::ui_text::common::power::STOPPED,
            hyper_gpu_support::model::Power::Other(_) =>
                hyper_gpu_support::ui_text::common::state::UNKNOWN,
        }
    ));
    if plan.preview.guest_downtime {
        lines.push(hyper_gpu_support::ui_text::operations::apply::review::DOWNTIME_WARNING.into());
    }
    if plan.preview.actions.contains(&PreviewAction::Prepare) {
        lines.push(hyper_gpu_support::ui_text::operations::apply::review::TRANSFER_TIME.into());
    }
    if plan.preview.actions.contains(&PreviewAction::Start)
        || plan.preview.actions.contains(&PreviewAction::Shutdown)
    {
        lines.push(hyper_gpu_support::ui_text::operations::apply::review::POWER_TIME.into());
    }
    if plan.preview.actions.contains(&PreviewAction::Verify) {
        lines.push(hyper_gpu_support::ui_text::operations::apply::review::VERIFY_TIME.into());
    }
    if !plan.preview.actions.is_empty() {
        lines.push(hyper_gpu_support::ui_text::operations::apply::review::DURATION_UNKNOWN.into());
    }
    if plan.preview.settings_preserved {
        lines
            .push(hyper_gpu_support::ui_text::operations::apply::review::SETTINGS_PRESERVED.into());
    }
    if plan.preview.pending_recovery {
        lines.push(hyper_gpu_support::ui_text::operations::apply::review::RECOVERY_REQUIRED.into());
    }
    lines.join("\n")
}

pub(in crate::gui) fn allocation_description(value: Option<&Allocation>, absent: &str) -> String {
    value.map_or_else(
        || absent.to_owned(),
        |v| format!("{} / {} / {}", v.minimum, v.optimal, v.maximum),
    )
}
