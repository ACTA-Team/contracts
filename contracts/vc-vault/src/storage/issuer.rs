//! Authorized and denied issuer index storage. O(1) operations via swap-and-pop.

use crate::constants::{MAX_ISSUERS_LIST, PERSISTENT_TTL_EXTEND_TO, PERSISTENT_TTL_THRESHOLD};
use crate::error::ContractError;
use super::DataKey;
use soroban_sdk::{panic_with_error, Address, Env};

// --- Authorized issuer index ---

pub fn read_issuer_count(e: &Env, owner: &Address) -> u32 {
    e.storage()
        .persistent()
        .get(&DataKey::VaultIssuerCount(owner.clone()))
        .unwrap_or(0)
}

pub fn write_issuer_count(e: &Env, owner: &Address, count: u32) {
    let key = DataKey::VaultIssuerCount(owner.clone());
    e.storage().persistent().set(&key, &count);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn read_issuer_at(e: &Env, owner: &Address, position: u32) -> Option<Address> {
    e.storage()
        .persistent()
        .get(&DataKey::VaultIssuerIndex(owner.clone(), position))
}

pub fn read_issuer_at_extend(e: &Env, owner: &Address, position: u32) -> Option<Address> {
    let key = DataKey::VaultIssuerIndex(owner.clone(), position);
    if !e.storage().persistent().has(&key) {
        return None;
    }
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    e.storage().persistent().get(&key)
}

pub fn write_issuer_at(e: &Env, owner: &Address, position: u32, issuer: &Address) {
    let key = DataKey::VaultIssuerIndex(owner.clone(), position);
    e.storage().persistent().set(&key, issuer);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_issuer_at(e: &Env, owner: &Address, position: u32) {
    e.storage()
        .persistent()
        .remove(&DataKey::VaultIssuerIndex(owner.clone(), position));
}

pub fn read_issuer_position(e: &Env, owner: &Address, issuer: &Address) -> Option<u32> {
    e.storage()
        .persistent()
        .get(&DataKey::VaultIssuerPosition(owner.clone(), issuer.clone()))
}

pub fn write_issuer_position(e: &Env, owner: &Address, issuer: &Address, position: u32) {
    let key = DataKey::VaultIssuerPosition(owner.clone(), issuer.clone());
    e.storage().persistent().set(&key, &position);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_issuer_position(e: &Env, owner: &Address, issuer: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::VaultIssuerPosition(owner.clone(), issuer.clone()));
}

pub fn issuer_index_contains(e: &Env, owner: &Address, issuer: &Address) -> bool {
    e.storage()
        .persistent()
        .has(&DataKey::VaultIssuerPosition(owner.clone(), issuer.clone()))
}

pub fn append_issuer_to_index(e: &Env, owner: &Address, issuer: &Address) {
    let count = read_issuer_count(e, owner);
    if count >= MAX_ISSUERS_LIST {
        panic_with_error!(e, ContractError::IssuerListTooLong);
    }
    write_issuer_at(e, owner, count, issuer);
    write_issuer_position(e, owner, issuer, count);
    write_issuer_count(e, owner, count + 1);
}

pub fn remove_issuer_from_index(e: &Env, owner: &Address, issuer: &Address) {
    let position = match read_issuer_position(e, owner, issuer) {
        Some(p) => p,
        None => return,
    };
    let count = read_issuer_count(e, owner);
    if count == 0 {
        return;
    }
    let last = count - 1;
    if position != last {
        if let Some(last_addr) = read_issuer_at(e, owner, last) {
            write_issuer_at(e, owner, position, &last_addr);
            write_issuer_position(e, owner, &last_addr, position);
        }
    }
    remove_issuer_at(e, owner, last);
    remove_issuer_position(e, owner, issuer);
    write_issuer_count(e, owner, last);
}

pub fn clear_issuer_index(e: &Env, owner: &Address) {
    let count = read_issuer_count(e, owner);
    for i in 0..count {
        if let Some(addr) = read_issuer_at(e, owner, i) {
            remove_issuer_position(e, owner, &addr);
        }
        remove_issuer_at(e, owner, i);
    }
    write_issuer_count(e, owner, 0);
}

// --- Denied issuer index ---

pub fn read_denied_issuer_count(e: &Env, owner: &Address) -> u32 {
    e.storage()
        .persistent()
        .get(&DataKey::VaultDeniedIssuerCount(owner.clone()))
        .unwrap_or(0)
}

pub fn write_denied_issuer_count(e: &Env, owner: &Address, count: u32) {
    let key = DataKey::VaultDeniedIssuerCount(owner.clone());
    e.storage().persistent().set(&key, &count);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn read_denied_issuer_at(e: &Env, owner: &Address, position: u32) -> Option<Address> {
    e.storage()
        .persistent()
        .get(&DataKey::VaultDeniedIssuerIndex(owner.clone(), position))
}

pub fn read_denied_issuer_at_extend(e: &Env, owner: &Address, position: u32) -> Option<Address> {
    let key = DataKey::VaultDeniedIssuerIndex(owner.clone(), position);
    if !e.storage().persistent().has(&key) {
        return None;
    }
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    e.storage().persistent().get(&key)
}

pub fn write_denied_issuer_at(e: &Env, owner: &Address, position: u32, issuer: &Address) {
    let key = DataKey::VaultDeniedIssuerIndex(owner.clone(), position);
    e.storage().persistent().set(&key, issuer);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_denied_issuer_at(e: &Env, owner: &Address, position: u32) {
    e.storage()
        .persistent()
        .remove(&DataKey::VaultDeniedIssuerIndex(owner.clone(), position));
}

pub fn read_denied_issuer_position(e: &Env, owner: &Address, issuer: &Address) -> Option<u32> {
    e.storage()
        .persistent()
        .get(&DataKey::VaultDeniedIssuerPosition(owner.clone(), issuer.clone()))
}

pub fn write_denied_issuer_position(e: &Env, owner: &Address, issuer: &Address, position: u32) {
    let key = DataKey::VaultDeniedIssuerPosition(owner.clone(), issuer.clone());
    e.storage().persistent().set(&key, &position);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_denied_issuer_position(e: &Env, owner: &Address, issuer: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::VaultDeniedIssuerPosition(owner.clone(), issuer.clone()));
}

pub fn denied_issuer_index_contains(e: &Env, owner: &Address, issuer: &Address) -> bool {
    e.storage()
        .persistent()
        .has(&DataKey::VaultDeniedIssuerPosition(owner.clone(), issuer.clone()))
}

/// Append an issuer to the denied index. O(1). No-op if already present.
pub fn append_denied_issuer_to_index(e: &Env, owner: &Address, issuer: &Address) {
    if denied_issuer_index_contains(e, owner, issuer) {
        return;
    }
    let count = read_denied_issuer_count(e, owner);
    write_denied_issuer_at(e, owner, count, issuer);
    write_denied_issuer_position(e, owner, issuer, count);
    write_denied_issuer_count(e, owner, count + 1);
}

/// Remove an issuer from the denied index using swap-and-pop. O(1).
pub fn remove_denied_issuer_from_index(e: &Env, owner: &Address, issuer: &Address) {
    let position = match read_denied_issuer_position(e, owner, issuer) {
        Some(p) => p,
        None => return,
    };
    let count = read_denied_issuer_count(e, owner);
    if count == 0 {
        return;
    }
    let last = count - 1;
    if position != last {
        if let Some(last_addr) = read_denied_issuer_at(e, owner, last) {
            write_denied_issuer_at(e, owner, position, &last_addr);
            write_denied_issuer_position(e, owner, &last_addr, position);
        }
    }
    remove_denied_issuer_at(e, owner, last);
    remove_denied_issuer_position(e, owner, issuer);
    write_denied_issuer_count(e, owner, last);
}

/// Clear the entire denied issuer index. O(n).
pub fn clear_denied_issuer_index(e: &Env, owner: &Address) {
    let count = read_denied_issuer_count(e, owner);
    for i in 0..count {
        if let Some(addr) = read_denied_issuer_at(e, owner, i) {
            remove_denied_issuer_position(e, owner, &addr);
        }
        remove_denied_issuer_at(e, owner, i);
    }
    write_denied_issuer_count(e, owner, 0);
}
