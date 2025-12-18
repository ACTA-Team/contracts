use crate::error::ContractError;
use crate::storage;
use soroban_sdk::{panic_with_error, Address, Env, Vec};

pub fn authorize_issuer(e: &Env, owner: &Address, issuer: &Address) {
    let mut issuers: Vec<Address> = storage::read_vault_issuers(e, owner);

    if is_authorized(&issuers, issuer) {
        panic_with_error!(e, ContractError::IssuerAlreadyAuthorized)
    }
    issuers.push_front(issuer.clone());

    storage::write_vault_issuers(e, owner, &issuers);
}

pub fn authorize_issuers(e: &Env, owner: &Address, issuers: &Vec<Address>) {
    storage::write_vault_issuers(e, owner, issuers);
}

pub fn revoke_issuer(e: &Env, owner: &Address, issuer: &Address) {
    let mut issuers = storage::read_vault_issuers(e, owner);

    if let Some(issuer_index) = issuers.first_index_of(issuer) {
        issuers.remove(issuer_index);
    } else {
        panic_with_error!(e, ContractError::IssuerNotAuthorized)
    }

    storage::write_vault_issuers(e, owner, &issuers);
}

pub fn is_authorized(issuers: &Vec<Address>, issuer: &Address) -> bool {
    issuers.contains(issuer.clone())
}
