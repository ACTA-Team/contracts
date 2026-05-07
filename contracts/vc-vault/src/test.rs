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
    assert_eq!(client.list_vc_ids(&owner).len(), 0);
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
    assert_eq!(client.list_vc_ids(&owner).len(), 1);
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
    assert_eq!(client.list_vc_ids(&owner).len(), 1);
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
    assert_eq!(client.list_vc_ids(&owner).len(), 2);
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
    assert_eq!(client.list_vc_ids(&owner).len(), 0);
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
    assert_eq!(client.list_vc_ids(&owner).len(), 0);
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
    assert_eq!(client.list_vc_ids(&owner).len(), 0);
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
