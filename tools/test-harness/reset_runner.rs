// Development-only disposable VM automation; never compiled by default.
fn run_reset(
    request: &Request,
    operation_id: &str,
    started: Instant,
) -> Result<(), Box<dyn std::error::Error>> {
    let _enrollment = prepare_install_state()?;
    let project = project_configuration()?;
    let state_directory = project.runner.data_directory.join("state");
    let result_directory = project.runner.data_directory.join("results");
    let audit_directory = project.runner.data_directory.join("audit");
    let lock_path = state_directory.join("operation.lock");
    let _lock = LockFile::acquire(&lock_path)?;
    ensure_reconciled(&state_directory, Operation::ResetSlot).map_err(std::io::Error::other)?;
    run_reconciled_operation(
        &state_directory,
        &result_directory,
        &audit_directory,
        request,
        Operation::ResetSlot,
        operation_id,
        || {
            let output = run_worker(
                NativeAction::Reset,
                adapter_budget(
                    project.runner.task_execution_timeout,
                    Duration::ZERO,
                    started.elapsed(),
                ),
            )
            .map_err(std::io::Error::other)?;
            let result = parse_reset_result(&output)?;
            Ok(format!(
                concat!(
                    "{{\n  \"schema\": 1,\n  \"request_id\": \"{}\",\n  \"operation_id\": \"{}\",\n",
                    "  \"operation\": \"reset-slot\",\n  \"status\": \"succeeded\",\n",
                    "  \"vm_id\": \"{}\",\n  \"child\": \"{}\",\n",
                    "  \"parent\": \"{}\",\n  \"parent_sha256\": \"{}\"\n}}\n"
                ),
                request.request_id,
                operation_id,
                result.vm_id,
                json_escape(&result.child),
                json_escape(&result.parent),
                result.parent_sha256
            ))
        },
    )
}

