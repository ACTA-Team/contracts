//! Public contract interface. All exported functions are defined here.

use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::types::{VCStatus, VerifiableCredential};
use crate::storage::FeeConfig;

#[allow(dead_code)]
pub trait VcVaultTrait {
    // --- Admin ---
    fn nominate_admin(e: Env, new_admin: Address);
    fn accept_contract_admin(e: Env);
    fn set_fee_enabled(e: Env, enabled: bool);
    fn set_fee_config(e: Env, token_contract: Address, fee_dest: Address, fee_amount: i128);
    fn set_fee_admin(e: Env, fee_amount: i128);
    fn set_fee_standard(e: Env, fee_amount: i128);
    fn set_fee_early(e: Env, fee_amount: i128);
    fn set_fee_custom(e: Env, issuer: Address, fee_amount: i128);
    fn get_fee_admin(e: Env) -> i128;
    fn get_fee_standard(e: Env) -> i128;
    fn get_fee_early(e: Env) -> i128;
    fn get_fee_custom(e: Env, issuer: Address) -> i128;
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>);
    fn version(e: Env) -> String;
    fn fee_config(e: Env) -> FeeConfig;

    // --- Vault management ---
    fn set_vault_admin(e: Env, new_admin: Address);
    fn authorize_issuers(e: Env, issuers: Vec<Address>);
    fn authorize_issuer(e: Env, issuer_addr: Address);
    fn revoke_issuer(e: Env, issuer_addr: Address);
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
        fee_override: i128,
    ) -> String;
    fn batch_issue(
        e: Env,
        issuer_addr: Address,
        vault_contract: Address,
        issuer_did: String,
        fee_override: i128,
        vcs: Vec<(String, String)>,
    ) -> Vec<String>;
    fn revoke(e: Env, vc_id: String, date: String);

    // --- Issuer queries ---
    fn list_authorized_issuers(e: Env, offset: u32, limit: u32) -> Vec<Address>;
    fn list_denied_issuers(e: Env, offset: u32, limit: u32) -> Vec<Address>;
    fn authorized_issuer_count(e: Env) -> u32;
    fn denied_issuer_count(e: Env) -> u32;
}
