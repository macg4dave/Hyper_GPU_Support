//! Bounded configuration import and explicit interrupted-publication reconciliation.
use super::{
    protocol::{Outcome, Progress, StageStatus},
    runner,
};
use crate::{
    configuration_store::{self, Revision},
    model::Configuration,
};

pub(super) fn import(
    enrollment: &runner::Enrollment,
    session: &str,
    source: String,
    expected: std::collections::BTreeMap<String, Revision>,
    mut notify: impl FnMut(Progress) -> Result<(), String>,
) -> Result<Outcome, String> {
    runner::require_no_recovery()?;
    crate::windows_wmi::require_no_pending_operation()?;
    let documents = Configuration::parse(&source)?.vm_documents()?;
    let ids = documents
        .iter()
        .map(|(filename, text)| {
            Configuration::parse_vm_file(text, filename).map(|mut c| c.targets.remove(0).vm_id)
        })
        .collect::<Result<Vec<_>, _>>()?;
    configuration_store::protected::check_capacity(&ids)?;
    if documents.len() != expected.len() {
        return Err("import destination scope mismatch".into());
    }
    for (filename, text) in &documents {
        let target = Configuration::parse_vm_file(text, filename)?
            .targets
            .remove(0);
        if expected.get(filename) != Some(&Revision::Missing)
            || configuration_store::protected::revision(&target.vm_id)? != Revision::Missing
        {
            return Err(format!(
                "import destination {filename} already exists or changed; explicit conflict resolution required"
            ));
        }
    }
    let mut record = configuration_store::ImportRecord {
        schema: 1,
        operation_id: session.into(),
        source_revision: Revision::of(source.as_bytes()),
        documents: documents
            .into_iter()
            .map(|(filename, source)| {
                (
                    filename,
                    configuration_store::ImportItem {
                        source,
                        published: false,
                    },
                )
            })
            .collect(),
    };
    runner::atomic_json(&runner::import_path()?, &record)?;
    publish_import_record(&mut record, &enrollment.client_sid, false, &mut notify)?;
    Ok(Outcome {
        operation: None,
        saved: true,
        finished: true,
        operation_id: session.into(),
        save_error: None,
    })
}

pub(super) fn resume(
    enrollment: &runner::Enrollment,
    operation_id: String,
    mut notify: impl FnMut(Progress) -> Result<(), String>,
) -> Result<Outcome, String> {
    if runner::pending_pair()?.is_some() {
        return Err("Finish the interrupted app-permission change before resuming import.".into());
    }
    if !runner::recovery_records()?.is_empty() {
        return Err("GPU recovery must remain distinct from import reconciliation".into());
    }
    let mut record = runner::import_record()?.ok_or("no interrupted import exists")?;
    if record.operation_id != operation_id {
        return Err("import operation identity mismatch".into());
    }
    publish_import_record(&mut record, &enrollment.client_sid, true, &mut notify)?;
    Ok(Outcome {
        operation: None,
        saved: true,
        finished: true,
        operation_id,
        save_error: None,
    })
}

fn publish_import_record(
    record: &mut configuration_store::ImportRecord,
    client: &str,
    resume: bool,
    notify: &mut impl FnMut(Progress) -> Result<(), String>,
) -> Result<(), String> {
    record.validate()?;
    // Recheck the complete destination set before writing any remaining file.
    for (filename, item) in &record.documents {
        let target = Configuration::parse_vm_file(&item.source, filename)?
            .targets
            .remove(0);
        let revision = configuration_store::protected::revision(&target.vm_id)?;
        if resume {
            record.validate_destination(filename, &revision)?;
        } else if revision != Revision::Missing {
            return Err(
                "import destination appeared after admission; explicit reconciliation required"
                    .into(),
            );
        }
    }
    for filename in record.documents.keys().cloned().collect::<Vec<_>>() {
        let source = record.documents[&filename].source.clone();
        let target = Configuration::parse_vm_file(&source, &filename)?
            .targets
            .remove(0);
        let current = configuration_store::protected::revision(&target.vm_id)?;
        record.validate_destination(&filename, &current)?;
        notify(Progress {
            stage: format!("Import {filename}"),
            status: StageStatus::Running,
        })?;
        if current != Revision::of(source.as_bytes()) {
            configuration_store::protected::publish_import(&filename, &source, client)?;
        }
        if configuration_store::protected::revision(&target.vm_id)?
            != Revision::of(source.as_bytes())
        {
            return Err("import publication readback changed".into());
        }
        record
            .documents
            .get_mut(&filename)
            .ok_or("import destination disappeared")?
            .published = true;
        runner::atomic_json(&runner::import_path()?, record)?;
        notify(Progress {
            stage: format!("Import {filename}"),
            status: StageStatus::Done,
        })?;
    }
    Ok(())
}
