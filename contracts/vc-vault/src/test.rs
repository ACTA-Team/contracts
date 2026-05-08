//! Unit tests for VC Vault contract.

use crate::contract::{VcVaultContract, VcVaultContractClient};
use crate::model::VCStatus;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
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

// --- O(1) index tests (issue #20) ---

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

// --- Pagination tests (issue #21) ---

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

// --- migrate_vc_index tests (issue #22) ---

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
