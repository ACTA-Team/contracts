//! Issuer list management: add, remove, replace authorized issuers per vault.

use crate::error::ContractError;
use crate::storage;
use soroban_sdk::{panic_with_error, Address, Env, Vec};

pub fn is_authorized(e: &Env, owner: &Address, issuer: &Address) -> bool {
    storage::issuer_index_contains(e, owner, issuer)
}

pub fn authorize_issuer(e: &Env, owner: &Address, issuer: &Address) {
    if storage::issuer_index_contains(e, owner, issuer) {
        panic_with_error!(e, ContractError::IssuerAlreadyAuthorized);
    }
    storage::append_issuer_to_index(e, owner, issuer);
    storage::remove_denied_issuer_from_index(e, owner, issuer);
}

/// Replaces the full authorized issuer index, silently dropping duplicates.
pub fn authorize_issuers(e: &Env, owner: &Address, issuers: &Vec<Address>) {
    storage::clear_issuer_index(e, owner);
    let mut seen: Vec<Address> = Vec::new(e);
    for issuer in issuers.iter() {
        if !seen.contains(issuer.clone()) {
            seen.push_back(issuer.clone());
            storage::append_issuer_to_index(e, owner, &issuer);
        }
    }
}

pub fn revoke_issuer(e: &Env, owner: &Address, issuer: &Address) {
    if !storage::issuer_index_contains(e, owner, issuer) {
        panic_with_error!(e, ContractError::IssuerNotAuthorized);
    }
    storage::remove_issuer_from_index(e, owner, issuer);
    storage::append_denied_issuer_to_index(e, owner, issuer);
}
