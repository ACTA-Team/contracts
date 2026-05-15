//! Guard functions for auth and input validation. All `require_*` helpers
//! panic on violation, matching Soroban's own `.require_auth()` idiom.

use crate::error::ContractError;
use crate::storage;
use crate::vault;
use soroban_sdk::{panic_with_error, Address, Env, Vec};

// --- Auth guards ---

/// Ensure contract admin exists and has signed. Returns admin address.
pub fn require_contract_admin(e: &Env) -> Address {
    if !storage::has_contract_admin(e) {
        panic_with_error!(e, ContractError::NotInitialized)
    }
    let admin = storage::read_contract_admin(e);
    admin.require_auth();
    admin
}

/// Ensure vault exists for owner.
pub fn require_vault_initialized(e: &Env, owner: &Address) {
    if !storage::has_vault_admin(e, owner) {
        panic_with_error!(e, ContractError::VaultNotInitialized)
    }
}

/// Ensure vault exists and caller is vault admin (has signed).
pub fn require_vault_admin(e: &Env, owner: &Address) {
    require_vault_initialized(e, owner);
    let admin = storage::read_vault_admin(e, owner);
    admin.require_auth();
}

/// Ensure vault exists and is not revoked.
pub fn require_vault_active(e: &Env, owner: &Address) {
    require_vault_initialized(e, owner);
    if storage::read_vault_revoked(e, owner) {
        panic_with_error!(e, ContractError::VaultRevoked)
    }
}

/// Ensure issuer is in vault's authorized index. No signature check.
pub fn require_issuer_authorized(e: &Env, owner: &Address, issuer_addr: &Address) {
    require_vault_initialized(e, owner);
    if !vault::is_authorized(e, owner, issuer_addr) {
        panic_with_error!(e, ContractError::IssuerNotAuthorized)
    }
}

/// Auto-authorizes issuer if not in the authorized index; panics if in the denied index.
pub fn ensure_issuer_authorized(e: &Env, owner: &Address, issuer_addr: &Address) {
    require_vault_initialized(e, owner);
    if !vault::is_authorized(e, owner, issuer_addr) {
        if storage::denied_issuer_index_contains(e, owner, issuer_addr) {
            panic_with_error!(e, ContractError::IssuerNotAuthorized)
        }
        storage::append_issuer_to_index(e, owner, issuer_addr);
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

pub fn require_issuers_list_len(e: &Env, issuers: &Vec<Address>) {
    if issuers.len() > storage::MAX_ISSUERS_LIST {
        panic_with_error!(e, ContractError::IssuerListTooLong);
    }
}

pub fn require_fee_amount(e: &Env, amount: i128) {
    if amount < 0 {
        panic_with_error!(e, ContractError::InvalidFeeAmount);
    }
    if amount > storage::MAX_FEE_AMOUNT {
        panic_with_error!(e, ContractError::FeeOutOfBounds);
    }
}
