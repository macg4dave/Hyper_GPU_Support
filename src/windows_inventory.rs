//! Bounded, read-only native inventory. The fixed sibling Rust worker contains
//! synchronous COM/provider calls; the parent never loads PowerShell.

use crate::{
    config::ProjectConfiguration,
    inventory::{InventoryError, InventoryReport, InventorySource, parse_protocol},
};
use std::{path::Path, process::Command, time::Duration};

const MAX_OUTPUT: usize = 64 * 1024;

/// Executes fixed native inventory under a deadline and a kill-on-close job.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsInventory;

impl InventorySource for WindowsInventory {
    fn collect(&self) -> Result<InventoryReport, InventoryError> {
        let project =
            ProjectConfiguration::embedded().map_err(|_| InventoryError::InvalidProtocol)?;
        Self::collect_for(&project)
    }
}

impl WindowsInventory {
    /// Collect using the caller's already parsed, validated configuration.
    ///
    /// # Errors
    /// Preserves bounded-worker failures and rejects mismatched configuration or identities.
    pub fn collect_for(project: &ProjectConfiguration) -> Result<InventoryReport, InventoryError> {
        let executable = std::env::current_exe().map_err(|e| InventoryError::AdapterLaunch {
            kind: e.kind(),
            code: e.raw_os_error(),
        })?;
        let worker = executable.with_file_name("hyper-gpu-inventory-worker.exe");
        validate_worker(collect_worker(&worker, project.inventory.timeout)?, project)
    }
}

fn validate_worker(
    report: InventoryReport,
    project: &ProjectConfiguration,
) -> Result<InventoryReport, InventoryError> {
    use crate::inventory::FactStatus;
    let binding = report
        .facts()
        .iter()
        .find(|f| f.key() == "worker.configuration")
        .ok_or(InventoryError::InvalidProtocol)?;
    if binding.status() != FactStatus::Known
        || binding.value() != crate::probe::sha256_hex(include_bytes!("../config/project.toml"))
    {
        return Err(InventoryError::AdapterExit(
            None,
            "inventory worker configuration differs; rebuild/deploy CLI and worker together".into(),
        ));
    }
    let expected = [
        "host.edition",
        "host.version",
        "host.build",
        "host.architecture",
        "gpu.model",
        "gpu.pciid",
        "gpu.driverversion",
        "gpu.driverinf",
        "gpup.interface",
        "vm.count",
        "vm.selection",
        "vm.state",
        "vm.generation",
        "vm.version",
    ];
    if report.facts().len() != expected.len() + 1
        || expected
            .iter()
            .any(|key| !report.facts().iter().any(|f| f.key() == *key))
    {
        return Err(InventoryError::InvalidProtocol);
    }
    let device = crate::driver_environment::physical_device_id(&project.slot.gpu_interface)
        .map_err(|_| InventoryError::InvalidProtocol)?;
    let pci = device
        .split('\\')
        .nth(1)
        .ok_or(InventoryError::InvalidProtocol)?;
    for (key, identity) in [
        ("vm.selection", project.slot.vm_id.as_str()),
        ("gpup.interface", project.slot.gpu_interface.as_str()),
        ("gpu.pciid", pci),
    ] {
        let fact = report
            .facts()
            .iter()
            .find(|f| f.key() == key)
            .ok_or(InventoryError::InvalidProtocol)?;
        if fact.status() == FactStatus::Known && !fact.value().eq_ignore_ascii_case(identity) {
            return Err(InventoryError::InvalidProtocol);
        }
    }
    InventoryReport::new(
        report
            .facts()
            .iter()
            .filter(|f| f.key() != "worker.configuration")
            .cloned()
            .collect(),
    )
}

fn collect_worker(worker: &Path, timeout: Duration) -> Result<InventoryReport, InventoryError> {
    collect_command(Command::new(worker), timeout)
}

fn collect_command(command: Command, timeout: Duration) -> Result<InventoryReport, InventoryError> {
    let output =
        crate::windows_validation::bounded_process_with_limit(command, timeout, MAX_OUTPUT)
            .map_err(|e| match e {
                crate::windows_validation::BoundedProcessError::Launch { kind, code } => {
                    InventoryError::AdapterLaunch { kind, code }
                }
                crate::windows_validation::BoundedProcessError::Timeout => {
                    InventoryError::AdapterTimeout
                }
                crate::windows_validation::BoundedProcessError::OutputTooLarge => {
                    InventoryError::AdapterOutputTooLarge
                }
                crate::windows_validation::BoundedProcessError::Execution(message) => {
                    InventoryError::AdapterExit(None, bounded_diagnostic(&message))
                }
            })?;
    if output.exit_code == Some(6) {
        return Err(InventoryError::AdapterTimeout);
    }
    if output.exit_code != Some(0) {
        return Err(InventoryError::AdapterExit(
            output.exit_code,
            bounded_diagnostic(&output.stderr),
        ));
    }
    parse_protocol(&output.stdout)
}

fn bounded_diagnostic(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(256)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write, thread};

    fn fixture(mode: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "windows_inventory::tests::native_child_fixture",
                "--nocapture",
            ])
            .env("HYPER_GPU_INVENTORY_TEST_MODE", mode);
        command
    }
    #[test]
    fn native_child_fixture() {
        match std::env::var("HYPER_GPU_INVENTORY_TEST_MODE").as_deref() {
            Ok("sleep") => thread::sleep(Duration::from_secs(30)),
            Ok("overflow") => {
                std::io::stdout()
                    .write_all(&vec![b'A'; MAX_OUTPUT + 1024])
                    .unwrap();
                thread::sleep(Duration::from_secs(30));
            }
            Ok("exit") => {
                eprintln!("native failure 0x80041010");
                std::process::exit(7);
            }
            Ok("watchdog") => {
                let _deadline =
                    crate::windows_validation::WorkerDeadline::start(Duration::from_millis(50));
                thread::sleep(Duration::from_secs(30));
            }
            _ => (),
        }
    }
    #[test]
    fn native_process_deadline_overflow_and_exit_are_distinct() {
        let start = std::time::Instant::now();
        assert_eq!(
            collect_command(fixture("sleep"), Duration::from_millis(100)),
            Err(InventoryError::AdapterTimeout)
        );
        assert!(start.elapsed() < Duration::from_secs(5));
        assert_eq!(
            collect_command(fixture("overflow"), Duration::from_secs(5)),
            Err(InventoryError::AdapterOutputTooLarge)
        );
        assert_eq!(
            collect_command(fixture("watchdog"), Duration::from_secs(5)),
            Err(InventoryError::AdapterTimeout)
        );
        assert_eq!(
            collect_command(fixture("exit"), Duration::from_secs(5)),
            Err(InventoryError::AdapterExit(
                Some(7),
                "native failure 0x80041010".into()
            ))
        );
        assert!(matches!(
            collect_worker(
                Path::new("absent-inventory-worker.exe"),
                Duration::from_secs(1)
            ),
            Err(InventoryError::AdapterLaunch {
                kind: std::io::ErrorKind::NotFound,
                ..
            })
        ));
    }
    #[test]
    fn successful_but_malformed_worker_output_is_rejected() {
        assert_eq!(
            collect_command(fixture("normal"), Duration::from_secs(5)),
            Err(InventoryError::InvalidProtocol)
        );
    }

    fn worker_report(binding: &str, known: Option<(&str, &str)>) -> InventoryReport {
        use crate::inventory::{Fact, FactStatus};
        let mut facts = [
            "host.edition",
            "host.version",
            "host.build",
            "host.architecture",
            "gpu.model",
            "gpu.pciid",
            "gpu.driverversion",
            "gpu.driverinf",
            "gpup.interface",
            "vm.count",
            "vm.selection",
            "vm.state",
            "vm.generation",
            "vm.version",
        ]
        .into_iter()
        .map(|key| match known {
            Some((selected, value)) if key == selected => {
                Fact::new(key, FactStatus::Known, value).unwrap()
            }
            _ => Fact::new(key, FactStatus::Denied, "fixture unavailable").unwrap(),
        })
        .collect::<Vec<_>>();
        facts.push(Fact::new("worker.configuration", FactStatus::Known, binding).unwrap());
        InventoryReport::new(facts).unwrap()
    }
    #[test]
    fn rejects_stale_configuration_and_wrong_target_worker_reports() {
        let project = ProjectConfiguration::embedded().unwrap();
        let digest = crate::probe::sha256_hex(include_bytes!("../config/project.toml"));
        let current = worker_report(&digest, Some(("vm.selection", &project.slot.vm_id)));
        let decoded = parse_protocol(&current.encode_protocol()).unwrap();
        let report = validate_worker(decoded, &project).unwrap();
        assert_eq!(report.facts().len(), 14);
        assert!(!report.render().contains("worker.configuration"));
        assert!(
            matches!(validate_worker(worker_report(&"0".repeat(64), None), &project), Err(InventoryError::AdapterExit(None, detail)) if detail.contains("rebuild/deploy"))
        );
        for key in ["vm.selection", "gpup.interface", "gpu.pciid"] {
            assert_eq!(
                validate_worker(
                    worker_report(&digest, Some((key, "other target"))),
                    &project
                ),
                Err(InventoryError::InvalidProtocol)
            );
        }
        let incomplete = InventoryReport::new(vec![
            crate::inventory::Fact::new(
                "worker.configuration",
                crate::inventory::FactStatus::Known,
                digest,
            )
            .unwrap(),
        ])
        .unwrap();
        assert_eq!(
            validate_worker(incomplete, &project),
            Err(InventoryError::InvalidProtocol)
        );
    }
}
