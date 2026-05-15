//! VC payload, O(1) index, parent links, and status storage.

use crate::constants::{PERSISTENT_TTL_EXTEND_TO, PERSISTENT_TTL_THRESHOLD};
use crate::error::ContractError;
use crate::types::{VCStatus, VerifiableCredential};
use super::VcVaultDataKey;
use soroban_sdk::{panic_with_error, Address, Env, String};

// --- VC payloads ---

pub fn write_vault_vc(e: &Env, owner: &Address, vc_id: &String, vc: &VerifiableCredential) {
    e.storage().persistent().set(&VcVaultDataKey::VaultVC(owner.clone(), vc_id.clone()), vc)
}

pub fn read_vault_vc(e: &Env, owner: &Address, vc_id: &String) -> Option<VerifiableCredential> {
    e.storage().persistent().get(&VcVaultDataKey::VaultVC(owner.clone(), vc_id.clone()))
}

pub fn remove_vault_vc(e: &Env, owner: &Address, vc_id: &String) {
    e.storage().persistent().remove(&VcVaultDataKey::VaultVC(owner.clone(), vc_id.clone()));
}

// --- O(1) VC index ---
//
// Three persistent keys per vault back the index:
//   VaultVCCount(owner)              -> u32 of active VCs
//   VaultVCIndex(owner, position)    -> vc_id at that slot
//   VaultVCPosition(owner, vc_id)    -> slot of that vc_id
//
// Append, remove, and existence-check are all O(1); enumeration is O(n).
// Removal uses swap-and-pop to keep slots dense.

pub fn read_vc_count(e: &Env, owner: &Address) -> u32 {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VaultVCCount(owner.clone()))
        .unwrap_or(0)
}

pub fn write_vc_count(e: &Env, owner: &Address, count: u32) {
    let key = VcVaultDataKey::VaultVCCount(owner.clone());
    e.storage().persistent().set(&key, &count);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn read_vc_id_at(e: &Env, owner: &Address, position: u32) -> Option<String> {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VaultVCIndex(owner.clone(), position))
}

/// Read a slot and refresh its TTL in a single call. Use this from enumeration
/// paths (e.g. `list_vc_ids`) so that callers who only ever list — without
/// touching individual VCs — keep the index alive. Returns None if the slot
/// has been archived/never written.
pub fn read_vc_id_at_extend(e: &Env, owner: &Address, position: u32) -> Option<String> {
    let key = VcVaultDataKey::VaultVCIndex(owner.clone(), position);
    if !e.storage().persistent().has(&key) {
        return None;
    }
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    e.storage().persistent().get(&key)
}

pub fn write_vc_id_at(e: &Env, owner: &Address, position: u32, vc_id: &String) {
    let key = VcVaultDataKey::VaultVCIndex(owner.clone(), position);
    e.storage().persistent().set(&key, vc_id);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_vc_id_at(e: &Env, owner: &Address, position: u32) {
    e.storage()
        .persistent()
        .remove(&VcVaultDataKey::VaultVCIndex(owner.clone(), position));
}

pub fn read_vc_position(e: &Env, owner: &Address, vc_id: &String) -> Option<u32> {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VaultVCPosition(owner.clone(), vc_id.clone()))
}

pub fn write_vc_position(e: &Env, owner: &Address, vc_id: &String, position: u32) {
    let key = VcVaultDataKey::VaultVCPosition(owner.clone(), vc_id.clone());
    e.storage().persistent().set(&key, &position);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_vc_position(e: &Env, owner: &Address, vc_id: &String) {
    e.storage()
        .persistent()
        .remove(&VcVaultDataKey::VaultVCPosition(owner.clone(), vc_id.clone()));
}

/// Returns true when vc_id has a recorded position in the active index.
pub fn vc_index_contains(e: &Env, owner: &Address, vc_id: &String) -> bool {
    e.storage()
        .persistent()
        .has(&VcVaultDataKey::VaultVCPosition(owner.clone(), vc_id.clone()))
}

/// Append vc_id to the index. O(1). Panics with `VaultFull` on u32 overflow.
pub fn append_vc_to_index(e: &Env, owner: &Address, vc_id: &String) {
    let count = read_vc_count(e, owner);
    let next = count
        .checked_add(1)
        .unwrap_or_else(|| panic_with_error!(e, ContractError::VaultFull));
    write_vc_id_at(e, owner, count, vc_id);
    write_vc_position(e, owner, vc_id, count);
    write_vc_count(e, owner, next);
}

/// Remove vc_id from the index using swap-and-pop. O(1). No-op when vc_id is
/// not indexed.
pub fn remove_vc_from_index(e: &Env, owner: &Address, vc_id: &String) {
    let position = match read_vc_position(e, owner, vc_id) {
        Some(p) => p,
        None => return,
    };
    let count = read_vc_count(e, owner);
    if count == 0 {
        return;
    }
    let last = count - 1;
    if position != last {
        // Tail slot must exist if count is consistent; panic to avoid
        // partial mutation that would leave a stale forward index entry.
        let last_id = read_vc_id_at(e, owner, last).unwrap();
        write_vc_id_at(e, owner, position, &last_id);
        write_vc_position(e, owner, &last_id, position);
    }
    remove_vc_id_at(e, owner, last);
    remove_vc_position(e, owner, vc_id);
    write_vc_count(e, owner, last);
}

// --- VC parent links ---

/// Write a parent link: (owner, vc_id) → (parent_owner, parent_vc_id).
pub fn write_vc_parent(
    e: &Env,
    owner: &Address,
    vc_id: &String,
    parent_owner: &Address,
    parent_vc_id: &String,
) {
    let key = VcVaultDataKey::VCParent(owner.clone(), vc_id.clone());
    e.storage()
        .persistent()
        .set(&key, &(parent_owner.clone(), parent_vc_id.clone()));
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

/// Read the parent link for a VC. Returns None if the VC has no parent.
pub fn read_vc_parent(e: &Env, owner: &Address, vc_id: &String) -> Option<(Address, String)> {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VCParent(owner.clone(), vc_id.clone()))
}

/// Return true if the VC has a recorded parent link.
pub fn has_vc_parent(e: &Env, owner: &Address, vc_id: &String) -> bool {
    e.storage()
        .persistent()
        .has(&VcVaultDataKey::VCParent(owner.clone(), vc_id.clone()))
}

/// Remove a parent link entry.
pub fn remove_vc_parent(e: &Env, owner: &Address, vc_id: &String) {
    e.storage()
        .persistent()
        .remove(&VcVaultDataKey::VCParent(owner.clone(), vc_id.clone()));
}

// --- VC status ---

/// VC status keyed by (owner, vc_id) to prevent cross-vault collisions.
pub fn write_vc_status(e: &Env, owner: &Address, vc_id: &String, status: &VCStatus) {
    e.storage()
        .persistent()
        .set(&VcVaultDataKey::VCStatus(owner.clone(), vc_id.clone()), status)
}

pub fn read_vc_status(e: &Env, owner: &Address, vc_id: &String) -> VCStatus {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VCStatus(owner.clone(), vc_id.clone()))
        .unwrap_or(VCStatus::Invalid)
}

/// Remove the status entry. After removal the default `Invalid` is returned by
/// `read_vc_status`.
pub fn remove_vc_status(e: &Env, owner: &Address, vc_id: &String) {
    e.storage()
        .persistent()
        .remove(&VcVaultDataKey::VCStatus(owner.clone(), vc_id.clone()));
}
