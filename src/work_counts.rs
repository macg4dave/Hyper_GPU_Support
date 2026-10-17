//! Fixed-size, thread-local work accounting. Counts describe attempts, never authority.
use std::cell::Cell;

#[derive(Clone, Copy)]
pub(crate) enum Counter {
    ProviderConnections,
    ProviderQueries,
    ProviderGets,
    FileHashes,
    HashedBytes,
    CatalogHashes,
    PayloadPasses,
    ProcessStarts,
    VmReads,
    GpuReads,
    JournalReads,
    JournalWrites,
    PowerStarts,
    PowerShutdowns,
    AttachWrites,
    DetachWrites,
    SettingsWrites,
    VramWrites,
    ComputeWrites,
    EncodeWrites,
    DecodeWrites,
    HelperBytes,
    PayloadBytes,
    TransferChunks,
    CompletionReads,
    DiagnosticFlushes,
    DiagnosticFlushNs,
}

const NAMES: [&str; 27] = [
    "provider_connections",
    "provider_queries",
    "provider_gets",
    "file_hashes",
    "hashed_bytes",
    "catalog_hashes",
    "payload_passes",
    "process_starts",
    "vm_reads",
    "gpu_reads",
    "journal_reads",
    "journal_writes",
    "power_starts",
    "power_shutdowns",
    "attach_writes",
    "detach_writes",
    "settings_writes",
    "vram_writes",
    "compute_writes",
    "encode_writes",
    "decode_writes",
    "helper_bytes",
    "payload_bytes",
    "transfer_chunks",
    "completion_reads",
    "diagnostic_flushes",
    "diagnostic_flush_ns",
];
const _: () = assert!(Counter::DiagnosticFlushNs as usize + 1 == NAMES.len());

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Counts([u64; NAMES.len()]);

impl Counts {
    pub(crate) fn add(&mut self, counter: Counter, amount: u64) {
        let value = &mut self.0[counter as usize];
        *value = value.saturating_add(amount);
    }

    pub(crate) fn since(self, before: Self) -> Self {
        Self(std::array::from_fn(|i| {
            self.0[i].saturating_sub(before.0[i])
        }))
    }

    #[cfg(test)]
    pub(crate) fn get(self, counter: Counter) -> u64 {
        self.0[counter as usize]
    }
}

impl serde::Serialize for Counts {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(NAMES.len()))?;
        for (name, value) in NAMES.iter().zip(self.0) {
            map.serialize_entry(name, &value)?;
        }
        map.end()
    }
}

thread_local! {
    static COUNTS: Cell<Counts> = const { Cell::new(Counts([0; NAMES.len()])) };
}

pub(crate) fn snapshot() -> Counts {
    COUNTS.get()
}

/// Pure in-memory accounting, including when diagnostics are disabled. No logger,
/// allocation, identity probe, write, or fallible operation at the counted boundary.
pub(crate) fn add(counter: Counter, amount: u64) {
    COUNTS.with(|cell| {
        let mut counts = cell.get();
        counts.add(counter, amount);
        cell.set(counts);
    });
}

pub(crate) fn allocation(resource: crate::model::Resource) -> Counter {
    use crate::model::Resource;
    match resource {
        Resource::Vram => Counter::VramWrites,
        Resource::Compute => Counter::ComputeWrites,
        Resource::Encode => Counter::EncodeWrites,
        Resource::Decode => Counter::DecodeWrites,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_intervals_and_other_threads_do_not_steal_counts() {
        let outer = snapshot();
        add(Counter::ProviderQueries, 2);
        let inner = snapshot();
        add(Counter::ProviderQueries, 3);
        std::thread::spawn(|| {
            add(Counter::ProviderQueries, 100);
            assert_eq!(snapshot().get(Counter::ProviderQueries), 100);
        })
        .join()
        .unwrap();
        assert_eq!(snapshot().since(inner).get(Counter::ProviderQueries), 3);
        assert_eq!(snapshot().since(outer).get(Counter::ProviderQueries), 5);
    }

    #[test]
    fn schema_is_bounded_secret_free_and_saturates() {
        let mut counts = Counts::default();
        counts.add(Counter::PayloadBytes, u64::MAX);
        counts.add(Counter::PayloadBytes, 1);
        let value = serde_json::to_value(counts).unwrap();
        assert_eq!(value.as_object().unwrap().len(), NAMES.len());
        assert_eq!(value["payload_bytes"], u64::MAX);
        assert!(value.as_object().unwrap().values().all(|v| v.is_u64()));
        assert_eq!(counts.since(counts), Counts::default());
    }
}
