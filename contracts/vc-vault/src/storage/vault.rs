//! Vault metadata storage: admin, DID, revoked flag.

use super::VcVaultDataKey;
use soroban_sdk::{Address, Env, String};

pub fn has_vault_admin(e: &Env, owner: &Address) -> bool {
    e.storage().persistent().has(&VcVaultDataKey::VaultAdmin(owner.clone()))
}

pub fn read_vault_admin(e: &Env, owner: &Address) -> Address {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VaultAdmin(owner.clone()))
        .unwrap()
}

pub fn write_vault_admin(e: &Env, owner: &Address, admin: &Address) {
    e.storage()
        .persistent()
        .set(&VcVaultDataKey::VaultAdmin(owner.clone()), admin);
}

pub fn write_vault_did(e: &Env, owner: &Address, did: &String) {
    e.storage()
        .persistent()
        .set(&VcVaultDataKey::VaultDid(owner.clone()), did);
}

pub fn read_vault_did(e: &Env, owner: &Address) -> Option<String> {
    e.storage().persistent().get(&VcVaultDataKey::VaultDid(owner.clone()))
}

pub fn read_vault_revoked(e: &Env, owner: &Address) -> bool {
    e.storage()
        .persistent()
        .get(&VcVaultDataKey::VaultRevoked(owner.clone()))
        .unwrap_or(false)
}

pub fn write_vault_revoked(e: &Env, owner: &Address, revoked: &bool) {
    e.storage()
        .persistent()
        .set(&VcVaultDataKey::VaultRevoked(owner.clone()), revoked);
}
