use crate::contract::{VaultContract, VaultContractClient};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

fn setup_vault_test() -> (Env, Address, Address, String, VaultContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let issuer = Address::generate(&env);
    let did_uri = String::from_str(
        &env,
        "did:pkh:stellar:testnet:GCUETKXJ2YNVADOF5SZBBZA6M3O6HEOXN4GRJZUW2MBRS2UKXZM37QDE",
    );
    let contract = VaultContractClient::new(&env, &env.register_contract(None, VaultContract));
    (env, owner, issuer, did_uri, contract)
}

#[test]
fn test_initialize() {
    let (_env, owner, _issuer, did_uri, contract) = setup_vault_test();
    contract.initialize(&owner, &did_uri);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_initialize_twice_should_fail() {
    let (_env, owner, _issuer, did_uri, contract) = setup_vault_test();
    contract.initialize(&owner, &did_uri);
    contract.initialize(&owner, &did_uri);
}

#[test]
fn test_authorize_issuer() {
    let (_env, owner, issuer, did_uri, contract) = setup_vault_test();
    contract.initialize(&owner, &did_uri);
    contract.authorize_issuer(&owner, &issuer);
}

#[test]
fn test_authorize_issuers() {
    let (env, owner, issuer, did_uri, contract) = setup_vault_test();
    contract.initialize(&owner, &did_uri);
    let issuers = vec![&env, issuer.clone()];
    contract.authorize_issuers(&owner, &issuers);
}

#[test]
fn test_revoke_issuer() {
    let (_env, owner, issuer, did_uri, contract) = setup_vault_test();
    contract.initialize(&owner, &did_uri);
    contract.authorize_issuer(&owner, &issuer);
    contract.revoke_issuer(&owner, &issuer);
}

#[test]
fn test_store_vc() {
    let (env, owner, issuer, did_uri, contract) = setup_vault_test();
    contract.initialize(&owner, &did_uri);
    contract.authorize_issuer(&owner, &issuer);

    let vc_id = String::from_str(&env, "vc-1");
    let vc_data = String::from_str(&env, "vc-data-1");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    let issuance_contract = Address::generate(&env);

    contract.store_vc(
        &owner,
        &vc_id,
        &vc_data,
        &issuer,
        &issuer_did,
        &issuance_contract,
    );
}

#[test]
fn test_list_vc_ids() {
    let (env, owner, issuer, did_uri, contract) = setup_vault_test();
    contract.initialize(&owner, &did_uri);
    contract.authorize_issuer(&owner, &issuer);

    let vc_id = String::from_str(&env, "vc-1");
    let vc_data = String::from_str(&env, "vc-data-1");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    let issuance_contract = Address::generate(&env);

    contract.store_vc(
        &owner,
        &vc_id,
        &vc_data,
        &issuer,
        &issuer_did,
        &issuance_contract,
    );

    let vc_ids = contract.list_vc_ids(&owner);
    assert_eq!(vc_ids.len(), 1);
    assert_eq!(vc_ids.get(0).unwrap(), vc_id);
}

#[test]
fn test_get_vc() {
    let (env, owner, issuer, did_uri, contract) = setup_vault_test();
    contract.initialize(&owner, &did_uri);
    contract.authorize_issuer(&owner, &issuer);

    let vc_id = String::from_str(&env, "vc-1");
    let vc_data = String::from_str(&env, "vc-data-1");
    let issuer_did = String::from_str(&env, "did:pkh:stellar:testnet:ISSUER");
    let issuance_contract = Address::generate(&env);

    contract.store_vc(
        &owner,
        &vc_id,
        &vc_data,
        &issuer,
        &issuer_did,
        &issuance_contract,
    );

    let vc = contract.get_vc(&owner, &vc_id);
    assert!(vc.is_some());
    let vc_unwrapped = vc.unwrap();
    assert_eq!(vc_unwrapped.data, vc_data);
}

#[test]
fn test_revoke_vault() {
    let (_env, owner, _issuer, did_uri, contract) = setup_vault_test();
    contract.initialize(&owner, &did_uri);
    contract.revoke_vault(&owner);
}

#[test]
fn test_set_admin() {
    let (env, owner, _issuer, did_uri, contract) = setup_vault_test();
    contract.initialize(&owner, &did_uri);

    let new_admin = Address::generate(&env);
    contract.set_admin(&owner, &new_admin);
}

#[test]
fn test_version() {
    let (env, owner, _issuer, did_uri, contract) = setup_vault_test();
    contract.initialize(&owner, &did_uri);
    let pkg_version = env!("CARGO_PKG_VERSION");
    let expected_version = String::from_str(&env, pkg_version);
    assert_eq!(contract.version(), expected_version);
}

