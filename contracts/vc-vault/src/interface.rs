//! Public contract interface. All exported functions are defined here.

use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::types::{VCStatus, VerifiableCredential};

#[allow(dead_code)]
pub trait VcVaultTrait {
    // --- Admin ---
    fn nominate_admin(e: Env, new_admin: Address);
    fn accept_contract_admin(e: Env);
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>);
    fn version(e: Env) -> String;

    // --- Vault management ---
    fn set_vault_admin(e: Env, new_admin: Address);
    fn set_vault_did(e: Env, did_uri: String);
    fn vault_did(e: Env) -> Option<String>;
    fn vault_owner(e: Env) -> Address;
    fn deny_issuer(e: Env, issuer_addr: Address);
    fn allow_issuer(e: Env, issuer_addr: Address);
    fn revoke_vault(e: Env);

    // --- Credential queries ---
    fn list_vc_ids(e: Env, offset: u32, limit: u32) -> Vec<String>;
    fn vc_count(e: Env) -> u32;
    fn get_vc(e: Env, vc_id: String) -> Option<VerifiableCredential>;
    fn verify_vc(e: Env, vc_id: String) -> VCStatus;

    // --- Issuance ---
    fn issue(
        e: Env,
        vc_id: String,
        vc_data: String,
        vault_contract: Address,
        issuer_addr: Address,
        issuer_did: String,
    ) -> String;
    fn batch_issue(
        e: Env,
        issuer_addr: Address,
        vault_contract: Address,
        issuer_did: String,
        vcs: Vec<(String, String)>,
    ) -> Vec<String>;
    fn revoke(e: Env, vc_id: String, date: String);
    fn push(e: Env, vc_id: String, dest_vault: Address);
    fn receive_push(
        e: Env,
        source_vault: Address,
        source_owner: Address,
        vc_id: String,
        vc_data: String,
        issuer_did: String,
    );

    // --- Issuer queries ---
    fn list_denied_issuers(e: Env, offset: u32, limit: u32) -> Vec<Address>;
    fn denied_issuer_count(e: Env) -> u32;
}
