//! Unit tests for VC Vault contract.

use crate::contract::{VcVaultContract, VcVaultContractClient};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

/// Create env, admin, issuer, contract, and client for tests.
fn setup() -> (Env, Address, Address, Address, VcVaultContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let contract_id = env.register_contract(None, VcVaultContract);
    let client = VcVaultContractClient::new(&env, &contract_id);
    (env, admin, issuer, contract_id, client)
}

#[test]
fn test_version() {
    let (_env, _admin, _issuer, _contract_id, client) = setup();
    let v = client.version();
    assert!(v.len() > 0);
}

#[test]
fn test_initialize_and_create_vault() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    let default_did = String::from_str(&env, "did:acta:default");
    client.initialize(&admin, &default_did);
    let owner = Address::generate(&env);
    let did_uri = String::from_str(&env, "did:pkh:stellar:testnet:OWNER");
    client.create_vault(&owner, &did_uri);
}

#[test]
#[should_panic]
fn test_initialize_twice_panics() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    let default_did = String::from_str(&env, "did:acta:default");
    client.initialize(&admin, &default_did);
    client.initialize(&admin, &default_did);
}

#[test]
fn test_set_contract_admin() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let new_admin = Address::generate(&env);
    client.set_contract_admin(&new_admin);
    let another_admin = Address::generate(&env);
    client.set_contract_admin(&another_admin);
}

#[test]
fn test_fee_config_default() {
    let (_env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&_env, "did:acta:default"));
    let config = client.fee_config();
    assert!(!config.enabled);
    assert!(!config.configured);
    assert!(config.token_contract.is_none());
    assert!(config.fee_dest.is_none());
    assert!(config.fee_amount.is_none());
}

#[test]
fn test_set_fee_config() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let token = Address::generate(&env);
    let fee_dest = Address::generate(&env);
    client.set_fee_config(&token, &fee_dest, &1_000_000_i128);
    let config = client.fee_config();
    assert!(config.configured);
    assert_eq!(config.token_contract, Some(token));
    assert_eq!(config.fee_dest, Some(fee_dest));
    assert_eq!(config.fee_amount, Some(1_000_000));
}

#[test]
fn test_set_fee_enabled() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    client.set_fee_enabled(&true);
    assert!(client.fee_config().enabled);
    client.set_fee_enabled(&false);
    assert!(!client.fee_config().enabled);
}

#[test]
fn test_set_and_get_fee_admin() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    assert_eq!(client.get_fee_admin(), 0);
    client.set_fee_admin(&100_i128);
    assert_eq!(client.get_fee_admin(), 100);
}

#[test]
fn test_set_and_get_fee_standard() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    assert_eq!(client.get_fee_standard(), 1_000_000);
    client.set_fee_standard(&2_000_000_i128);
    assert_eq!(client.get_fee_standard(), 2_000_000);
}

#[test]
fn test_set_and_get_fee_early() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    assert_eq!(client.get_fee_early(), 400_000);
    client.set_fee_early(&500_000_i128);
    assert_eq!(client.get_fee_early(), 500_000);
}

#[test]
fn test_set_and_get_fee_custom() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    client.set_fee_custom(&issuer, &300_000_i128);
    assert_eq!(client.get_fee_custom(&issuer), 300_000);
}

#[test]
#[should_panic]
fn test_create_vault_twice_panics() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    let did_uri = String::from_str(&env, "did:pkh:stellar:testnet:OWNER");
    client.create_vault(&owner, &did_uri);
    client.create_vault(&owner, &did_uri);
}

#[test]
fn test_set_vault_admin() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    let new_admin = Address::generate(&env);
    client.set_vault_admin(&owner, &new_admin);
    let issuer = Address::generate(&env);
    client.authorize_issuer(&owner, &issuer);
}

#[test]
fn test_authorize_issuer() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.authorize_issuer(&owner, &issuer);
}

#[test]
fn test_authorize_issuers_bulk() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    let issuer2 = Address::generate(&env);
    let issuers = vec![&env, issuer.clone(), issuer2.clone()];
    client.authorize_issuers(&owner, &issuers);
}

#[test]
fn test_revoke_issuer() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.authorize_issuer(&owner, &issuer);
    client.revoke_issuer(&owner, &issuer);
}

#[test]
#[should_panic]
fn test_issue_after_revoke_issuer_panics() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.authorize_issuer(&owner, &issuer);
    client.revoke_issuer(&owner, &issuer);
    let vc_id = String::from_str(&env, "vc-1");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
}

#[test]
fn test_revoke_vault() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.revoke_vault(&owner);
}

#[test]
#[should_panic]
fn test_issue_after_revoke_vault_panics() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.authorize_issuer(&owner, &issuer);
    client.revoke_vault(&owner);
    let vc_id = String::from_str(&env, "vc-1");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
}

#[test]
fn test_list_vc_ids_empty() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    assert_eq!(client.list_vc_ids(&owner).len(), 0);
}

#[test]
fn test_get_vc_none_for_missing() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    let vc_id = String::from_str(&env, "nonexistent");
    assert!(client.get_vc(&owner, &vc_id).is_none());
}

#[test]
fn test_verify_vc_invalid_when_not_in_vault() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    let vc_id = String::from_str(&env, "nonexistent");
    let m = client.verify_vc(&owner, &vc_id);
    assert_eq!(m.get(String::from_str(&env, "status")).unwrap(), String::from_str(&env, "invalid"));
}

#[test]
fn test_vault_authorize_and_store_and_list_and_get() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.authorize_issuer(&owner, &issuer);
    let vc_id = String::from_str(&env, "vc-1");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
    assert_eq!(client.list_vc_ids(&owner).len(), 1);
    assert_eq!(client.get_vc(&owner, &vc_id).unwrap().data, vc_data);
}

#[test]
fn test_issue_verify_revoke_flow_local_vault() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.authorize_issuer(&owner, &issuer);
    let vc_id = String::from_str(&env, "vc-123");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
    assert_eq!(client.verify_vc(&owner, &vc_id).get(String::from_str(&env, "status")).unwrap(), String::from_str(&env, "valid"));
    client.revoke(&vc_id, &String::from_str(&env, "2025-12-18T00:00:00Z"));
    assert_eq!(client.verify_vc(&owner, &vc_id).get(String::from_str(&env, "status")).unwrap(), String::from_str(&env, "revoked"));
}

#[test]
fn test_push_moves_between_vaults() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let from_owner = Address::generate(&env);
    let to_owner = Address::generate(&env);
    client.create_vault(&from_owner, &String::from_str(&env, "did:pkh:stellar:testnet:FROM"));
    client.create_vault(&to_owner, &String::from_str(&env, "did:pkh:stellar:testnet:TO"));
    client.authorize_issuer(&from_owner, &issuer);
    let vc_id = String::from_str(&env, "vc-push");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&from_owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
    client.push(&from_owner, &to_owner, &vc_id, &issuer);
    assert!(client.get_vc(&from_owner, &vc_id).is_none());
    assert!(client.get_vc(&to_owner, &vc_id).is_some());
}

#[test]
fn test_issue_returns_vc_id() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.authorize_issuer(&owner, &issuer);
    let vc_id = String::from_str(&env, "vc-return");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    let returned = client.issue(&owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
    assert_eq!(returned, vc_id);
}

#[test]
fn test_issue_with_fee_override() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.authorize_issuer(&owner, &issuer);
    let vc_id = String::from_str(&env, "vc-fee");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
    assert!(client.get_vc(&owner, &vc_id).is_some());
}

#[test]
#[should_panic]
fn test_issue_invalid_vault_contract_panics() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.authorize_issuer(&owner, &issuer);
    let wrong_contract = Address::generate(&env);
    let vc_id = String::from_str(&env, "vc-1");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&owner, &vc_id, &vc_data, &wrong_contract, &issuer, &issuer_did, &0_i128);
}

#[test]
#[should_panic]
fn test_revoke_nonexistent_vc_panics() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let vc_id = String::from_str(&env, "nonexistent");
    let date = String::from_str(&env, "2025-12-18T00:00:00Z");
    client.revoke(&vc_id, &date);
}

#[test]
#[should_panic]
fn test_push_nonexistent_vc_panics() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let from_owner = Address::generate(&env);
    let to_owner = Address::generate(&env);
    client.create_vault(&from_owner, &String::from_str(&env, "did:pkh:stellar:testnet:FROM"));
    client.create_vault(&to_owner, &String::from_str(&env, "did:pkh:stellar:testnet:TO"));
    client.authorize_issuer(&from_owner, &issuer);
    let vc_id = String::from_str(&env, "nonexistent");
    client.push(&from_owner, &to_owner, &vc_id, &issuer);
}

#[test]
#[should_panic]
fn test_migrate_none_without_legacy_panics() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    client.migrate(&None);
}

#[test]
#[should_panic]
fn test_migrate_some_without_legacy_vault_panics() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.migrate(&Some(owner));
}
