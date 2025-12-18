use crate::storage;
use soroban_sdk::{contracttype, Address, Env, String};

/// Verifiable Credential stored in a vault.
///
/// `data` is expected to be **ciphertext** (encrypted off-chain) or a safe reference.
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

pub fn store_vc(
    e: &Env,
    owner: &Address,
    id: String,
    data: String,
    issuance_contract: Address,
    issuer_did: String,
) {
    let new_vc: VerifiableCredential = VerifiableCredential {
        id: id.clone(),
        data,
        issuance_contract,
        issuer_did,
    };

    storage::write_vault_vc(e, owner, &id, &new_vc);
    storage::append_vault_vc_id(e, owner, &id);
}
