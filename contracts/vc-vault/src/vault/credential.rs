//! Store VC payload in vault and update index.

use crate::model::VerifiableCredential;
use crate::storage;
use soroban_sdk::{Address, Env, String};

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
