use crate::contract::{ActaContract, ActaContractClient};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

fn setup() -> (Env, Address, Address, Address, ActaContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let contract_id = env.register_contract(None, ActaContract);
    let client = ActaContractClient::new(&env, &contract_id);
    (env, admin, issuer, contract_id, client)
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
fn test_vault_authorize_and_store_and_list_and_get() {
    let (env, admin, issuer, contract_id, client) = setup();

    client.initialize(&admin, &String::from_str(&env, "did:acta:default"));

    let owner = Address::generate(&env);
    client.create_vault(&owner, &String::from_str(&env, "did:pkh:stellar:testnet:OWNER"));

    client.authorize_issuer(&owner, &issuer);

    let vc_id = String::from_str(&env, "vc-1");
    let vc_data = String::from_str(&env, "<ciphertext>");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");

    // Issue (stores payload + status)
    client.issue(
        &owner,
        &vc_id,
        &vc_data,
        &contract_id,
        &issuer,
        &issuer_did,
    );

    let ids = client.list_vc_ids(&owner);
    assert_eq!(ids.len(), 1);

    let vc = client.get_vc(&owner, &vc_id).unwrap();
    assert_eq!(vc.data, vc_data);
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

    client.issue(
        &owner,
        &vc_id,
        &vc_data,
        &contract_id,
        &issuer,
        &issuer_did,
    );

    let m = client.verify_vc(&owner, &vc_id);
    let status = m.get(String::from_str(&env, "status")).unwrap();
    assert_eq!(status, String::from_str(&env, "valid"));

    let date = String::from_str(&env, "2025-12-18T00:00:00Z");
    client.revoke(&vc_id, &date);

    let m2 = client.verify_vc(&owner, &vc_id);
    let status2 = m2.get(String::from_str(&env, "status")).unwrap();
    assert_eq!(status2, String::from_str(&env, "revoked"));
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

    client.issue(
        &from_owner,
        &vc_id,
        &vc_data,
        &contract_id,
        &issuer,
        &issuer_did,
    );

    client.push(&from_owner, &to_owner, &vc_id, &issuer);

    assert!(client.get_vc(&from_owner, &vc_id).is_none());
    assert!(client.get_vc(&to_owner, &vc_id).is_some());
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
