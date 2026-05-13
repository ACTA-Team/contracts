//! Unit tests for VC Vault contract.

use crate::contract::{VcVaultContract, VcVaultContractClient};
use crate::model::VCStatus;
use soroban_sdk::{
    testutils::{Address as _, Events, MockAuth, MockAuthInvoke},
    vec, Address, Env, IntoVal, String,
};

/// Create env, admin, issuer, contract, and client for tests.
fn setup() -> (Env, Address, Address, Address, VcVaultContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let contract_id = env.register(VcVaultContract, ());
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
    client.initialize(&admin);
    let owner = Address::generate(&env);
    let did_uri = String::from_str(&env, "did:pkh:stellar:testnet:OWNER");
    client.create_vault(&owner, &did_uri);
}

#[test]
#[should_panic]
fn test_initialize_twice_panics() {
    let (_env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
fn test_nominate_and_accept_admin() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let new_admin = Address::generate(&env);
    client.nominate_admin(&new_admin);
    client.accept_contract_admin();
    // New admin can now nominate a third admin.
    let another_admin = Address::generate(&env);
    client.nominate_admin(&another_admin);
    client.accept_contract_admin();
}

#[test]
fn test_fee_config_default() {
    let (_env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
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
    client.initialize(&admin);
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
    let (_env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    client.set_fee_enabled(&true);
    assert!(client.fee_config().enabled);
    client.set_fee_enabled(&false);
    assert!(!client.fee_config().enabled);
}

#[test]
fn test_set_and_get_fee_admin() {
    let (_env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    assert_eq!(client.get_fee_admin(), 0);
    client.set_fee_admin(&100_i128);
    assert_eq!(client.get_fee_admin(), 100);
}

#[test]
fn test_set_and_get_fee_standard() {
    let (_env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    assert_eq!(client.get_fee_standard(), 1_000_000);
    client.set_fee_standard(&2_000_000_i128);
    assert_eq!(client.get_fee_standard(), 2_000_000);
}

#[test]
fn test_set_and_get_fee_early() {
    let (_env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    assert_eq!(client.get_fee_early(), 400_000);
    client.set_fee_early(&500_000_i128);
    assert_eq!(client.get_fee_early(), 500_000);
}

#[test]
fn test_set_and_get_fee_custom() {
    let (_env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    client.set_fee_custom(&issuer, &300_000_i128);
    assert_eq!(client.get_fee_custom(&issuer), 300_000);
}

#[test]
#[should_panic]
fn test_create_vault_twice_panics() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    let did_uri = String::from_str(&env, "did:pkh:stellar:testnet:OWNER");
    client.create_vault(&owner, &did_uri);
    client.create_vault(&owner, &did_uri);
}

#[test]
fn test_set_vault_admin() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
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
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.authorize_issuer(&owner, &issuer);
}

#[test]
fn test_authorize_issuers_bulk() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    let issuer2 = Address::generate(&env);
    let issuers = vec![&env, issuer.clone(), issuer2.clone()];
    client.authorize_issuers(&owner, &issuers);
}

#[test]
fn test_revoke_issuer() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.authorize_issuer(&owner, &issuer);
    client.revoke_issuer(&owner, &issuer);
}

#[test]
#[should_panic]
fn test_issue_after_revoke_issuer_panics() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
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
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.revoke_vault(&owner);
}

#[test]
#[should_panic]
fn test_issue_after_revoke_vault_panics() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
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
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    assert_eq!(client.list_vc_ids(&owner, &0_u32, &200_u32).len(), 0);
}

#[test]
fn test_get_vc_none_for_missing() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    let vc_id = String::from_str(&env, "nonexistent");
    assert!(client.get_vc(&owner, &vc_id).is_none());
}

#[test]
fn test_verify_vc_invalid_when_not_in_vault() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    let vc_id = String::from_str(&env, "nonexistent");
    assert_eq!(client.verify_vc(&owner, &vc_id), VCStatus::Invalid);
}

#[test]
fn test_vault_authorize_and_store_and_list_and_get() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.authorize_issuer(&owner, &issuer);
    let vc_id = String::from_str(&env, "vc-1");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
    assert_eq!(client.list_vc_ids(&owner, &0_u32, &200_u32).len(), 1);
    assert_eq!(client.get_vc(&owner, &vc_id).unwrap().data, vc_data);
}

#[test]
fn test_issue_verify_revoke_flow_local_vault() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.authorize_issuer(&owner, &issuer);
    let vc_id = String::from_str(&env, "vc-123");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
    assert_eq!(client.verify_vc(&owner, &vc_id), VCStatus::Valid);
    let date = String::from_str(&env, "2025-12-18T00:00:00Z");
    client.revoke(&owner, &vc_id, &date);
    assert_eq!(client.verify_vc(&owner, &vc_id), VCStatus::Revoked(date));
}

#[test]
fn test_push_moves_between_vaults() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
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
#[should_panic]
fn test_issue_after_push_same_vc_id_panics() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
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
    // Re-issuing the same vc_id after push must fail: vc_id is already registered
    // in from_owner's identity space, and now lives in to_owner's vault.
    client.issue(&from_owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
}

#[test]
#[should_panic]
fn test_revoke_after_push_panics() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
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
    // Revoking from the source vault after push must fail: the vc no longer
    // belongs to from_owner's vault.
    let date = String::from_str(&env, "2025-12-18T00:00:00Z");
    client.revoke(&from_owner, &vc_id, &date);
}

#[test]
fn test_verify_vc_valid_after_push_on_destination() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
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
    assert_eq!(client.verify_vc(&to_owner, &vc_id), VCStatus::Valid);
}

#[test]
fn test_revoke_after_push_on_destination_succeeds() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let from_owner = Address::generate(&env);
    let to_owner = Address::generate(&env);
    client.create_vault(&from_owner, &String::from_str(&env, "did:pkh:stellar:testnet:FROM"));
    client.create_vault(&to_owner, &String::from_str(&env, "did:pkh:stellar:testnet:TO"));
    client.authorize_issuer(&from_owner, &issuer);
    let vc_id = String::from_str(&env, "vc-push");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    let date = String::from_str(&env, "2025-12-18T00:00:00Z");
    client.issue(&from_owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
    client.push(&from_owner, &to_owner, &vc_id, &issuer);
    client.revoke(&to_owner, &vc_id, &date);
    assert_eq!(client.verify_vc(&to_owner, &vc_id), VCStatus::Revoked(date));
}

#[test]
#[should_panic]
fn test_push_to_destination_with_existing_vc_id_panics() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let attacker = Address::generate(&env);
    let to_owner = Address::generate(&env);
    client.create_vault(&attacker, &String::from_str(&env, "did:pkh:stellar:testnet:ATTACKER"));
    client.create_vault(&to_owner, &String::from_str(&env, "did:pkh:stellar:testnet:TO"));
    client.authorize_issuer(&attacker, &issuer);
    let vc_id = String::from_str(&env, "vc-shared");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    let date = String::from_str(&env, "2025-12-18T00:00:00Z");
    // to_owner has vc-shared issued and revoked.
    client.issue(&to_owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
    client.revoke(&to_owner, &vc_id, &date);
    // Attacker issues the same vc_id to their own vault and pushes to to_owner.
    // Must fail: to_owner already has a status for this vc_id (Revoked).
    client.issue(&attacker, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
    client.push(&attacker, &to_owner, &vc_id, &issuer);
}

#[test]
#[should_panic]
fn test_push_revoked_vc_panics() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let from_owner = Address::generate(&env);
    let to_owner = Address::generate(&env);
    client.create_vault(&from_owner, &String::from_str(&env, "did:pkh:stellar:testnet:FROM"));
    client.create_vault(&to_owner, &String::from_str(&env, "did:pkh:stellar:testnet:TO"));
    client.authorize_issuer(&from_owner, &issuer);
    let vc_id = String::from_str(&env, "vc-push");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    let date = String::from_str(&env, "2025-12-18T00:00:00Z");
    client.issue(&from_owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
    client.revoke(&from_owner, &vc_id, &date);
    // Pushing a revoked VC must fail: revoked credentials are invalidated.
    client.push(&from_owner, &to_owner, &vc_id, &issuer);
}

#[test]
fn test_issue_returns_vc_id() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
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
    client.initialize(&admin);
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
    client.initialize(&admin);
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
    client.initialize(&admin);
    let owner = Address::generate(&env);
    let vc_id = String::from_str(&env, "nonexistent");
    let date = String::from_str(&env, "2025-12-18T00:00:00Z");
    client.revoke(&owner, &vc_id, &date);
}

#[test]
#[should_panic]
fn test_push_nonexistent_vc_panics() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let from_owner = Address::generate(&env);
    let to_owner = Address::generate(&env);
    client.create_vault(&from_owner, &String::from_str(&env, "did:pkh:stellar:testnet:FROM"));
    client.create_vault(&to_owner, &String::from_str(&env, "did:pkh:stellar:testnet:TO"));
    client.authorize_issuer(&from_owner, &issuer);
    let vc_id = String::from_str(&env, "nonexistent");
    client.push(&from_owner, &to_owner, &vc_id, &issuer);
}

// --- Auto-authorization on issue ---

#[test]
fn test_issue_auto_authorizes_issuer() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    let vc_id = String::from_str(&env, "vc-auto");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
    assert!(client.get_vc(&owner, &vc_id).is_some());
    assert_eq!(client.list_vc_ids(&owner, &0_u32, &200_u32).len(), 1);
}

#[test]
fn test_issue_auto_authorizes_multiple_issuers() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    let issuer2 = Address::generate(&env);
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&owner, &String::from_str(&env, "vc-1"), &String::from_str(&env, "<data1>"), &contract_id, &issuer, &issuer_did, &0_i128);
    client.issue(&owner, &String::from_str(&env, "vc-2"), &String::from_str(&env, "<data2>"), &contract_id, &issuer2, &issuer_did, &0_i128);
    assert_eq!(client.list_vc_ids(&owner, &0_u32, &200_u32).len(), 2);
}

#[test]
fn test_holder_revokes_auto_authorized_issuer() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&owner, &String::from_str(&env, "vc-1"), &String::from_str(&env, "<data>"), &contract_id, &issuer, &issuer_did, &0_i128);
    assert!(client.get_vc(&owner, &String::from_str(&env, "vc-1")).is_some());
    client.revoke_issuer(&owner, &issuer);
}

#[test]
#[should_panic]
fn test_issue_after_holder_revokes_auto_authorized_issuer_panics() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&owner, &String::from_str(&env, "vc-1"), &String::from_str(&env, "<data>"), &contract_id, &issuer, &issuer_did, &0_i128);
    client.revoke_issuer(&owner, &issuer);
    client.issue(&owner, &String::from_str(&env, "vc-2"), &String::from_str(&env, "<data2>"), &contract_id, &issuer, &issuer_did, &0_i128);
}

#[test]
#[should_panic]
fn test_migrate_without_legacy_vault_panics() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    client.migrate(&owner);
}

// --- Sponsored vault tests ---

#[test]
fn test_sponsored_vault_open_to_all_defaults_false() {
    let (_env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    assert!(!client.get_sponsored_vault_open_to_all());
}

#[test]
fn test_admin_creates_sponsored_vault() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    let did_uri = String::from_str(&env, "did:pkh:stellar:testnet:OWNER");
    client.create_sponsored_vault(&admin, &owner, &did_uri);
    // Vault exists: list_vc_ids returns empty without panicking.
    assert_eq!(client.list_vc_ids(&owner, &0_u32, &200_u32).len(), 0);
}

#[test]
fn test_authorized_sponsor_creates_sponsored_vault() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let sponsor = Address::generate(&env);
    client.add_sponsored_vault_sponsor(&sponsor);
    let owner = Address::generate(&env);
    let did_uri = String::from_str(&env, "did:pkh:stellar:testnet:OWNER");
    client.create_sponsored_vault(&sponsor, &owner, &did_uri);
    assert_eq!(client.list_vc_ids(&owner, &0_u32, &200_u32).len(), 0);
}

#[test]
#[should_panic]
fn test_unauthorized_address_cannot_create_sponsored_vault_in_restricted_mode() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    // Confirm restricted mode (default).
    assert!(!client.get_sponsored_vault_open_to_all());
    let random = Address::generate(&env);
    let owner = Address::generate(&env);
    let did_uri = String::from_str(&env, "did:pkh:stellar:testnet:OWNER");
    client.create_sponsored_vault(&random, &owner, &did_uri);
}

#[test]
fn test_open_mode_allows_anyone_to_create_sponsored_vault() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    client.set_sponsored_vault_open_to_all(&true);
    assert!(client.get_sponsored_vault_open_to_all());
    let random = Address::generate(&env);
    let owner = Address::generate(&env);
    let did_uri = String::from_str(&env, "did:pkh:stellar:testnet:OWNER");
    client.create_sponsored_vault(&random, &owner, &did_uri);
    assert_eq!(client.list_vc_ids(&owner, &0_u32, &200_u32).len(), 0);
}

#[test]
#[should_panic]
fn test_back_to_restricted_mode_blocks_unauthorized() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    client.set_sponsored_vault_open_to_all(&true);
    client.set_sponsored_vault_open_to_all(&false);
    let random = Address::generate(&env);
    let owner = Address::generate(&env);
    let did_uri = String::from_str(&env, "did:pkh:stellar:testnet:OWNER");
    client.create_sponsored_vault(&random, &owner, &did_uri);
}

#[test]
#[should_panic]
fn test_removed_sponsor_cannot_create_sponsored_vault() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let sponsor = Address::generate(&env);
    client.add_sponsored_vault_sponsor(&sponsor);
    client.remove_sponsored_vault_sponsor(&sponsor);
    let owner = Address::generate(&env);
    let did_uri = String::from_str(&env, "did:pkh:stellar:testnet:OWNER");
    // Must fail: sponsor was removed.
    client.create_sponsored_vault(&sponsor, &owner, &did_uri);
}

#[test]
#[should_panic]
fn test_duplicate_sponsored_vault_panics() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    let did_uri = String::from_str(&env, "did:pkh:stellar:testnet:OWNER");
    client.create_sponsored_vault(&admin, &owner, &did_uri);
    // Second creation for same owner must fail.
    client.create_sponsored_vault(&admin, &owner, &did_uri);
}

// --- Targeted auth tests ---
// The main test suite uses mock_all_auths() which bypasses all require_auth() checks.
// These tests use targeted mocks (or no mocks) to confirm that auth guards are
// actually enforced and would catch regressions where a guard is accidentally removed.

fn setup_no_mock() -> (Env, Address, Address, Address, VcVaultContractClient<'static>) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let contract_id = env.register(VcVaultContract, ());
    let client = VcVaultContractClient::new(&env, &contract_id);
    (env, admin, issuer, contract_id, client)
}

#[test]
#[should_panic]
fn test_auth_initialize_requires_admin_signature() {
    let (_env, admin, _issuer, _contract_id, client) = setup_no_mock();
    // No auth mocked — admin.require_auth() must fail.
    client.initialize(&admin);
}

#[test]
#[should_panic]
fn test_auth_nominate_admin_requires_current_admin_signature() {
    let (env, admin, _issuer, contract_id, client) = setup_no_mock();
    // Initialize with explicit admin auth only.
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "initialize",
            args: (&admin,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin);
    // No auth mocked for nominate_admin — must fail.
    let new_admin = Address::generate(&env);
    client.nominate_admin(&new_admin);
}

#[test]
#[should_panic]
fn test_auth_create_vault_requires_owner_signature() {
    let (env, admin, _issuer, contract_id, client) = setup_no_mock();
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "initialize",
            args: (&admin,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin);
    // Owner auth not mocked — create_vault must fail.
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:test"));
}

#[test]
#[should_panic]
fn test_auth_authorize_issuer_requires_vault_admin_signature() {
    let (env, admin, issuer, contract_id, client) = setup_no_mock();
    let owner = Address::generate(&env);
    let did = String::from_str(&env, "did:test");
    env.mock_auths(&[
        MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "initialize",
                args: (&admin,).into_val(&env),
                sub_invokes: &[],
            },
        },
        MockAuth {
            address: &owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "create_vault",
                args: (&owner, &did).into_val(&env),
                sub_invokes: &[],
            },
        },
    ]);
    client.initialize(&admin);
    client.create_vault(&owner, &did);
    // No auth mocked for authorize_issuer — must fail.
    client.authorize_issuer(&owner, &issuer);
}

// --- Linked VC tests ---

#[test]
fn test_issue_linked_requires_valid_parent_vc() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);

    // Foundation vault with a primary VC.
    let foundation = Address::generate(&env);
    client.create_vault(&foundation, &String::from_str(&env, "did:pkh:stellar:testnet:FOUNDATION"));
    let parent_vc_id = String::from_str(&env, "vc-empresa-001");
    let vc_data = String::from_str(&env, "<primary-data>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&foundation, &parent_vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);

    // Empresario vault receives a linked VC.
    let empresario = Address::generate(&env);
    client.create_vault(&empresario, &String::from_str(&env, "did:pkh:stellar:testnet:EMPRESARIO"));
    let linked_vc_id = String::from_str(&env, "vc-endorse-001");
    client.issue_linked(
        &issuer,
        &empresario,
        &linked_vc_id,
        &String::from_str(&env, "<endorse-data>"),
        &contract_id,
        &issuer_did,
        &foundation,
        &parent_vc_id,
    );

    assert_eq!(client.verify_vc(&empresario, &linked_vc_id), crate::model::VCStatus::Valid);
}

#[test]
#[should_panic]
fn test_issue_linked_fails_if_parent_not_found() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);

    let foundation = Address::generate(&env);
    client.create_vault(&foundation, &String::from_str(&env, "did:pkh:stellar:testnet:FOUNDATION"));

    let empresario = Address::generate(&env);
    client.create_vault(&empresario, &String::from_str(&env, "did:pkh:stellar:testnet:EMPRESARIO"));

    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    // parent_vc_id does not exist → ParentVCInvalid
    client.issue_linked(
        &issuer,
        &empresario,
        &String::from_str(&env, "vc-endorse-001"),
        &String::from_str(&env, "<endorse-data>"),
        &contract_id,
        &issuer_did,
        &foundation,
        &String::from_str(&env, "nonexistent-vc"),
    );
}

#[test]
#[should_panic]
fn test_issue_linked_fails_if_parent_revoked() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);

    let foundation = Address::generate(&env);
    client.create_vault(&foundation, &String::from_str(&env, "did:pkh:stellar:testnet:FOUNDATION"));
    let parent_vc_id = String::from_str(&env, "vc-empresa-001");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&foundation, &parent_vc_id, &String::from_str(&env, "<data>"), &contract_id, &issuer, &issuer_did, &0_i128);
    client.revoke(&foundation, &parent_vc_id, &String::from_str(&env, "2026-01-01T00:00:00Z"));

    let empresario = Address::generate(&env);
    client.create_vault(&empresario, &String::from_str(&env, "did:pkh:stellar:testnet:EMPRESARIO"));

    // parent VC is revoked → ParentVCInvalid
    client.issue_linked(
        &issuer,
        &empresario,
        &String::from_str(&env, "vc-endorse-001"),
        &String::from_str(&env, "<endorse-data>"),
        &contract_id,
        &issuer_did,
        &foundation,
        &parent_vc_id,
    );
}

#[test]
fn test_get_vc_parent_returns_none_for_regular_vc() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));
    let vc_id = String::from_str(&env, "vc-plain");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&owner, &vc_id, &String::from_str(&env, "<data>"), &contract_id, &issuer, &issuer_did, &0_i128);
    assert!(client.get_vc_parent(&owner, &vc_id).is_none());
}

#[test]
fn test_get_vc_parent_returns_link_for_linked_vc() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);

    let foundation = Address::generate(&env);
    client.create_vault(&foundation, &String::from_str(&env, "did:pkh:stellar:testnet:FOUNDATION"));
    let parent_vc_id = String::from_str(&env, "vc-primary");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(&foundation, &parent_vc_id, &String::from_str(&env, "<data>"), &contract_id, &issuer, &issuer_did, &0_i128);

    let empresario = Address::generate(&env);
    client.create_vault(&empresario, &String::from_str(&env, "did:pkh:stellar:testnet:EMPRESARIO"));
    let linked_vc_id = String::from_str(&env, "vc-linked");
    client.issue_linked(
        &issuer,
        &empresario,
        &linked_vc_id,
        &String::from_str(&env, "<linked-data>"),
        &contract_id,
        &issuer_did,
        &foundation,
        &parent_vc_id,
    );

    let result = client.get_vc_parent(&empresario, &linked_vc_id);
    assert!(result.is_some());
    let (returned_owner, returned_id) = result.unwrap();
    assert_eq!(returned_owner, foundation);
    assert_eq!(returned_id, parent_vc_id);
}

#[test]
fn test_foundation_flow_end_to_end() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);

    // Step 1: foundation creates its own vault.
    let foundation = Address::generate(&env);
    client.create_vault(&foundation, &String::from_str(&env, "did:pkh:stellar:testnet:FOUNDATION"));

    // Step 2: Admin sponsors vault for the empresario.
    let empresario = Address::generate(&env);
    client.create_sponsored_vault(
        &admin,
        &empresario,
        &String::from_str(&env, "did:pkh:stellar:testnet:EMPRESARIO"),
    );

    // Step 3: Foundation issues a primary VC in its own vault.
    let parent_vc_id = String::from_str(&env, "vc-empresa-001");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    client.issue(
        &foundation,
        &parent_vc_id,
        &String::from_str(&env, "<primary-data>"),
        &contract_id,
        &issuer,
        &issuer_did,
        &0_i128,
    );
    assert_eq!(client.verify_vc(&foundation, &parent_vc_id), crate::model::VCStatus::Valid);

    // Step 4: Empresario issues an endorsed VC linked to the foundation's VC.
    let linked_vc_id = String::from_str(&env, "vc-endorse-001");
    client.issue_linked(
        &issuer,
        &empresario,
        &linked_vc_id,
        &String::from_str(&env, "<endorse-data>"),
        &contract_id,
        &issuer_did,
        &foundation,
        &parent_vc_id,
    );

    // Step 5: Verify both VCs and confirm the parent link.
    assert_eq!(client.verify_vc(&foundation, &parent_vc_id), crate::model::VCStatus::Valid);
    assert_eq!(client.verify_vc(&empresario, &linked_vc_id), crate::model::VCStatus::Valid);
    let parent_link = client.get_vc_parent(&empresario, &linked_vc_id).unwrap();
    assert_eq!(parent_link.0, foundation);
    assert_eq!(parent_link.1, parent_vc_id);
}

#[test]
#[should_panic]
fn test_auth_issue_requires_issuer_signature() {
    let (env, admin, issuer, contract_id, client) = setup_no_mock();
    let owner = Address::generate(&env);
    let did = String::from_str(&env, "did:test");
    env.mock_auths(&[
        MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "initialize",
                args: (&admin,).into_val(&env),
                sub_invokes: &[],
            },
        },
        MockAuth {
            address: &owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "create_vault",
                args: (&owner, &did).into_val(&env),
                sub_invokes: &[],
            },
        },
    ]);
    client.initialize(&admin);
    client.create_vault(&owner, &did);
    // Issuer auth not mocked — issue must fail.
    client.issue(
        &owner,
        &String::from_str(&env, "vc-1"),
        &String::from_str(&env, "<data>"),
        &contract_id,
        &issuer,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
    );
}

// --- event coverage ----------------------------------------------------------

#[test]
fn test_push_emits_event_and_moves_vc() {
    use soroban_sdk::testutils::Events;

    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let from_owner = Address::generate(&env);
    let to_owner = Address::generate(&env);
    client.create_vault(&from_owner, &String::from_str(&env, "did:from"));
    client.create_vault(&to_owner, &String::from_str(&env, "did:to"));
    client.authorize_issuer(&from_owner, &issuer);
    let vc_id = String::from_str(&env, "vc-push-event");
    client.issue(
        &from_owner,
        &vc_id,
        &String::from_str(&env, "<data>"),
        &contract_id,
        &issuer,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
    );
    client.push(&from_owner, &to_owner, &vc_id, &issuer);

    // Check events BEFORE any subsequent invocation — env.events().all()
    // returns events from the most recent contract call only.
    assert_eq!(env.events().all().len(), 1, "push must emit exactly one VCPushed event");

    // VC moved: gone from source, present in destination.
    assert!(client.get_vc(&from_owner, &vc_id).is_none());
    assert!(client.get_vc(&to_owner, &vc_id).is_some());
}

#[test]
fn test_set_vault_admin_emits_event() {
    use soroban_sdk::testutils::Events;

    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    let new_admin = Address::generate(&env);
    client.set_vault_admin(&owner, &new_admin);

    // Exactly one event emitted by set_vault_admin (VaultAdminChanged).
    assert_eq!(env.events().all().len(), 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")] // VCAlreadyRevoked
fn test_push_revoked_vc_returns_already_revoked_error() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let from_owner = Address::generate(&env);
    let to_owner = Address::generate(&env);
    client.create_vault(&from_owner, &String::from_str(&env, "did:from"));
    client.create_vault(&to_owner, &String::from_str(&env, "did:to"));
    client.authorize_issuer(&from_owner, &issuer);
    let vc_id = String::from_str(&env, "vc-rev");
    client.issue(
        &from_owner,
        &vc_id,
        &String::from_str(&env, "<data>"),
        &contract_id,
        &issuer,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
    );
    client.revoke(&from_owner, &vc_id, &String::from_str(&env, "2025-01-01T00:00:00Z"));
    // Must fail with VCAlreadyRevoked (#7), not VCNotFound (#6).
    client.push(&from_owner, &to_owner, &vc_id, &issuer);
}

// --- O(1) index tests ---

#[test]
fn test_index_remove_middle_uses_swap_and_pop() {
    // Issuing three VCs places them at positions 0, 1, 2. Revoking the middle
    // one (position 1) must move the last one (position 2) into position 1
    // via swap-and-pop, leaving an active count of 2 with the surviving IDs
    // queryable via list_vc_ids.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let issuer_did = String::from_str(&env, "did:issuer");
    let data = String::from_str(&env, "<data>");
    let id_a = String::from_str(&env, "vc-a");
    let id_b = String::from_str(&env, "vc-b");
    let id_c = String::from_str(&env, "vc-c");
    client.issue(&owner, &id_a, &data, &contract_id, &issuer, &issuer_did, &0_i128);
    client.issue(&owner, &id_b, &data, &contract_id, &issuer, &issuer_did, &0_i128);
    client.issue(&owner, &id_c, &data, &contract_id, &issuer, &issuer_did, &0_i128);
    assert_eq!(client.list_vc_ids(&owner, &0_u32, &200_u32).len(), 3);

    client.revoke(&owner, &id_b, &String::from_str(&env, "2025-01-01T00:00:00Z"));
    let remaining = client.list_vc_ids(&owner, &0_u32, &200_u32);
    assert_eq!(remaining.len(), 2);
    assert!(remaining.contains(id_a.clone()));
    assert!(remaining.contains(id_c.clone()));
    assert!(!remaining.contains(id_b));
    // The revoked VC payload survives — only the active index is freed.
    assert_eq!(
        client.verify_vc(&owner, &id_a),
        crate::model::VCStatus::Valid
    );
    assert_eq!(
        client.verify_vc(&owner, &id_c),
        crate::model::VCStatus::Valid
    );
}

#[test]
fn test_revoke_frees_index_slot_for_reissuance_under_new_id() {
    // After revoke, the active count must drop so a new vc_id can take an
    // index slot. (Re-using the same vc_id is forbidden by VCAlreadyExists,
    // which is why we issue under a different id.)
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let issuer_did = String::from_str(&env, "did:issuer");
    let data = String::from_str(&env, "<data>");
    let id1 = String::from_str(&env, "vc-1");
    client.issue(&owner, &id1, &data, &contract_id, &issuer, &issuer_did, &0_i128);
    assert_eq!(client.list_vc_ids(&owner, &0_u32, &200_u32).len(), 1);
    client.revoke(&owner, &id1, &String::from_str(&env, "2025-01-01T00:00:00Z"));
    assert_eq!(client.list_vc_ids(&owner, &0_u32, &200_u32).len(), 0);
    let id2 = String::from_str(&env, "vc-2");
    client.issue(&owner, &id2, &data, &contract_id, &issuer, &issuer_did, &0_i128);
    assert_eq!(client.list_vc_ids(&owner, &0_u32, &200_u32).len(), 1);
}

#[test]
fn test_push_reindexes_source_and_destination() {
    // After push, the source vault's index must shrink and the destination's
    // must grow — both via the O(1) helpers.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let from_owner = Address::generate(&env);
    let to_owner = Address::generate(&env);
    client.create_vault(&from_owner, &String::from_str(&env, "did:from"));
    client.create_vault(&to_owner, &String::from_str(&env, "did:to"));
    client.authorize_issuer(&from_owner, &issuer);
    let issuer_did = String::from_str(&env, "did:issuer");
    let data = String::from_str(&env, "<data>");
    let id_a = String::from_str(&env, "vc-a");
    let id_b = String::from_str(&env, "vc-b");
    client.issue(&from_owner, &id_a, &data, &contract_id, &issuer, &issuer_did, &0_i128);
    client.issue(&from_owner, &id_b, &data, &contract_id, &issuer, &issuer_did, &0_i128);
    assert_eq!(client.list_vc_ids(&from_owner, &0_u32, &200_u32).len(), 2);
    assert_eq!(client.list_vc_ids(&to_owner, &0_u32, &200_u32).len(), 0);

    client.push(&from_owner, &to_owner, &id_a, &issuer);

    let from_ids = client.list_vc_ids(&from_owner, &0_u32, &200_u32);
    assert_eq!(from_ids.len(), 1);
    assert!(from_ids.contains(id_b));
    let to_ids = client.list_vc_ids(&to_owner, &0_u32, &200_u32);
    assert_eq!(to_ids.len(), 1);
    assert!(to_ids.contains(id_a));
}

#[test]
fn test_push_moves_parent_link_to_destination() {
    // Regression: VCParent must follow the VC into the destination so
    // get_vc_parent(to_owner, vc_id) returns the link, and the source no
    // longer reports a parent for a payload it does not hold.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let parent_owner = Address::generate(&env);
    let from_owner = Address::generate(&env);
    let to_owner = Address::generate(&env);
    client.create_vault(&parent_owner, &String::from_str(&env, "did:parent"));
    client.create_vault(&from_owner, &String::from_str(&env, "did:from"));
    client.create_vault(&to_owner, &String::from_str(&env, "did:to"));
    client.authorize_issuer(&parent_owner, &issuer);
    client.authorize_issuer(&from_owner, &issuer);

    let issuer_did = String::from_str(&env, "did:issuer");
    let data = String::from_str(&env, "<data>");
    let parent_id = String::from_str(&env, "vc-parent");
    let child_id = String::from_str(&env, "vc-child");

    client.issue(
        &parent_owner,
        &parent_id,
        &data,
        &contract_id,
        &issuer,
        &issuer_did,
        &0_i128,
    );
    client.issue_linked(
        &issuer,
        &from_owner,
        &child_id,
        &data,
        &contract_id,
        &issuer_did,
        &parent_owner,
        &parent_id,
    );
    // Sanity: parent link is at the source before push.
    let pre = client.get_vc_parent(&from_owner, &child_id);
    assert!(pre.is_some());
    let (pre_owner, pre_id) = pre.unwrap();
    assert_eq!(pre_owner, parent_owner);
    assert_eq!(pre_id, parent_id);

    client.push(&from_owner, &to_owner, &child_id, &issuer);

    // Link followed the VC.
    let post = client.get_vc_parent(&to_owner, &child_id);
    assert!(post.is_some());
    let (post_owner, post_id) = post.unwrap();
    assert_eq!(post_owner, parent_owner);
    assert_eq!(post_id, parent_id);
    // Source no longer claims a link for a VC it does not hold.
    assert!(client.get_vc_parent(&from_owner, &child_id).is_none());
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")] // ParentVCInvalid
fn test_issue_linked_rejects_pushed_away_parent() {
    // Regression: issue_linked must check both parent payload AND status.
    // Previously only status was checked; after push the source vault keeps a
    // stale Valid status as a vc_id-uniqueness tombstone, which would let an
    // attacker pass the source as parent for a payload that has moved away.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let parent_holder = Address::generate(&env);
    let new_holder = Address::generate(&env);
    let child_owner = Address::generate(&env);
    client.create_vault(&parent_holder, &String::from_str(&env, "did:parent"));
    client.create_vault(&new_holder, &String::from_str(&env, "did:new"));
    client.create_vault(&child_owner, &String::from_str(&env, "did:child"));
    client.authorize_issuer(&parent_holder, &issuer);
    client.authorize_issuer(&child_owner, &issuer);

    let issuer_did = String::from_str(&env, "did:issuer");
    let data = String::from_str(&env, "<data>");
    let parent_id = String::from_str(&env, "vc-parent");

    client.issue(
        &parent_holder,
        &parent_id,
        &data,
        &contract_id,
        &issuer,
        &issuer_did,
        &0_i128,
    );
    // Push the parent away. parent_holder retains a stale Valid status tombstone.
    client.push(&parent_holder, &new_holder, &parent_id, &issuer);

    // Attempt to link a new child to the orphaned source — must be rejected.
    let child_id = String::from_str(&env, "vc-child");
    client.issue_linked(
        &issuer,
        &child_owner,
        &child_id,
        &data,
        &contract_id,
        &issuer_did,
        &parent_holder,
        &parent_id,
    );
}

#[test]
fn test_index_remains_consistent_after_many_issues_and_revokes() {
    // Stress the swap-and-pop logic: issue 10 VCs, revoke half, ensure the
    // index reflects exactly the surviving IDs.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let issuer_did = String::from_str(&env, "did:issuer");
    let data = String::from_str(&env, "<data>");
    let revoke_date = String::from_str(&env, "2025-01-01T00:00:00Z");

    let labels = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
    let mut ids: soroban_sdk::Vec<String> = soroban_sdk::Vec::new(&env);
    for label in labels.iter() {
        let id = String::from_str(&env, label);
        client.issue(&owner, &id, &data, &contract_id, &issuer, &issuer_did, &0_i128);
        ids.push_back(id);
    }
    assert_eq!(client.list_vc_ids(&owner, &0_u32, &200_u32).len(), 10);

    // Revoke every other VC.
    for i in (0..10).step_by(2) {
        let id = ids.get_unchecked(i);
        client.revoke(&owner, &id, &revoke_date);
    }
    let remaining = client.list_vc_ids(&owner, &0_u32, &200_u32);
    assert_eq!(remaining.len(), 5);
    // Surviving VCs: indices 1, 3, 5, 7, 9 (b, d, f, h, j).
    for i in (1..10).step_by(2) {
        let id = ids.get_unchecked(i);
        assert!(remaining.contains(id));
    }
}

// --- Pagination tests ---

#[test]
fn test_vc_count_is_zero_for_empty_vault() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    assert_eq!(client.vc_count(&owner), 0);
}

#[test]
fn test_vc_count_tracks_issue_revoke_push() {
    // vc_count must reflect the active set: increment on issue, decrement on
    // revoke and on the source side of push, increment on the destination.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let from_owner = Address::generate(&env);
    let to_owner = Address::generate(&env);
    client.create_vault(&from_owner, &String::from_str(&env, "did:from"));
    client.create_vault(&to_owner, &String::from_str(&env, "did:to"));
    client.authorize_issuer(&from_owner, &issuer);

    let issuer_did = String::from_str(&env, "did:issuer");
    let data = String::from_str(&env, "<data>");

    assert_eq!(client.vc_count(&from_owner), 0);
    let id_a = String::from_str(&env, "vc-a");
    let id_b = String::from_str(&env, "vc-b");
    client.issue(&from_owner, &id_a, &data, &contract_id, &issuer, &issuer_did, &0_i128);
    client.issue(&from_owner, &id_b, &data, &contract_id, &issuer, &issuer_did, &0_i128);
    assert_eq!(client.vc_count(&from_owner), 2);

    client.revoke(&from_owner, &id_a, &String::from_str(&env, "2025-01-01T00:00:00Z"));
    assert_eq!(client.vc_count(&from_owner), 1);

    client.push(&from_owner, &to_owner, &id_b, &issuer);
    assert_eq!(client.vc_count(&from_owner), 0);
    assert_eq!(client.vc_count(&to_owner), 1);
}

#[test]
fn test_list_vc_ids_paginates_consistently() {
    // Issue 5 VCs. Querying with various (offset, limit) combinations must
    // partition the set without duplicates or gaps.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let issuer_did = String::from_str(&env, "did:issuer");
    let data = String::from_str(&env, "<data>");
    for label in ["a", "b", "c", "d", "e"].iter() {
        let id = String::from_str(&env, label);
        client.issue(&owner, &id, &data, &contract_id, &issuer, &issuer_did, &0_i128);
    }
    assert_eq!(client.vc_count(&owner), 5);

    // Full window.
    let all = client.list_vc_ids(&owner, &0_u32, &200_u32);
    assert_eq!(all.len(), 5);

    // First two and last three must reconstruct the full set.
    let first = client.list_vc_ids(&owner, &0_u32, &2_u32);
    let rest = client.list_vc_ids(&owner, &2_u32, &10_u32);
    assert_eq!(first.len(), 2);
    assert_eq!(rest.len(), 3);
    let mut joined = soroban_sdk::Vec::<String>::new(&env);
    for id in first.iter() {
        joined.push_back(id);
    }
    for id in rest.iter() {
        joined.push_back(id);
    }
    assert_eq!(joined.len(), 5);
    for id in all.iter() {
        assert!(joined.contains(id));
    }
}

#[test]
fn test_list_vc_ids_zero_limit_returns_empty() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    client.issue(
        &owner,
        &String::from_str(&env, "vc-1"),
        &String::from_str(&env, "<data>"),
        &contract_id,
        &issuer,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
    );
    let result = client.list_vc_ids(&owner, &0_u32, &0_u32);
    assert_eq!(result.len(), 0);
}

#[test]
fn test_list_vc_ids_offset_beyond_count_returns_empty() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    client.issue(
        &owner,
        &String::from_str(&env, "vc-1"),
        &String::from_str(&env, "<data>"),
        &contract_id,
        &issuer,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
    );
    // Vault has 1 VC at position 0; asking from 5 onward returns empty.
    let result = client.list_vc_ids(&owner, &5_u32, &10_u32);
    assert_eq!(result.len(), 0);
}

#[test]
fn test_list_vc_ids_limit_clamped_to_count() {
    // Asking for more than count returns exactly count entries — no padding,
    // no panic.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let issuer_did = String::from_str(&env, "did:issuer");
    let data = String::from_str(&env, "<data>");
    for label in ["a", "b", "c"].iter() {
        let id = String::from_str(&env, label);
        client.issue(&owner, &id, &data, &contract_id, &issuer, &issuer_did, &0_i128);
    }
    let result = client.list_vc_ids(&owner, &0_u32, &200_u32);
    assert_eq!(result.len(), 3);
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")] // LimitTooLarge
fn test_list_vc_ids_limit_above_max_panics() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    // MAX_LIST_LIMIT = 200; 201 must panic.
    client.list_vc_ids(&owner, &0_u32, &201_u32);
}

#[test]
fn test_vc_count_zero_for_unknown_vault() {
    // No panic, no read failure — unknown vaults report 0 active VCs.
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let stranger = Address::generate(&env);
    assert_eq!(client.vc_count(&stranger), 0);
}

// --- migrate_vc_index tests ---

/// Write a legacy `VaultVCIds(owner)` entry as if produced by v0.1. The new
/// O(1) index introduced in #20 has no public writer for this key, so tests
/// that need to simulate a pre-upgrade vault drop into `env.as_contract` to
/// populate it directly.
fn seed_legacy_vault_vc_ids(env: &Env, contract_id: &Address, owner: &Address, ids: &[&str]) {
    env.as_contract(contract_id, || {
        let mut vec_ids = soroban_sdk::Vec::<String>::new(env);
        for id in ids.iter() {
            vec_ids.push_back(String::from_str(env, id));
        }
        let key = crate::storage::DataKey::VaultVCIds(owner.clone());
        env.storage().persistent().set(&key, &vec_ids);
    });
}

#[test]
fn test_migrate_vc_index_moves_legacy_to_new_index() {
    let (env, admin, _issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));

    // Simulate a v0.1 vault that has three vc_ids in the legacy Vec but no
    // entries in the new index.
    seed_legacy_vault_vc_ids(&env, &contract_id, &owner, &["vc-a", "vc-b", "vc-c"]);
    assert_eq!(client.vc_count(&owner), 0);

    client.migrate_vc_index(&owner);

    // New index now has all three; legacy entry is gone.
    assert_eq!(client.vc_count(&owner), 3);
    let listed = client.list_vc_ids(&owner, &0_u32, &200_u32);
    assert_eq!(listed.len(), 3);
    assert!(listed.contains(String::from_str(&env, "vc-a")));
    assert!(listed.contains(String::from_str(&env, "vc-b")));
    assert!(listed.contains(String::from_str(&env, "vc-c")));
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")] // VCSAlreadyMigrated
fn test_migrate_vc_index_double_call_panics() {
    let (env, admin, _issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    seed_legacy_vault_vc_ids(&env, &contract_id, &owner, &["vc-1"]);

    client.migrate_vc_index(&owner);
    // Second call: vc_count > 0 already, must reject.
    client.migrate_vc_index(&owner);
}

#[test]
fn test_migrate_vc_index_with_no_legacy_is_noop() {
    // A vault created fresh post-upgrade has no legacy data and an empty new
    // index. migrate_vc_index must complete without panicking.
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    assert_eq!(client.vc_count(&owner), 0);

    client.migrate_vc_index(&owner);

    // Still empty, no panic.
    assert_eq!(client.vc_count(&owner), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")] // VCSAlreadyMigrated
fn test_migrate_vc_index_panics_when_new_index_has_entries() {
    // A vault that was created post-upgrade and already received VCs through
    // the new index has nothing to migrate; calling migrate_vc_index must be
    // rejected so callers can detect "already on the new schema" without
    // pretending the call was successful.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    client.issue(
        &owner,
        &String::from_str(&env, "vc-1"),
        &String::from_str(&env, "<data>"),
        &contract_id,
        &issuer,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
    );
    assert_eq!(client.vc_count(&owner), 1);

    client.migrate_vc_index(&owner);
}

#[test]
fn test_migrate_vc_index_requires_no_auth() {
    // Migration is deterministic from on-chain state: any caller — not just
    // the vault admin — can drive it. This test runs without
    // env.mock_all_auths to confirm there is no `require_auth` on the path.
    let env = Env::default();
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let contract_id = env.register(VcVaultContract, ());
    let client = VcVaultContractClient::new(&env, &contract_id);

    // initialize and create_vault still need auths; mock just for those.
    env.mock_all_auths();
    client.initialize(&admin);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    seed_legacy_vault_vc_ids(&env, &contract_id, &owner, &["vc-1", "vc-2"]);

    // Drop the auth mock so the next call would fail if any require_auth
    // were inserted in the path.
    env.set_auths(&[]);

    client.migrate_vc_index(&owner);
    assert_eq!(client.vc_count(&owner), 2);
}

// --- batch_issue tests ---

#[test]
fn test_batch_issue_writes_all_vcs_in_order() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let issuer_did = String::from_str(&env, "did:issuer");
    let data = String::from_str(&env, "<data>");

    let id_a = String::from_str(&env, "vc-a");
    let id_b = String::from_str(&env, "vc-b");
    let id_c = String::from_str(&env, "vc-c");
    let vcs = soroban_sdk::vec![
        &env,
        (id_a.clone(), data.clone()),
        (id_b.clone(), data.clone()),
        (id_c.clone(), data.clone()),
    ];
    let returned = client.batch_issue(&issuer, &owner, &contract_id, &issuer_did, &0_i128, &vcs);

    assert_eq!(returned.len(), 3);
    assert_eq!(returned.get_unchecked(0), id_a);
    assert_eq!(returned.get_unchecked(1), id_b);
    assert_eq!(returned.get_unchecked(2), id_c);
    assert_eq!(client.vc_count(&owner), 3);
    let listed = client.list_vc_ids(&owner, &0_u32, &10_u32);
    assert!(listed.contains(id_a));
    assert!(listed.contains(id_b));
    assert!(listed.contains(id_c));
}

#[test]
fn test_batch_issue_at_max_size_succeeds() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let issuer_did = String::from_str(&env, "did:issuer");
    let data = String::from_str(&env, "<data>");

    let vcs = soroban_sdk::vec![
        &env,
        (String::from_str(&env, "vc-1"), data.clone()),
        (String::from_str(&env, "vc-2"), data.clone()),
        (String::from_str(&env, "vc-3"), data.clone()),
        (String::from_str(&env, "vc-4"), data.clone()),
        (String::from_str(&env, "vc-5"), data.clone()),
    ];
    let returned = client.batch_issue(&issuer, &owner, &contract_id, &issuer_did, &0_i128, &vcs);
    assert_eq!(returned.len(), 5);
    assert_eq!(client.vc_count(&owner), 5);
}

#[test]
#[should_panic(expected = "Error(Contract, #17)")] // BatchTooLarge
fn test_batch_issue_above_max_size_panics() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let data = String::from_str(&env, "<data>");

    let vcs = soroban_sdk::vec![
        &env,
        (String::from_str(&env, "vc-1"), data.clone()),
        (String::from_str(&env, "vc-2"), data.clone()),
        (String::from_str(&env, "vc-3"), data.clone()),
        (String::from_str(&env, "vc-4"), data.clone()),
        (String::from_str(&env, "vc-5"), data.clone()),
        (String::from_str(&env, "vc-6"), data.clone()),
    ];
    client.batch_issue(
        &issuer,
        &owner,
        &contract_id,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
        &vcs,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #18)")] // BatchEmpty
fn test_batch_issue_empty_panics() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let vcs = soroban_sdk::Vec::<(String, String)>::new(&env);
    client.batch_issue(
        &issuer,
        &owner,
        &contract_id,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
        &vcs,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")] // VCAlreadyExists
fn test_batch_issue_with_duplicate_within_batch_panics() {
    // First entry writes vc-x; second entry's existence check finds it and
    // panics with VCAlreadyExists.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let data = String::from_str(&env, "<data>");
    let dup = String::from_str(&env, "vc-x");
    let vcs = soroban_sdk::vec![&env, (dup.clone(), data.clone()), (dup, data),];
    client.batch_issue(
        &issuer,
        &owner,
        &contract_id,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
        &vcs,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")] // VCAlreadyExists
fn test_batch_issue_with_existing_vc_panics() {
    // A VC with this id was previously issued; batch's existence check
    // catches it on the first iteration.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let issuer_did = String::from_str(&env, "did:issuer");
    let data = String::from_str(&env, "<data>");

    client.issue(
        &owner,
        &String::from_str(&env, "vc-existing"),
        &data,
        &contract_id,
        &issuer,
        &issuer_did,
        &0_i128,
    );

    let vcs = soroban_sdk::vec![
        &env,
        (String::from_str(&env, "vc-new"), data.clone()),
        (String::from_str(&env, "vc-existing"), data),
    ];
    client.batch_issue(&issuer, &owner, &contract_id, &issuer_did, &0_i128, &vcs);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")] // VaultRevoked
fn test_batch_issue_on_revoked_vault_panics() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    client.revoke_vault(&owner);
    let data = String::from_str(&env, "<data>");
    let vcs = soroban_sdk::vec![&env, (String::from_str(&env, "vc-1"), data),];
    client.batch_issue(
        &issuer,
        &owner,
        &contract_id,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
        &vcs,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")] // InvalidVaultContract
fn test_batch_issue_with_wrong_vault_contract_panics() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let wrong_contract = Address::generate(&env);
    let data = String::from_str(&env, "<data>");
    let vcs = soroban_sdk::vec![&env, (String::from_str(&env, "vc-1"), data),];
    client.batch_issue(
        &issuer,
        &owner,
        &wrong_contract,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
        &vcs,
    );
}

#[test]
fn test_batch_issue_emits_one_event_per_vc() {
    // Off-chain indexers expect one VCIssued per credential, even when the
    // credentials are written together. Capture events before any read so
    // env.events().all() still holds them.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let data = String::from_str(&env, "<data>");
    let vcs = soroban_sdk::vec![
        &env,
        (String::from_str(&env, "vc-a"), data.clone()),
        (String::from_str(&env, "vc-b"), data.clone()),
        (String::from_str(&env, "vc-c"), data),
    ];
    client.batch_issue(
        &issuer,
        &owner,
        &contract_id,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
        &vcs,
    );
    // 3 VCIssued events from the batch (no other contract calls between).
    assert_eq!(env.events().all().len(), 3);
}

#[test]
fn test_batch_issue_auto_authorizes_unknown_issuer() {
    // Mirrors single issue() semantics: if the issuer is not yet on the
    // vault's authorized list and not in the denied list, batch_issue
    // auto-authorizes them.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    // No explicit authorize_issuer call.
    let data = String::from_str(&env, "<data>");
    let vcs = soroban_sdk::vec![&env, (String::from_str(&env, "vc-1"), data),];
    client.batch_issue(
        &issuer,
        &owner,
        &contract_id,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
        &vcs,
    );
    assert_eq!(client.vc_count(&owner), 1);
}

#[test]
fn test_migrate_vc_index_preserves_legacy_order() {
    // The legacy Vec is enumerated in stored order and append_vc_to_index
    // assigns positions 0..N in iteration order. After migration, listing
    // from the new index returns the same sequence.
    let (env, admin, _issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    seed_legacy_vault_vc_ids(&env, &contract_id, &owner, &["first", "second", "third"]);

    client.migrate_vc_index(&owner);

    let listed = client.list_vc_ids(&owner, &0_u32, &10_u32);
    assert_eq!(listed.len(), 3);
    assert_eq!(listed.get_unchecked(0), String::from_str(&env, "first"));
    assert_eq!(listed.get_unchecked(1), String::from_str(&env, "second"));
    assert_eq!(listed.get_unchecked(2), String::from_str(&env, "third"));
}

// --- Input length cap tests ---
//
// Each cap is enforced at every entrypoint that accepts the field, so the
// tests pick the entrypoint where each input first reaches the contract:
// `vc_id` and `vc_data` via issue(), `did_uri` via create_vault(),
// `issuer_did` via issue(), `date` via revoke(), and the issuers-list
// length via authorize_issuers().
//
// Strings exactly at the cap must succeed; one byte over must panic with
// InputTooLong (#19) — except for the issuer-list cap, which uses
// IssuerListTooLong (#20).

fn long_string(env: &Env, byte: u8, n: usize) -> String {
    extern crate alloc;
    let v: alloc::vec::Vec<u8> = (0..n).map(|_| byte).collect();
    String::from_bytes(env, &v)
}

#[test]
fn test_create_vault_accepts_did_uri_at_max_len() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    let did_uri = long_string(&env, b'd', 256); // MAX_DID_URI_LEN
    client.create_vault(&owner, &did_uri);
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")] // InputTooLong
fn test_create_vault_rejects_did_uri_over_max_len() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    let did_uri = long_string(&env, b'd', 257);
    client.create_vault(&owner, &did_uri);
}

#[test]
fn test_issue_accepts_vc_id_at_max_len() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let vc_id = long_string(&env, b'a', 64); // MAX_VC_ID_LEN
    client.issue(
        &owner,
        &vc_id,
        &String::from_str(&env, "<data>"),
        &contract_id,
        &issuer,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
    );
    assert_eq!(client.vc_count(&owner), 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")] // InputTooLong
fn test_issue_rejects_vc_id_over_max_len() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let vc_id = long_string(&env, b'a', 65);
    client.issue(
        &owner,
        &vc_id,
        &String::from_str(&env, "<data>"),
        &contract_id,
        &issuer,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")] // InputTooLong
fn test_issue_rejects_vc_data_over_max_len() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    // MAX_VC_DATA_LEN is 10_000; 10_001 bytes must reject.
    let vc_data = long_string(&env, b'd', 10_001);
    client.issue(
        &owner,
        &String::from_str(&env, "vc-1"),
        &vc_data,
        &contract_id,
        &issuer,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")] // InputTooLong
fn test_issue_rejects_issuer_did_over_max_len() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let issuer_did = long_string(&env, b'i', 257);
    client.issue(
        &owner,
        &String::from_str(&env, "vc-1"),
        &String::from_str(&env, "<data>"),
        &contract_id,
        &issuer,
        &issuer_did,
        &0_i128,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")] // InputTooLong
fn test_revoke_rejects_date_over_max_len() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let vc_id = String::from_str(&env, "vc-1");
    client.issue(
        &owner,
        &vc_id,
        &String::from_str(&env, "<data>"),
        &contract_id,
        &issuer,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
    );
    let date = long_string(&env, b'X', 65); // MAX_DATE_LEN = 64
    client.revoke(&owner, &vc_id, &date);
}

#[test]
fn test_authorize_issuers_accepts_max_list_size() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    let mut issuers = soroban_sdk::Vec::<Address>::new(&env);
    for _ in 0..100 {
        // MAX_ISSUERS_LIST
        issuers.push_back(Address::generate(&env));
    }
    client.authorize_issuers(&owner, &issuers);
}

#[test]
#[should_panic(expected = "Error(Contract, #20)")] // IssuerListTooLong
fn test_authorize_issuers_rejects_oversized_list() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    let mut issuers = soroban_sdk::Vec::<Address>::new(&env);
    for _ in 0..101 {
        issuers.push_back(Address::generate(&env));
    }
    client.authorize_issuers(&owner, &issuers);
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")] // InputTooLong
fn test_batch_issue_rejects_oversized_vc_id_within_batch() {
    // The cap applies inside batch_issue too: even if 4 entries are valid, a
    // 5th oversize id rejects the whole batch.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let data = String::from_str(&env, "<data>");
    let bad_id = long_string(&env, b'z', 65);
    let vcs = soroban_sdk::vec![
        &env,
        (String::from_str(&env, "vc-1"), data.clone()),
        (bad_id, data),
    ];
    client.batch_issue(
        &issuer,
        &owner,
        &contract_id,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
        &vcs,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")] // InputTooLong
fn test_get_vc_rejects_oversized_vc_id() {
    // Read paths cap the input too so an attacker can't force the contract
    // to spend instructions hashing a 1MB key before the lookup misses.
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    let vc_id = long_string(&env, b'q', 65);
    client.get_vc(&owner, &vc_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #20)")] // IssuerListTooLong
fn test_authorize_issuer_rejects_when_list_at_cap() {
    // Cap must apply to single-add too, not just the bulk replace path.
    // Fill the list to MAX_ISSUERS_LIST=100 via authorize_issuers (which is
    // capped at exactly that count), then authorize_issuer one more — must
    // panic with IssuerListTooLong instead of silently growing past the cap.
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    let mut issuers = soroban_sdk::Vec::<Address>::new(&env);
    for _ in 0..100 {
        issuers.push_back(Address::generate(&env));
    }
    client.authorize_issuers(&owner, &issuers);
    let extra = Address::generate(&env);
    client.authorize_issuer(&owner, &extra);
}

#[test]
#[should_panic(expected = "Error(Contract, #20)")] // IssuerListTooLong
fn test_issue_rejects_auto_authorization_when_list_at_cap() {
    // Auto-authorization inside ensure_issuer_authorized is the silent
    // growth path CodeRabbit flagged: an attacker could spam issue() from
    // many fresh addresses to grow VaultIssuers past the cap. The fix moves
    // the cap check into vault::authorize_issuer, which fires on this path.
    let (env, admin, _issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    let mut issuers = soroban_sdk::Vec::<Address>::new(&env);
    for _ in 0..100 {
        issuers.push_back(Address::generate(&env));
    }
    client.authorize_issuers(&owner, &issuers);
    let attacker = Address::generate(&env);
    client.issue(
        &owner,
        &String::from_str(&env, "vc-1"),
        &String::from_str(&env, "<data>"),
        &contract_id,
        &attacker,
        &String::from_str(&env, "did:attacker"),
        &0_i128,
    );
}

// --- Event emission tests ---
//
// env.events().all() returns events from the last invocation only, so each
// test calls the entrypoint and asserts the event count immediately —
// before any other contract call that would clear the event buffer.

#[test]
fn test_initialize_emits_contract_initialized() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::ContractInitialized;
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(VcVaultContract, ());
    let client = VcVaultContractClient::new(&env, &contract_id);
    client.initialize(&admin);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = ContractInitialized { admin: admin.clone() };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

#[test]
fn test_nominate_admin_emits_admin_nominated() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::AdminNominated;
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let nominee = Address::generate(&env);
    client.nominate_admin(&nominee);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = AdminNominated { current_admin: admin.clone(), nominee: nominee.clone() };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

#[test]
fn test_accept_contract_admin_emits_admin_transferred() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::AdminTransferred;
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let nominee = Address::generate(&env);
    client.nominate_admin(&nominee);
    // accept_contract_admin emits one event in its own invocation; the prior
    // nominate_admin event is in a separate invocation and not in this buffer.
    client.accept_contract_admin();
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = AdminTransferred { old_admin: admin.clone(), new_admin: nominee.clone() };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

#[test]
fn test_set_fee_enabled_emits_fee_enabled_changed() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::FeeEnabledChanged;
    let (_env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    // setup's env mocks auths; reuse without renaming.
    let env_ref = client.env.clone();
    client.set_fee_enabled(&true);
    let events = env_ref.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = FeeEnabledChanged { enabled: true };
    assert_eq!(topics, expected.topics(&env_ref));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env_ref, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env_ref, &expected.data(&env_ref)).unwrap(),
    );
}

#[test]
fn test_set_fee_config_emits_fee_config_set() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::FeeConfigSet;
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let token = Address::generate(&env);
    let dest = Address::generate(&env);
    client.set_fee_config(&token, &dest, &1_500_000_i128);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = FeeConfigSet { token_contract: token.clone(), fee_dest: dest.clone(), fee_amount: 1_500_000_i128 };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

#[test]
fn test_set_fee_admin_emits_fee_admin_set() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::FeeAdminSet;
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    client.set_fee_admin(&500_i128);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = FeeAdminSet { amount: 500_i128 };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

#[test]
fn test_set_fee_standard_emits_fee_standard_set() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::FeeStandardSet;
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    client.set_fee_standard(&2_000_000_i128);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = FeeStandardSet { amount: 2_000_000_i128 };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

#[test]
fn test_set_fee_early_emits_fee_early_set() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::FeeEarlySet;
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    client.set_fee_early(&350_000_i128);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = FeeEarlySet { amount: 350_000_i128 };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

#[test]
fn test_set_fee_custom_emits_fee_custom_set() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::FeeCustomSet;
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    client.set_fee_custom(&issuer, &100_000_i128);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = FeeCustomSet { issuer: issuer.clone(), amount: 100_000_i128 };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

#[test]
fn test_set_sponsored_vault_open_to_all_emits_event() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::SponsorOpenToAllChanged;
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    client.set_sponsored_vault_open_to_all(&true);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = SponsorOpenToAllChanged { open: true };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

#[test]
fn test_add_sponsored_vault_sponsor_emits_sponsor_added() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::SponsorAdded;
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let sponsor = Address::generate(&env);
    client.add_sponsored_vault_sponsor(&sponsor);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = SponsorAdded { sponsor: sponsor.clone() };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

#[test]
fn test_remove_sponsored_vault_sponsor_emits_sponsor_removed() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::SponsorRemoved;
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let sponsor = Address::generate(&env);
    client.add_sponsored_vault_sponsor(&sponsor);
    // The remove call is the last invocation; the add was a separate one.
    client.remove_sponsored_vault_sponsor(&sponsor);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = SponsorRemoved { sponsor: sponsor.clone() };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

#[test]
fn test_migrate_vc_index_emits_event_with_count() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::VaultIndexMigrated;
    let (env, admin, _issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    seed_legacy_vault_vc_ids(&env, &contract_id, &owner, &["x", "y", "z"]);
    // migrate_vc_index is the last invocation; we don't care about events from
    // create_vault (different invocation). The exact event count for this
    // invocation is 1 (VaultIndexMigrated).
    client.migrate_vc_index(&owner);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = VaultIndexMigrated { owner: owner.clone(), migrated_count: 3 };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

#[test]
fn test_migrate_vc_index_noop_still_emits_event() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::VaultIndexMigrated;
    // Even a no-op migration (vault that never had legacy data) emits the
    // event so indexers see the migration attempt resolved.
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.migrate_vc_index(&owner);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = VaultIndexMigrated { owner: owner.clone(), migrated_count: 0 };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

/// Seed a legacy `LegacyVaultVCs(owner)` entry as if produced by v0 (pre
/// VaultVC + VaultVCIds split). Mirrors `seed_legacy_vault_vc_ids` but for
/// the older schema consumed by `migrate(owner)`.
fn seed_legacy_vault_vcs(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    ids: &[&str],
    data: &str,
    issuance_contract: &Address,
    issuer_did: &str,
) {
    env.as_contract(contract_id, || {
        let mut vec_vcs =
            soroban_sdk::Vec::<crate::model::VerifiableCredential>::new(env);
        for id in ids.iter() {
            vec_vcs.push_back(crate::model::VerifiableCredential {
                id: String::from_str(env, id),
                data: String::from_str(env, data),
                issuance_contract: issuance_contract.clone(),
                issuer_did: String::from_str(env, issuer_did),
            });
        }
        let key = crate::storage::DataKey::LegacyVaultVCs(owner.clone());
        env.storage().persistent().set(&key, &vec_vcs);
    });
}

#[test]
fn test_migrate_emits_vault_migrated() {
    use soroban_sdk::{Event as SorobanEvent, Map, Symbol, TryFromVal, Val};
    use crate::events::VaultMigrated;
    // Simulate a v0 vault with one VC in LegacyVaultVCs and verify migrate()
    // emits VaultMigrated. The legacy schema is reached only via the v0 →
    // v0.1 migration path, which is what migrate() is for.
    let (env, admin, _issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    seed_legacy_vault_vcs(
        &env,
        &contract_id,
        &owner,
        &["legacy-vc-1"],
        "<data>",
        &contract_id,
        "did:issuer",
    );
    // migrate() writes via vault::store_vc which appends to the new index,
    // but emits VaultMigrated as the only event from this entrypoint (the
    // VC-level writes do not emit events because vault::store_vc never has).
    client.migrate(&owner);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (_, topics, data) = events.get(0).unwrap();
    let expected = VaultMigrated { owner: owner.clone() };
    assert_eq!(topics, expected.topics(&env));
    assert_eq!(
        Map::<Symbol, Val>::try_from_val(&env, &data).unwrap(),
        Map::<Symbol, Val>::try_from_val(&env, &expected.data(&env)).unwrap(),
    );
}

// --- Issuer O(1) index tests ---

fn seed_legacy_vault_issuers(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    addrs: &[&Address],
) {
    env.as_contract(contract_id, || {
        let mut vec_issuers = soroban_sdk::Vec::<Address>::new(env);
        for addr in addrs.iter() {
            vec_issuers.push_back((*addr).clone());
        }
        let key = crate::storage::DataKey::VaultIssuers(owner.clone());
        env.storage().persistent().set(&key, &vec_issuers);
    });
}

fn seed_legacy_vault_denied_issuers(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    addrs: &[&Address],
) {
    env.as_contract(contract_id, || {
        let mut vec_denied = soroban_sdk::Vec::<Address>::new(env);
        for addr in addrs.iter() {
            vec_denied.push_back((*addr).clone());
        }
        let key = crate::storage::DataKey::VaultDeniedIssuers(owner.clone());
        env.storage().persistent().set(&key, &vec_denied);
    });
}

#[test]
fn test_migrate_issuer_index_no_legacy() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    // No legacy issuer data — migrating is a no-op but must succeed.
    client.migrate_issuer_index(&owner);
    assert_eq!(client.authorized_issuer_count(&owner), 0);
    assert_eq!(client.denied_issuer_count(&owner), 0);
}

#[test]
fn test_migrate_issuer_index_with_issuers() {
    let (env, admin, _issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    let issuer1 = Address::generate(&env);
    let issuer2 = Address::generate(&env);
    seed_legacy_vault_issuers(&env, &contract_id, &owner, &[&issuer1, &issuer2]);
    client.migrate_issuer_index(&owner);
    assert_eq!(client.authorized_issuer_count(&owner), 2);
    let listed = client.list_authorized_issuers(&owner, &0_u32, &100_u32);
    assert!(listed.contains(issuer1));
    assert!(listed.contains(issuer2));
}

#[test]
fn test_migrate_issuer_index_with_denied() {
    let (env, admin, _issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    let denied1 = Address::generate(&env);
    seed_legacy_vault_denied_issuers(&env, &contract_id, &owner, &[&denied1]);
    client.migrate_issuer_index(&owner);
    assert_eq!(client.denied_issuer_count(&owner), 1);
    let listed = client.list_denied_issuers(&owner, &0_u32, &100_u32);
    assert!(listed.contains(denied1));
}

#[test]
#[should_panic]
fn test_migrate_issuer_index_already_done() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    // Authorize an issuer to populate the new index.
    client.authorize_issuer(&owner, &issuer);
    // Second call must panic because the index already has entries.
    client.migrate_issuer_index(&owner);
}

#[test]
fn test_migrate_issuer_index_emits_event() {
    let (env, admin, _issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    let i1 = Address::generate(&env);
    let i2 = Address::generate(&env);
    let d1 = Address::generate(&env);
    seed_legacy_vault_issuers(&env, &contract_id, &owner, &[&i1, &i2]);
    seed_legacy_vault_denied_issuers(&env, &contract_id, &owner, &[&d1]);
    client.migrate_issuer_index(&owner);
    assert_eq!(env.events().all().len(), 1);
}

#[test]
fn test_list_authorized_issuers_pagination() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    let i1 = Address::generate(&env);
    let i2 = Address::generate(&env);
    let i3 = Address::generate(&env);
    client.authorize_issuer(&owner, &i1);
    client.authorize_issuer(&owner, &i2);
    client.authorize_issuer(&owner, &i3);
    // Full page.
    let all = client.list_authorized_issuers(&owner, &0_u32, &100_u32);
    assert_eq!(all.len(), 3);
    // Paginated first two.
    let page1 = client.list_authorized_issuers(&owner, &0_u32, &2_u32);
    assert_eq!(page1.len(), 2);
    // Offset past end.
    let empty = client.list_authorized_issuers(&owner, &10_u32, &100_u32);
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_list_denied_issuers_pagination() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    let i1 = Address::generate(&env);
    let i2 = Address::generate(&env);
    client.authorize_issuer(&owner, &i1);
    client.authorize_issuer(&owner, &i2);
    client.revoke_issuer(&owner, &i1);
    client.revoke_issuer(&owner, &i2);
    let all = client.list_denied_issuers(&owner, &0_u32, &100_u32);
    assert_eq!(all.len(), 2);
    let page1 = client.list_denied_issuers(&owner, &0_u32, &1_u32);
    assert_eq!(page1.len(), 1);
    let empty = client.list_denied_issuers(&owner, &5_u32, &100_u32);
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_authorized_issuer_count() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    assert_eq!(client.authorized_issuer_count(&owner), 0);
    client.authorize_issuer(&owner, &issuer);
    assert_eq!(client.authorized_issuer_count(&owner), 1);
}

#[test]
fn test_is_authorized_o1() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let listed = client.list_authorized_issuers(&owner, &0_u32, &100_u32);
    assert!(listed.contains(issuer));
}

#[test]
fn test_revoke_issuer_updates_index() {
    let (env, admin, issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    assert_eq!(client.authorized_issuer_count(&owner), 1);
    assert_eq!(client.denied_issuer_count(&owner), 0);
    client.revoke_issuer(&owner, &issuer);
    assert_eq!(client.authorized_issuer_count(&owner), 0);
    assert_eq!(client.denied_issuer_count(&owner), 1);
    let denied = client.list_denied_issuers(&owner, &0_u32, &100_u32);
    assert!(denied.contains(issuer));
}

#[test]
fn test_auto_authorize_on_repeated_issue() {
    // Exercises the auto-authorize path in ensure_issuer_authorized across
    // multiple issuances: issuer must end up in the index exactly once.
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    let issuer_did = String::from_str(&env, "did:issuer");
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));

    let vc_data = String::from_str(&env, "data");
    for id in ["vc-0","vc-1","vc-2","vc-3","vc-4","vc-5","vc-6","vc-7","vc-8","vc-9"] {
        let vc_id = String::from_str(&env, id);
        client.issue(&owner, &vc_id, &vc_data, &contract_id, &issuer, &issuer_did, &0_i128);
    }

    assert_eq!(client.authorized_issuer_count(&owner), 1);
    assert_eq!(client.vc_count(&owner), 10);
}

// --- Fee bounds validation tests ---

#[test]
#[should_panic(expected = "Error(Contract, #22)")]
fn test_set_fee_config_rejects_negative_amount() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let token = Address::generate(&env);
    let dest = Address::generate(&env);
    client.set_fee_config(&token, &dest, &-1_i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #23)")]
fn test_set_fee_config_rejects_amount_over_max() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let token = Address::generate(&env);
    let dest = Address::generate(&env);
    client.set_fee_config(&token, &dest, &1_000_000_000_000_000_001_i128);
}

#[test]
fn test_set_fee_config_accepts_zero() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let token = Address::generate(&env);
    let dest = Address::generate(&env);
    client.set_fee_config(&token, &dest, &0_i128);
    assert_eq!(client.fee_config().fee_amount, Some(0));
}

#[test]
fn test_set_fee_config_accepts_max() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let token = Address::generate(&env);
    let dest = Address::generate(&env);
    client.set_fee_config(&token, &dest, &1_000_000_000_000_000_000_i128);
    assert_eq!(client.fee_config().fee_amount, Some(1_000_000_000_000_000_000));
}

#[test]
#[should_panic(expected = "Error(Contract, #22)")]
fn test_set_fee_admin_rejects_negative() {
    let (_env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    client.set_fee_admin(&-1_i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #22)")]
fn test_set_fee_standard_rejects_negative() {
    let (_env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    client.set_fee_standard(&-1_i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #22)")]
fn test_set_fee_early_rejects_negative() {
    let (_env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    client.set_fee_early(&-1_i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #22)")]
fn test_set_fee_custom_rejects_negative() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let issuer = Address::generate(&env);
    client.set_fee_custom(&issuer, &-1_i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #22)")]
fn test_issue_rejects_negative_fee_override() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    client.issue(
        &owner,
        &String::from_str(&env, "vc-1"),
        &String::from_str(&env, "data"),
        &contract_id,
        &issuer,
        &String::from_str(&env, "did:issuer"),
        &-1_i128,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #23)")]
fn test_issue_rejects_fee_override_over_max() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    client.issue(
        &owner,
        &String::from_str(&env, "vc-1"),
        &String::from_str(&env, "data"),
        &contract_id,
        &issuer,
        &String::from_str(&env, "did:issuer"),
        &1_000_000_000_000_000_001_i128,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #22)")]
fn test_batch_issue_rejects_negative_fee_override() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);
    let vcs = vec![
        &env,
        (
            String::from_str(&env, "vc-1"),
            String::from_str(&env, "data"),
        ),
    ];
    client.batch_issue(
        &issuer,
        &owner,
        &contract_id,
        &String::from_str(&env, "did:issuer"),
        &-1_i128,
        &vcs,
    );
}

// --- migrate_vc_index_chunk tests ---

#[test]
fn test_migrate_vc_index_chunk_partial_then_complete() {
    let (env, admin, _issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));

    seed_legacy_vault_vc_ids(
        &env,
        &contract_id,
        &owner,
        &[
            "vc-0", "vc-1", "vc-2", "vc-3", "vc-4",
            "vc-5", "vc-6", "vc-7", "vc-8", "vc-9",
            "vc-10", "vc-11", "vc-12", "vc-13", "vc-14",
        ],
    );

    // First chunk: migrates 10, returns 5 remaining.
    let remaining = client.migrate_vc_index_chunk(&owner, &10_u32);
    assert_eq!(remaining, 5);
    assert_eq!(client.vc_count(&owner), 10);

    // Second chunk: completes migration, returns 0.
    let remaining = client.migrate_vc_index_chunk(&owner, &10_u32);
    assert_eq!(remaining, 0);
    assert_eq!(client.vc_count(&owner), 15);

    let listed = client.list_vc_ids(&owner, &0_u32, &200_u32);
    assert_eq!(listed.len(), 15);
    assert!(listed.contains(String::from_str(&env, "vc-0")));
    assert!(listed.contains(String::from_str(&env, "vc-7")));
    assert!(listed.contains(String::from_str(&env, "vc-14")));
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")] // VCSAlreadyMigrated
fn test_migrate_vc_index_chunk_already_migrated() {
    let (env, admin, _issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    seed_legacy_vault_vc_ids(&env, &contract_id, &owner, &["vc-1"]);

    client.migrate_vc_index(&owner);
    // Now vc_count > 0 and no legacy entry; chunk call must panic.
    client.migrate_vc_index_chunk(&owner, &10_u32);
}

#[test]
fn test_migrate_vc_index_chunk_fresh_vault_noop() {
    let (env, admin, _issuer, _contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));

    let remaining = client.migrate_vc_index_chunk(&owner, &10_u32);
    assert_eq!(remaining, 0);
    assert_eq!(client.vc_count(&owner), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")] // VaultFull
fn test_vault_full_at_u32_max() {
    let (env, admin, issuer, contract_id, client) = setup();
    client.initialize(&admin);
    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:owner"));
    client.authorize_issuer(&owner, &issuer);

    // Seed VaultVCCount = u32::MAX directly to simulate overflow boundary.
    env.as_contract(&contract_id, || {
        let key = crate::storage::DataKey::VaultVCCount(owner.clone());
        env.storage().persistent().set(&key, &u32::MAX);
    });

    client.issue(
        &owner,
        &String::from_str(&env, "overflow-vc"),
        &String::from_str(&env, "data"),
        &contract_id,
        &issuer,
        &String::from_str(&env, "did:issuer"),
        &0_i128,
    );
}
