//! Issuance status registry: revoke VC by ID.

use crate::error::ContractError;
use crate::model::VCStatus;
use crate::storage;
use soroban_sdk::{panic_with_error, Address, Env, String};

/// Set VC status to Revoked. Panics if not Valid.
/// A-17: owner param added so status is read/written from the namespaced key (owner, vc_id).
pub fn revoke_vc(e: &Env, owner: &Address, vc_id: String, date: String) {
    let vc_status = storage::read_vc_status(e, owner, &vc_id);
    if vc_status != VCStatus::Valid {
        panic_with_error!(e, ContractError::VCAlreadyRevoked)
    }
    storage::write_vc_status(e, owner, &vc_id, &VCStatus::Revoked(date))
}
