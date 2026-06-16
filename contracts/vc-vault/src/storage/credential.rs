//! VC payload, O(1) index, parent links, and status storage.

use crate::constants::{PERSISTENT_TTL_EXTEND_TO, PERSISTENT_TTL_THRESHOLD};
use crate::error::ContractError;
use crate::types::{VCStatus, VerifiableCredential};
use super::VcVaultDataKey;
use soroban_sdk::{panic_with_error, Env, String};

// --- VC payloads ---

pub fn write_vault_vc(e: &Env, vc_id: &String, vc: &VerifiableCredential) {
    e.storage().persistent().set(&VcVaultDataKey::VaultVC(vc_id.clone()), vc)
}

pub fn read_vault_vc(e: &Env, vc_id: &String) -> Option<VerifiableCredential> {
    e.storage().persistent().get(&VcVaultDataKey::VaultVC(vc_id.clone()))
}

pub fn remove_vault_vc(e: &Env, vc_id: &String) {
    e.storage().persistent().remove(&VcVaultDataKey::VaultVC(vc_id.clone()));
}

// --- O(1) VC index ---
//
// Three persistent keys back the index:
//   VaultVCCount              -> u32 of active VCs
//   VaultVCIndex(position)    -> vc_id at that slot
//   VaultVCPosition(vc_id)    -> slot of that vc_id
//
// Append, remove, and existence-check are all O(1); enumeration is O(n).
// Removal uses swap-and-pop to keep slots dense.

pub fn read_vc_count(e: &Env) -> u32 {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VaultVCCount)
        .unwrap_or(0)
}

pub fn write_vc_count(e: &Env, count: u32) {
    let key = VcVaultDataKey::VaultVCCount;
    e.storage().persistent().set(&key, &count);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn read_vc_id_at(e: &Env, position: u32) -> Option<String> {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VaultVCIndex(position))
}

pub fn read_vc_id_at_extend(e: &Env, position: u32) -> Option<String> {
    let key = VcVaultDataKey::VaultVCIndex(position);
    if !e.storage().persistent().has(&key) {
        return None;
    }
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    e.storage().persistent().get(&key)
}

pub fn write_vc_id_at(e: &Env, position: u32, vc_id: &String) {
    let key = VcVaultDataKey::VaultVCIndex(position);
    e.storage().persistent().set(&key, vc_id);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_vc_id_at(e: &Env, position: u32) {
    e.storage()
        .persistent()
        .remove(&VcVaultDataKey::VaultVCIndex(position));
}

pub fn read_vc_position(e: &Env, vc_id: &String) -> Option<u32> {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VaultVCPosition(vc_id.clone()))
}

pub fn write_vc_position(e: &Env, vc_id: &String, position: u32) {
    let key = VcVaultDataKey::VaultVCPosition(vc_id.clone());
    e.storage().persistent().set(&key, &position);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_vc_position(e: &Env, vc_id: &String) {
    e.storage()
        .persistent()
        .remove(&VcVaultDataKey::VaultVCPosition(vc_id.clone()));
}

pub fn vc_index_contains(e: &Env, vc_id: &String) -> bool {
    e.storage()
        .persistent()
        .has(&VcVaultDataKey::VaultVCPosition(vc_id.clone()))
}

/// Append vc_id to the index. O(1). Panics with `VaultFull` on u32 overflow.
pub fn append_vc_to_index(e: &Env, vc_id: &String) {
    let count = read_vc_count(e);
    let next = count
        .checked_add(1)
        .unwrap_or_else(|| panic_with_error!(e, ContractError::VaultFull));
    write_vc_id_at(e, count, vc_id);
    write_vc_position(e, vc_id, count);
    write_vc_count(e, next);
}

/// Remove vc_id from the index using swap-and-pop. O(1).
pub fn remove_vc_from_index(e: &Env, vc_id: &String) {
    let position = match read_vc_position(e, vc_id) {
        Some(p) => p,
        None => return,
    };
    let count = read_vc_count(e);
    if count == 0 {
        return;
    }
    let last = count - 1;
    if position != last {
        if let Some(last_id) = read_vc_id_at(e, last) {
            write_vc_id_at(e, position, &last_id);
            write_vc_position(e, &last_id, position);
        }
    }
    remove_vc_id_at(e, last);
    remove_vc_position(e, vc_id);
    write_vc_count(e, last);
}

// --- VC parent links ---

pub fn write_vc_parent(e: &Env, vc_id: &String, parent_vc_id: &String) {
    let key = VcVaultDataKey::VCParent(vc_id.clone());
    e.storage()
        .persistent()
        .set(&key, parent_vc_id);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn read_vc_parent(e: &Env, vc_id: &String) -> Option<String> {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VCParent(vc_id.clone()))
}

pub fn has_vc_parent(e: &Env, vc_id: &String) -> bool {
    e.storage()
        .persistent()
        .has(&VcVaultDataKey::VCParent(vc_id.clone()))
}

pub fn remove_vc_parent(e: &Env, vc_id: &String) {
    e.storage()
        .persistent()
        .remove(&VcVaultDataKey::VCParent(vc_id.clone()));
}

// --- VC status ---

pub fn write_vc_status(e: &Env, vc_id: &String, status: &VCStatus) {
    e.storage()
        .persistent()
        .set(&VcVaultDataKey::VCStatus(vc_id.clone()), status)
}

pub fn read_vc_status(e: &Env, vc_id: &String) -> VCStatus {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VCStatus(vc_id.clone()))
        .unwrap_or(VCStatus::Invalid)
}

pub fn remove_vc_status(e: &Env, vc_id: &String) {
    e.storage()
        .persistent()
        .remove(&VcVaultDataKey::VCStatus(vc_id.clone()));
}
