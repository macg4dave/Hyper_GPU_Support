//! Bounded framing and wire types shared by the runner and pipe adapters.
use crate::{credentials::Credential, model::Target};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Fixed maximum authenticated protocol frame.
pub const FRAME_LIMIT: usize = 1024 * 1024;
/// Bounded protocol error.
#[derive(Debug)]
pub struct FrameError;
impl std::error::Error for FrameError {}
impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid runner frame")
    }
}
/// Read a bounded length-prefixed frame.
pub fn read_frame(r: &mut impl Read) -> Result<Vec<u8>, FrameError> {
    let mut h = [0; 4];
    r.read_exact(&mut h).map_err(|_| FrameError)?;
    let n = u32::from_le_bytes(h) as usize;
    if n > FRAME_LIMIT {
        return Err(FrameError);
    }
    let mut b = vec![0; n];
    r.read_exact(&mut b).map_err(|_| FrameError)?;
    Ok(b)
}
/// Publish one bounded frame.
pub fn write_frame(w: &mut impl Write, b: &[u8]) -> Result<(), FrameError> {
    if b.len() > FRAME_LIMIT {
        return Err(FrameError);
    }
    w.write_all(&(b.len() as u32).to_le_bytes())
        .and_then(|()| w.write_all(b))
        .and_then(|()| w.flush())
        .map_err(|_| FrameError)
}
/// Fixed product operations. No arbitrary scripts, source paths or host lifecycle commands.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Operation {
    /// Enumerate host GPUs and existing VMs.
    Discover,
    /// Inspect an enrolled target.
    Status,
    /// Inspect desired changes and authenticate the current preparation payload without effects.
    Plan,
    /// Predict ordinary changes from recorded receipts; admission authenticates afresh.
    PreliminaryPlan,
    /// Read-only preview of an explicitly requested complete driver reinstall.
    ReinstallPlan,
    /// Read-only review of exact pairing/profile record removal.
    RemovalPlan,
    /// Read-only preview of the guest verification lifecycle.
    VerifyPlan,
    /// Apply desired GPU state.
    Apply,
    /// Verify selected guest health and rendering.
    Verify,
}
/// Authenticated request. Credentials never enter journal/audit output.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// Protocol version.
    pub schema: u32,
    /// Unique request identity, consumed before mutation.
    pub nonce: String,
    /// Fixed operation.
    pub operation: Operation,
    /// Selected target, checked against protected enrollment.
    pub target: Option<Target>,
    /// Ephemeral guest credential, obtained by the unprivileged client.
    pub credential: Option<Credential>,
}
#[derive(Serialize, Deserialize)]
pub(super) struct Reply {
    pub(super) nonce: String,
    pub(super) result: Option<serde_json::Value>,
    pub(super) error: Option<String>,
}
/// Create a unique request with no credential persistence or logging.
pub fn request(
    operation: Operation,
    target: Option<Target>,
    credential: Option<Credential>,
) -> Request {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    Request {
        schema: 2,
        nonce: format!(
            "{time}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ),
        operation,
        target,
        credential,
    }
}
