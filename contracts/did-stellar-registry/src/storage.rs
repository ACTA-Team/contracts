//! Storage layout and TTL management for the DID registry.
//!
//! Every DID record is stored in `persistent` storage, keyed by its 16-byte
//! `did_id`. TTL is extended on every read AND every write so that any DID
//! that is regularly resolved or mutated stays alive without manual rent
//! extension calls.

use crate::model::DidRecord;
use soroban_sdk::{contracttype, BytesN, Env};

// --- TTL ---

/// One day in ledgers, assuming a ~5s ledger close time.
const ONE_DAY_LEDGERS: u32 = 17_280;

/// Threshold for record TTL: extend if remaining TTL falls below ~30 days.
const LEDGER_THRESHOLD_DID: u32 = ONE_DAY_LEDGERS * 30;
/// Bump target for record TTL: extend up to ~180 days.
const LEDGER_BUMP_DID: u32 = ONE_DAY_LEDGERS * 180;

// --- Storage keys ---

/// Persistent storage keys for the registry.
#[derive(Clone)]
#[contracttype]
pub enum DidDataKey {
    /// Per-DID record, keyed by the 16-byte `did_id`.
    Record(BytesN<16>),
}

// --- Helpers ---

/// Returns `true` if a record exists for `did_id`. Does NOT extend TTL.
pub fn has_record(e: &Env, did_id: &BytesN<16>) -> bool {
    e.storage()
        .persistent()
        .has(&DidDataKey::Record(did_id.clone()))
}

/// Reads the current `DidRecord` for `did_id`, extending its TTL as a side
/// effect. Returns `None` if the record does not exist.
pub fn read_record(e: &Env, did_id: &BytesN<16>) -> Option<DidRecord> {
    let key = DidDataKey::Record(did_id.clone());
    if let Some(record) = e.storage().persistent().get::<DidDataKey, DidRecord>(&key) {
        e.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD_DID, LEDGER_BUMP_DID);
        Some(record)
    } else {
        None
    }
}

/// Writes (or overwrites) the `DidRecord` for `did_id` and extends its TTL.
pub fn write_record(e: &Env, did_id: &BytesN<16>, record: &DidRecord) {
    let key = DidDataKey::Record(did_id.clone());
    e.storage().persistent().set(&key, record);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD_DID, LEDGER_BUMP_DID);
}
