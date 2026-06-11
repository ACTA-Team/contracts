//! Guard functions for auth and input validation. All `require_*` helpers
//! panic on violation, matching Soroban's own `.require_auth()` idiom.

use crate::error::ContractError;
use crate::storage;
use soroban_sdk::{panic_with_error, Address, Env};

// --- Auth guards ---

pub fn require_contract_admin(e: &Env) -> Address {
    if !storage::has_contract_admin(e) {
        panic_with_error!(e, ContractError::NotInitialized)
    }
    let admin = storage::read_contract_admin(e);
    admin.require_auth();
    admin
}

pub fn require_vault_initialized(e: &Env) {
    if !storage::has_vault_admin(e) {
        panic_with_error!(e, ContractError::VaultNotInitialized)
    }
}

pub fn require_vault_admin(e: &Env) {
    require_vault_initialized(e);
    let admin = storage::read_vault_admin(e);
    admin.require_auth();
}

pub fn require_vault_active(e: &Env) {
    require_vault_initialized(e);
    if storage::read_vault_revoked(e) {
        panic_with_error!(e, ContractError::VaultRevoked)
    }
}

// --- Input length guards ---

pub fn require_vc_id_len(e: &Env, vc_id: &soroban_sdk::String) {
    if vc_id.len() > storage::MAX_VC_ID_LEN {
        panic_with_error!(e, ContractError::InputTooLong);
    }
}

pub fn require_vc_data_len(e: &Env, vc_data: &soroban_sdk::String) {
    if vc_data.len() > storage::MAX_VC_DATA_LEN {
        panic_with_error!(e, ContractError::InputTooLong);
    }
}

pub fn require_did_uri_len(e: &Env, did_uri: &soroban_sdk::String) {
    if did_uri.len() > storage::MAX_DID_URI_LEN {
        panic_with_error!(e, ContractError::InputTooLong);
    }
}

pub fn require_issuer_did_len(e: &Env, issuer_did: &soroban_sdk::String) {
    if issuer_did.len() > storage::MAX_ISSUER_DID_LEN {
        panic_with_error!(e, ContractError::InputTooLong);
    }
}

pub fn require_date_len(e: &Env, date: &soroban_sdk::String) {
    if date.len() > storage::MAX_DATE_LEN {
        panic_with_error!(e, ContractError::InputTooLong);
    }
}
