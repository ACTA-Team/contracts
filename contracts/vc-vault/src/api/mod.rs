//! Public contract interface. All exported functions are defined here.

use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::model::{VCStatus, VerifiableCredential};
use crate::storage::FeeConfig;

/// Trait defining all public contract entrypoints.
#[allow(dead_code)]
pub trait VcVaultTrait {
    fn initialize(e: Env, contract_admin: Address);
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
    fn create_vault(e: Env, owner: Address, did_uri: String);
    fn set_vault_admin(e: Env, owner: Address, new_admin: Address);
    fn authorize_issuers(e: Env, owner: Address, issuers: Vec<Address>);
    fn authorize_issuer(e: Env, owner: Address, issuer: Address);
    fn revoke_issuer(e: Env, owner: Address, issuer: Address);
    fn revoke_vault(e: Env, owner: Address);
    fn list_vc_ids(e: Env, owner: Address, offset: u32, limit: u32) -> Vec<String>;
    fn vc_count(e: Env, owner: Address) -> u32;
    fn get_vc(e: Env, owner: Address, vc_id: String) -> Option<VerifiableCredential>;
    fn verify_vc(e: Env, owner: Address, vc_id: String) -> VCStatus;
    fn push(e: Env, from_owner: Address, to_owner: Address, vc_id: String, issuer: Address);
    fn issue(
        e: Env,
        owner: Address,
        vc_id: String,
        vc_data: String,
        vault_contract: Address,
        issuer: Address,
        issuer_did: String,
        fee_override: i128,
    ) -> String;
    fn batch_issue(
        e: Env,
        issuer: Address,
        owner: Address,
        vault_contract: Address,
        issuer_did: String,
        fee_override: i128,
        vcs: Vec<(String, String)>,
    ) -> Vec<String>;
    fn revoke(e: Env, owner: Address, vc_id: String, date: String);
    fn issue_linked(
        e: Env,
        issuer: Address,
        owner: Address,
        vc_id: String,
        data: String,
        issuance_contract: Address,
        issuer_did: String,
        parent_owner: Address,
        parent_vc_id: String,
    );
    fn get_vc_parent(e: Env, owner: Address, vc_id: String) -> Option<(Address, String)>;
    fn list_authorized_issuers(e: Env, owner: Address, offset: u32, limit: u32) -> Vec<Address>;
    fn list_denied_issuers(e: Env, owner: Address, offset: u32, limit: u32) -> Vec<Address>;
    fn authorized_issuer_count(e: Env, owner: Address) -> u32;
    fn denied_issuer_count(e: Env, owner: Address) -> u32;
    fn create_sponsored_vault(e: Env, sponsor: Address, owner: Address, did_uri: String);
    fn set_sponsored_vault_open_to_all(e: Env, open: bool);
    fn get_sponsored_vault_open_to_all(e: Env) -> bool;
    fn add_sponsored_vault_sponsor(e: Env, sponsor: Address);
    fn remove_sponsored_vault_sponsor(e: Env, sponsor: Address);
}
