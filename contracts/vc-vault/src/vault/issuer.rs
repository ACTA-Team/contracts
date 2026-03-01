//! Issuer list management: add, remove, replace authorized issuers per vault.

use crate::error::ContractError;
use crate::storage;
use soroban_sdk::{panic_with_error, Address, Env, Vec};

/// Add single issuer to vault. Panics if already authorized.
pub fn authorize_issuer(e: &Env, owner: &Address, issuer: &Address) {
    let mut issuers: Vec<Address> = storage::read_vault_issuers(e, owner);
    if is_authorized(&issuers, issuer) {
        panic_with_error!(e, ContractError::IssuerAlreadyAuthorized)
    }
    issuers.push_front(issuer.clone());
    storage::write_vault_issuers(e, owner, &issuers);
    storage::remove_denied_issuer(e, owner, issuer);
}

/// Replace full issuer list for vault. Duplicates are silently removed.
pub fn authorize_issuers(e: &Env, owner: &Address, issuers: &Vec<Address>) {
    let mut deduped: Vec<Address> = Vec::new(e);
    for issuer in issuers.iter() {
        if !deduped.contains(issuer.clone()) {
            deduped.push_back(issuer);
        }
    }
    storage::write_vault_issuers(e, owner, &deduped);
}

/// Remove issuer from vault and add to denied list so auto-authorization won't re-add it.
/// All duplicate occurrences are removed. Panics if issuer was not present.
pub fn revoke_issuer(e: &Env, owner: &Address, issuer: &Address) {
    let issuers = storage::read_vault_issuers(e, owner);
    let original_len = issuers.len();
    let mut filtered: Vec<Address> = Vec::new(e);
    for addr in issuers.iter() {
        if &addr != issuer {
            filtered.push_back(addr);
        }
    }
    if filtered.len() == original_len {
        panic_with_error!(e, ContractError::IssuerNotAuthorized)
    }
    storage::write_vault_issuers(e, owner, &filtered);
    storage::add_denied_issuer(e, owner, issuer);
}

/// Check if issuer is in the list.
pub fn is_authorized(issuers: &Vec<Address>, issuer: &Address) -> bool {
    issuers.contains(issuer.clone())
}
