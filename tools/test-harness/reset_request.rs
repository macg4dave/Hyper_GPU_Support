// Development-only disposable VM automation; never compiled by default.
fn execute_reset_request(request: &Request, started: Instant) -> Result<String, ExecutionFailure> {
    let operation_id = operation_id().map_err(|_| ExecutionFailure {
        diagnostic: "operation-id-failed",
        operation_id: None,
    })?;
    match run_reset(request, &operation_id, started) {
        Ok(()) => Ok(operation_id),
        Err(_) => Err(ExecutionFailure {
            diagnostic: "operation-failed-reconciliation-required",
            operation_id: Some(operation_id),
        }),
    }
}

