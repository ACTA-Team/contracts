//! Verifiable Credential type stored in vaults.

use soroban_sdk::{contracttype, Address, String};

/// VC payload stored in a vault. `data` should be ciphertext only (never plaintext PII).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiableCredential {
    /// Application-level VC identifier.
    pub id: String,

    /// VC payload (ciphertext or reference).
    pub data: String,

    /// Issuance contract that can verify/revoke the VC status.
    pub issuance_contract: Address,

    /// Issuer DID (metadata for wallets/UX).
    pub issuer_did: String,
}
