//! Credential lifecycle: store, issue with fee, revoke.

use crate::error::ContractError;
use crate::types::{VCStatus, VerifiableCredential};
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

pub fn store_vc_with_fee(
    e: &Env,
    vc_id: String,
    vc_data: String,
    issuer_addr: &Address,
    issuer_did: String,
    issuance_contract: Address,
    fee_override: i128,
) {
    if storage::read_fee_enabled(e) {
        let fee_token = storage::read_fee_token_contract(e);
        let fee_dest = storage::read_fee_dest(e);
        if fee_override > 0 {
            e.invoke_contract::<()>(
                &fee_token,
                &symbol_short!("transfer"),
                (issuer_addr.clone(), fee_dest, fee_override).into_val(e),
            );
        }
    }
    store_vc(e, vc_id, vc_data, issuance_contract, issuer_did);
}

pub fn revoke_vc(e: &Env, vc_id: String, date: String) {
    let vc_status = storage::read_vc_status(e, &vc_id);
    if vc_status != VCStatus::Valid {
        panic_with_error!(e, ContractError::VCAlreadyRevoked)
    }
    storage::write_vc_status(e, &vc_id, &VCStatus::Revoked(date))
}
