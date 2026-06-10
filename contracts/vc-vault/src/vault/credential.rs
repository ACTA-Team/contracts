//! Credential lifecycle: store, issue with fee, revoke.

use crate::error::ContractError;
use crate::types::{FeeQuote, VCStatus, VerifiableCredential};
use crate::storage;
use soroban_sdk::{panic_with_error, symbol_short, Address, Env, IntoVal, String};

pub fn store_vc(
    e: &Env,
    id: String,
    data: String,
    issuance_contract: Address,
    issuer_did: String,
) {
    let new_vc = VerifiableCredential {
        id: id.clone(),
        data,
        issuance_contract,
        issuer_did,
    };
    storage::write_vault_vc(e, &id, &new_vc);
    storage::append_vc_to_index(e, &id);
}

/// Reads the central fee from the factory and, if enabled and > 0, transfers
/// `amount` from `issuer` to the configured destination. Returns the amount charged.
pub fn charge_fee(e: &Env, factory: &Address, issuer: &Address) -> i128 {
    let q: FeeQuote = e.invoke_contract(
        factory,
        &symbol_short!("quote_fee"),
        (issuer.clone(),).into_val(e),
    );
    if q.enabled && q.amount > 0 {
        let token = q.token.unwrap();
        let dest = q.dest.unwrap();
        e.invoke_contract::<()>(
            &token,
            &symbol_short!("transfer"),
            (issuer.clone(), dest, q.amount).into_val(e),
        );
    }
    q.amount
}

pub fn revoke_vc(e: &Env, vc_id: String, date: String) {
    let vc_status = storage::read_vc_status(e, &vc_id);
    if vc_status != VCStatus::Valid {
        panic_with_error!(e, ContractError::VCAlreadyRevoked)
    }
    storage::write_vc_status(e, &vc_id, &VCStatus::Revoked(date))
}
