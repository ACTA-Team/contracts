//! Denied issuer index storage. O(1) operations via swap-and-pop.

use crate::constants::{PERSISTENT_TTL_EXTEND_TO, PERSISTENT_TTL_THRESHOLD};
use super::VcVaultDataKey;
use soroban_sdk::{Address, Env};

// --- Denied issuer index ---

pub fn read_denied_issuer_count(e: &Env) -> u32 {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VaultDeniedIssuerCount)
        .unwrap_or(0)
}

pub fn write_denied_issuer_count(e: &Env, count: u32) {
    let key = VcVaultDataKey::VaultDeniedIssuerCount;
    e.storage().persistent().set(&key, &count);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn read_denied_issuer_at(e: &Env, position: u32) -> Option<Address> {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VaultDeniedIssuerIndex(position))
}

pub fn read_denied_issuer_at_extend(e: &Env, position: u32) -> Option<Address> {
    let key = VcVaultDataKey::VaultDeniedIssuerIndex(position);
    if !e.storage().persistent().has(&key) {
        return None;
    }
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    e.storage().persistent().get(&key)
}

pub fn write_denied_issuer_at(e: &Env, position: u32, issuer: &Address) {
    let key = VcVaultDataKey::VaultDeniedIssuerIndex(position);
    e.storage().persistent().set(&key, issuer);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_denied_issuer_at(e: &Env, position: u32) {
    e.storage()
        .persistent()
        .remove(&VcVaultDataKey::VaultDeniedIssuerIndex(position));
}

pub fn read_denied_issuer_position(e: &Env, issuer: &Address) -> Option<u32> {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VaultDeniedIssuerPosition(issuer.clone()))
}

pub fn write_denied_issuer_position(e: &Env, issuer: &Address, position: u32) {
    let key = VcVaultDataKey::VaultDeniedIssuerPosition(issuer.clone());
    e.storage().persistent().set(&key, &position);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_denied_issuer_position(e: &Env, issuer: &Address) {
    e.storage()
        .persistent()
        .remove(&VcVaultDataKey::VaultDeniedIssuerPosition(issuer.clone()));
}

pub fn denied_issuer_index_contains(e: &Env, issuer: &Address) -> bool {
    e.storage()
        .persistent()
        .has(&VcVaultDataKey::VaultDeniedIssuerPosition(issuer.clone()))
}

/// Append an issuer to the denied index. O(1). No-op if already present.
pub fn append_denied_issuer_to_index(e: &Env, issuer: &Address) {
    if denied_issuer_index_contains(e, issuer) {
        return;
    }
    let count = read_denied_issuer_count(e);
    write_denied_issuer_at(e, count, issuer);
    write_denied_issuer_position(e, issuer, count);
    write_denied_issuer_count(e, count + 1);
}

/// Remove an issuer from the denied index using swap-and-pop. O(1).
pub fn remove_denied_issuer_from_index(e: &Env, issuer: &Address) {
    let position = match read_denied_issuer_position(e, issuer) {
        Some(p) => p,
        None => return,
    };
    let count = read_denied_issuer_count(e);
    if count == 0 {
        return;
    }
    let last = count - 1;
    if position != last {
        let last_addr = read_denied_issuer_at(e, last).unwrap();
        write_denied_issuer_at(e, position, &last_addr);
        write_denied_issuer_position(e, &last_addr, position);
    }
    remove_denied_issuer_at(e, last);
    remove_denied_issuer_position(e, issuer);
    write_denied_issuer_count(e, last);
}

/// Clear the entire denied issuer index. O(n).
pub fn clear_denied_issuer_index(e: &Env) {
    let count = read_denied_issuer_count(e);
    for i in 0..count {
        if let Some(addr) = read_denied_issuer_at(e, i) {
            remove_denied_issuer_position(e, &addr);
        }
        remove_denied_issuer_at(e, i);
    }
    write_denied_issuer_count(e, 0);
}
