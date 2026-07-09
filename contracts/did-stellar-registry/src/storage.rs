//! Storage layout and TTL management for the DID registry.
//!
//! Every DID record is stored in `persistent` storage, keyed by its 16-byte
//! `did_id`. TTL is extended on every read AND every write so that any DID
//! that is regularly resolved or mutated stays alive without manual rent
//! extension calls.

use crate::model::DidRecord;
use soroban_sdk::{contracttype, Address, BytesN, Env, Symbol};

// --- TTL ---

/// One day in ledgers, assuming a ~5s ledger close time.
const ONE_DAY_LEDGERS: u32 = 17_280;

/// Threshold for record TTL: extend if remaining TTL falls below ~30 days.
const LEDGER_THRESHOLD_DID: u32 = ONE_DAY_LEDGERS * 30;
/// Bump target for record TTL: extend up to ~180 days.
const LEDGER_BUMP_DID: u32 = ONE_DAY_LEDGERS * 180;

/// Threshold for instance storage TTL bumps.
const LEDGER_THRESHOLD_INSTANCE: u32 = ONE_DAY_LEDGERS * 30;
/// Bump target for instance storage TTL.
const LEDGER_BUMP_INSTANCE: u32 = ONE_DAY_LEDGERS * 90;

/// TTL for a pending admin proposal in temporary storage. After this many
/// ledgers (~10 days) the proposal expires and `accept_admin` will fail
/// with `NoProposedAdmin`. The current admin can always re-propose.
const PROPOSED_ADMIN_TTL: u32 = ONE_DAY_LEDGERS * 10;

/// Symbol-based key for the contract admin (instance storage).
const ADMIN_KEY: &str = "Admin";
/// Symbol-based key for the proposed admin (temporary storage).
const PROPOSED_ADMIN_KEY: &str = "PropAdmin";

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

// --- Admin (instance storage) -----------------------------------------------
// The admin lives in instance storage so it shares the contract's lifetime - // every contract invocation that touches admin extends the TTL via
// `extend_instance`. The proposed admin uses temporary storage so an
// unaccepted nomination auto-expires; this bounds the time window during
// which a stale proposal could be accepted.

/// Bump the contract instance TTL. Call from any admin operation.
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_INSTANCE, LEDGER_BUMP_INSTANCE);
}

/// Read the current admin. Panics if no admin is set - callers must ensure
/// the constructor has run before calling this. In practice the constructor
/// is always invoked at deploy time, so this invariant always holds.
pub fn get_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(e, ADMIN_KEY))
        .unwrap()
}

/// Write a new admin to instance storage.
pub fn set_admin(e: &Env, new_admin: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, ADMIN_KEY), new_admin);
}

// --- Proposed admin (temporary storage) -------------------------------------
// The proposed admin lives in temporary storage so an unaccepted proposal
// auto-expires (PROPOSED_ADMIN_TTL ~ 10 days). This bounds the window during
// which a stale proposal could be accepted.

/// Read the current proposed admin, or `None` if none / expired.
pub fn get_proposed_admin(e: &Env) -> Option<Address> {
    e.storage()
        .temporary()
        .get::<Symbol, Address>(&Symbol::new(e, PROPOSED_ADMIN_KEY))
}

/// Write a new proposed admin and extend its temporary-storage TTL.
pub fn set_proposed_admin(e: &Env, proposed_admin: &Address) {
    let key = Symbol::new(e, PROPOSED_ADMIN_KEY);
    e.storage()
        .temporary()
        .set::<Symbol, Address>(&key, proposed_admin);
    e.storage()
        .temporary()
        .extend_ttl(&key, PROPOSED_ADMIN_TTL, PROPOSED_ADMIN_TTL);
}

/// Remove any pending proposed admin. Called after a successful transfer.
pub fn remove_proposed_admin(e: &Env) {
    e.storage()
        .temporary()
        .remove(&Symbol::new(e, PROPOSED_ADMIN_KEY));
}
