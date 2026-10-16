//! Durable admission and terminal audit records exclude credentials and provider prose.
use super::protocol::{Operation, Request};
use crate::model::Target;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum AuditOutcome {
    Started,
    Succeeded,
    Failed,
}
// Deliberately excludes Request::credential, error text and response payloads.
// Failures retain their full error in the authenticated reply, not persistent logs.
#[derive(Serialize, Deserialize)]
pub(super) struct AuditRecord {
    schema: u32,
    nonce: String,
    operation: Operation,
    target: Option<Target>,
    outcome: AuditOutcome,
}
impl AuditRecord {
    pub(super) fn admission(request: &Request) -> Self {
        Self {
            schema: 1,
            nonce: request.nonce.clone(),
            operation: request.operation,
            target: request
                .target
                .as_ref()
                .filter(|t| t.validate().is_ok())
                .cloned(),
            outcome: AuditOutcome::Started,
        }
    }
}
pub(super) fn run_audited<T>(
    mut record: AuditRecord,
    mut publish: impl FnMut(&AuditRecord) -> Result<(), String>,
    operation: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    publish(&record)?;
    let result = operation();
    record.outcome = if result.is_ok() {
        AuditOutcome::Succeeded
    } else {
        AuditOutcome::Failed
    };
    if let Err(audit_error) = publish(&record) {
        return Err(match result {
            Ok(_) => format!(
                "operation completed but terminal audit publication failed: {audit_error}; inspect state before retry"
            ),
            Err(error) => format!(
                "{error}; terminal audit publication failed: {audit_error}; inspect state before retry"
            ),
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::super::protocol::request;
    use super::*;
    use crate::credentials::Credential;

    #[test]
    fn audit_admission_precedes_work_and_terminal_outcome_excludes_secrets() {
        for failed in [false, true] {
            let request = request(
                Operation::Status,
                Some(Target {
                    compute: None,
                    encode: None,
                    decode: None,
                    vm_id: "abcdef01-2345-6789-abcd-ef0123456789".into(),
                    gpu_interface: r"\\?\PCI#VEN_10DE&DEV_2D05#test\GPUPARAV".into(),
                    enabled: true,
                    vram: None,
                }),
                Some(Credential {
                    username: "private-user".into(),
                    password: "private-password".into(),
                }),
            );
            let published = std::cell::RefCell::new(Vec::new());
            let result = run_audited(
                AuditRecord::admission(&request),
                |record| {
                    published
                        .borrow_mut()
                        .push(serde_json::to_value(record).unwrap());
                    Ok(())
                },
                || {
                    assert_eq!(published.borrow()[0]["outcome"], "Started");
                    if failed {
                        Err("private-password provider error".into())
                    } else {
                        Ok("private-user response")
                    }
                },
            );
            assert_eq!(result.is_err(), failed);
            let values = published.into_inner();
            assert_eq!(values.len(), 2);
            assert_eq!(
                values[1]["outcome"],
                if failed { "Failed" } else { "Succeeded" }
            );
            let bytes = serde_json::to_string(&values).unwrap();
            assert!(!bytes.contains("private-"));
            assert!(!bytes.contains("credential"));
        }
    }

    #[test]
    fn audit_publication_failure_never_runs_unrecorded_work_or_reports_false_success() {
        let request = request(Operation::Discover, None, None);
        let result: Result<(), String> = run_audited(
            AuditRecord::admission(&request),
            |_| Err("admission unavailable".into()),
            || panic!("must not execute without durable admission"),
        );
        assert!(result.unwrap_err().contains("admission unavailable"));
        for failed in [false, true] {
            let published = std::cell::RefCell::new(Vec::new());
            let result: Result<(), String> = run_audited(
                AuditRecord::admission(&request),
                |record| {
                    if matches!(record.outcome, AuditOutcome::Started) {
                        published
                            .borrow_mut()
                            .push(serde_json::to_value(record).unwrap());
                        Ok(())
                    } else {
                        Err("disk failure".into())
                    }
                },
                || {
                    if failed {
                        Err("provider failed".into())
                    } else {
                        Ok(())
                    }
                },
            );
            let error = result.unwrap_err();
            assert!(error.contains("terminal audit publication failed"));
            assert!(!failed || error.contains("provider failed"));
            assert_eq!(published.borrow()[0]["outcome"], "Started");
        }
    }

    #[test]
    fn interrupted_operation_leaves_an_unfinished_admission() {
        let request = request(Operation::Discover, None, None);
        let published = std::cell::RefCell::new(Vec::new());
        let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _: Result<(), String> = run_audited(
                AuditRecord::admission(&request),
                |record| {
                    published
                        .borrow_mut()
                        .push(serde_json::to_value(record).unwrap());
                    Ok(())
                },
                || panic!("simulated interruption before an operation returns"),
            );
        }));
        assert!(interrupted.is_err());
        let values = published.into_inner();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0]["outcome"], "Started");
    }
}
