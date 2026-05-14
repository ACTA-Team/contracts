//! TTL extension helpers. Call from every read/write path to prevent ledger
//! entry archival.

use crate::constants::{
    INSTANCE_TTL_EXTEND_TO, INSTANCE_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO,
    PERSISTENT_TTL_THRESHOLD,
};
use super::credential::read_vc_position;
use super::DataKey;
use soroban_sdk::{Address, Env, String};

/// Extend instance TTL (admin, fees). Call from handlers that touch global state.
pub fn extend_instance_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

/// Extend TTL of vault keys. Call when reading/writing vault.
pub fn extend_vault_ttl(e: &Env, owner: &Address) {
    let keys = [
        DataKey::VaultAdmin(owner.clone()),
        DataKey::VaultDid(owner.clone()),
        DataKey::VaultRevoked(owner.clone()),
        DataKey::VaultIssuerCount(owner.clone()),
        DataKey::VaultDeniedIssuerCount(owner.clone()),
        DataKey::VaultVCCount(owner.clone()),
    ];
    for key in keys {
        if e.storage().persistent().has(&key) {
            e.storage()
                .persistent()
                .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
        }
    }
}

/// Extend TTL of VC payload, status, and the per-VC index entries. Call when
/// touching a VC. Index entries (`VaultVCIndex`, `VaultVCPosition`) are kept
/// alive only via reads/writes of the VC itself; they are not extended in
/// `extend_vault_ttl` because each is a distinct ledger entry per position.
pub fn extend_vc_ttl(e: &Env, owner: &Address, vc_id: &String) {
    let vc_key = DataKey::VaultVC(owner.clone(), vc_id.clone());
    let status_key = DataKey::VCStatus(owner.clone(), vc_id.clone());
    let position_key = DataKey::VaultVCPosition(owner.clone(), vc_id.clone());
    for key in [&vc_key, &status_key, &position_key] {
        if e.storage().persistent().has(key) {
            e.storage()
                .persistent()
                .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
        }
    }
    // Extend the index slot entry that points to this vc_id, if present.
    if let Some(pos) = read_vc_position(e, owner, vc_id) {
        let index_key = DataKey::VaultVCIndex(owner.clone(), pos);
        if e.storage().persistent().has(&index_key) {
            e.storage()
                .persistent()
                .extend_ttl(&index_key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
        }
    }
}

/// Extend TTL of VC status only. Call from revoke flow.
pub fn extend_vc_status_ttl(e: &Env, owner: &Address, vc_id: &String) {
    let key = DataKey::VCStatus(owner.clone(), vc_id.clone());
    if e.storage().persistent().has(&key) {
        e.storage()
            .persistent()
            .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    }
}
