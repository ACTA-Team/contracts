//! VC status in the issuance registry.

use soroban_sdk::{contracttype, String};

/// Status of a VC in the issuance registry.
#[derive(Debug, PartialEq)]
#[contracttype]
pub enum VCStatus {
    /// VC exists and is currently valid.
    Valid,

    /// VC does not exist in the registry.
    Invalid,

    /// VC was revoked at the given ISO-8601 date string.
    Revoked(String),
}
