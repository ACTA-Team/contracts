use crate::error::ContractError;
use crate::storage;
use soroban_sdk::{contracttype, panic_with_error, Env, String};

/// Status registry entry for a VC ID.
#[derive(PartialEq)]
#[contracttype]
pub enum VCStatus {
    /// VC exists and is currently valid.
    Valid,

    /// VC does not exist in the registry.
    Invalid,

    /// VC was revoked at the given ISO-8601 date string.
    Revoked(String),
}

pub fn revoke_vc(e: &Env, vc_id: String, date: String) {
    let vc_status = storage::read_vc_status(e, &vc_id);

    if vc_status != VCStatus::Valid {
        panic_with_error!(e, ContractError::VCAlreadyRevoked)
    }
    storage::write_vc_status(e, &vc_id, &VCStatus::Revoked(date))
}
