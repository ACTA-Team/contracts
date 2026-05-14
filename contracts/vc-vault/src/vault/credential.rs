//! Credential lifecycle: store, issue with fee, revoke.

use crate::error::ContractError;
use crate::types::{VCStatus, VerifiableCredential};
use crate::storage;
use soroban_sdk::{panic_with_error, symbol_short, Address, Env, IntoVal, String};

/// Write VC to vault and append ID to the O(1) index.
///
/// `append_vc_to_index` panics with `VaultFull` once the vault hits
/// `MAX_VCS_PER_VAULT`, so the VC payload is intentionally written first:
/// callers that catch the panic will not leave a payload without an index
/// entry (the transaction reverts atomically).
pub fn store_vc(
    e: &Env,
    owner: &Address,
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
    storage::write_vault_vc(e, owner, &id, &new_vc);
    storage::append_vc_to_index(e, owner, &id);
}

/// Store VC in vault and charge fee if enabled.
pub fn store_vc_with_fee(
    e: &Env,
    owner: &Address,
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
    store_vc(e, owner, vc_id, vc_data, issuance_contract, issuer_did);
}

/// Set VC status to Revoked. Panics if not Valid.
pub fn revoke_vc(e: &Env, owner: &Address, vc_id: String, date: String) {
    let vc_status = storage::read_vc_status(e, owner, &vc_id);
    if vc_status != VCStatus::Valid {
        panic_with_error!(e, ContractError::VCAlreadyRevoked)
    }
    storage::write_vc_status(e, owner, &vc_id, &VCStatus::Revoked(date))
}

/// Transfer a VC from one vault to another. Tombstones the source status
/// entry so vc_id stays unique in the source vault index.
pub fn push_vc(e: &Env, from_owner: &Address, to_owner: &Address, vc_id: &String) {
    let vc_opt = storage::read_vault_vc(e, from_owner, vc_id);
    if vc_opt.is_none() {
        panic_with_error!(e, ContractError::VCNotFound);
    }
    if storage::read_vc_status(e, from_owner, vc_id) != VCStatus::Valid {
        panic_with_error!(e, ContractError::VCAlreadyRevoked);
    }
    if storage::read_vault_vc(e, to_owner, vc_id).is_some()
        || storage::read_vc_status(e, to_owner, vc_id) != VCStatus::Invalid
    {
        panic_with_error!(e, ContractError::VCAlreadyExists);
    }
    let vc = vc_opt.unwrap();
    let parent = storage::read_vc_parent(e, from_owner, vc_id);

    storage::remove_vault_vc(e, from_owner, vc_id);
    storage::remove_vc_from_index(e, from_owner, vc_id);
    if parent.is_some() {
        storage::remove_vc_parent(e, from_owner, vc_id);
    }
    // VCStatus(from_owner, vc_id) stays Valid as tombstone — preserves
    // vc_id uniqueness in the source vault.

    storage::write_vault_vc(e, to_owner, vc_id, &vc);
    storage::append_vc_to_index(e, to_owner, vc_id);
    storage::write_vc_status(e, to_owner, vc_id, &VCStatus::Valid);
    if let Some((parent_owner, parent_vc_id)) = parent {
        storage::write_vc_parent(e, to_owner, vc_id, &parent_owner, &parent_vc_id);
    }

    storage::extend_vault_ttl(e, from_owner);
    storage::extend_vault_ttl(e, to_owner);
    storage::extend_vc_status_ttl(e, from_owner, vc_id);
    storage::extend_vc_ttl(e, to_owner, vc_id);
    crate::events::vc_pushed(e, from_owner, to_owner, vc_id);
}
