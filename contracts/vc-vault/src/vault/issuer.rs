//! Issuer list management: add, remove, replace authorized issuers per vault.

use crate::error::ContractError;
use crate::storage;
use soroban_sdk::{panic_with_error, Address, Env, Vec};

/// Adds an issuer to the vault's authorized list; panics if already present.
///
/// Enforces `MAX_ISSUERS_LIST` here rather than at the entrypoint so the cap
/// covers every growth path:
///   - direct `authorize_issuer(owner, issuer)` calls
///   - auto-authorization triggered by `issue` / `batch_issue` /
///     `issue_linked` via `ensure_issuer_authorized`
///
/// `authorize_issuers(list)` is checked at its entrypoint instead because it
/// fully replaces the stored list (the cap on the new list is what matters).
pub fn authorize_issuer(e: &Env, owner: &Address, issuer: &Address) {
    let mut issuers: Vec<Address> = storage::read_vault_issuers(e, owner);
    if is_authorized(&issuers, issuer) {
        panic_with_error!(e, ContractError::IssuerAlreadyAuthorized)
    }
    if issuers.len() >= storage::MAX_ISSUERS_LIST {
        panic_with_error!(e, ContractError::IssuerListTooLong)
    }
    issuers.push_front(issuer.clone());
    storage::write_vault_issuers(e, owner, &issuers);
    storage::remove_denied_issuer(e, owner, issuer);
}

/// Replaces the vault's full issuer list, silently dropping any duplicates.
pub fn authorize_issuers(e: &Env, owner: &Address, issuers: &Vec<Address>) {
    let mut deduped: Vec<Address> = Vec::new(e);
    for issuer in issuers.iter() {
        if !deduped.contains(issuer.clone()) {
            deduped.push_back(issuer);
        }
    }
    storage::write_vault_issuers(e, owner, &deduped);
}

/// Removes an issuer from the vault and adds them to the deny list; panics if not present.
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

/// Returns true if the issuer is present in the authorized list.
pub fn is_authorized(issuers: &Vec<Address>, issuer: &Address) -> bool {
    issuers.contains(issuer.clone())
}
