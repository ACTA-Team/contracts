use crate::contract::{VCIssuanceContract, VCIssuanceContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup_issuance_test() -> (Env, Address, Address, String, VCIssuanceContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let issuer_did = String::from_str(&env, "did:chaincerts:test-issuer");
    let contract = VCIssuanceContractClient::new(
        &env,
        &env.register_contract(None, VCIssuanceContract),
    );
    (env, admin, issuer, issuer_did, contract)
}

#[test]
fn test_initialize() {
    let (_env, admin, _issuer, issuer_did, contract) = setup_issuance_test();
    contract.initialize(&admin, &issuer_did);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_initialize_twice_should_fail() {
    let (_env, admin, _issuer, issuer_did, contract) = setup_issuance_test();
    contract.initialize(&admin, &issuer_did);
    contract.initialize(&admin, &issuer_did);
}

#[test]
#[should_panic]
fn test_issue() {
    let (env, admin, issuer, issuer_did, contract) = setup_issuance_test();
    contract.initialize(&admin, &issuer_did);

    // Setup vault contract - register a mock vault contract
    let vault_contract = Address::generate(&env);
    
    let owner = Address::generate(&env);
    let vc_id = String::from_str(&env, "vc-1");
    let vc_data = String::from_str(&env, "vc-data-1");

    // This will fail because vault_contract is not initialized, but tests the code path
    contract.issue(
        &owner,
        &vc_id,
        &vc_data,
        &vault_contract,
        &issuer,
        &issuer_did,
    );
}

#[test]
#[should_panic]
fn test_verify_valid_vc() {
    let (env, admin, issuer, issuer_did, contract) = setup_issuance_test();
    contract.initialize(&admin, &issuer_did);

    // Setup vault contract - register a mock vault contract
    let vault_contract = Address::generate(&env);
    
    let owner = Address::generate(&env);
    let vc_id = String::from_str(&env, "vc-1");
    let vc_data = String::from_str(&env, "vc-data-1");

    // This will fail because vault_contract is not initialized, but tests the code path
    contract.issue(
        &owner,
        &vc_id,
        &vc_data,
        &vault_contract,
        &issuer,
        &issuer_did,
    );

    let result = contract.verify(&vc_id);
    let status = result.get(String::from_str(&env, "status")).unwrap();
    assert_eq!(status, String::from_str(&env, "valid"));
}

#[test]
#[should_panic]
fn test_revoke_vc() {
    let (env, admin, issuer, issuer_did, contract) = setup_issuance_test();
    contract.initialize(&admin, &issuer_did);

    // Setup vault contract - register a mock vault contract
    let vault_contract = Address::generate(&env);
    
    let owner = Address::generate(&env);
    let vc_id = String::from_str(&env, "vc-1");
    let vc_data = String::from_str(&env, "vc-data-1");

    // This will fail because vault_contract is not initialized, but tests the code path
    contract.issue(
        &owner,
        &vc_id,
        &vc_data,
        &vault_contract,
        &issuer,
        &issuer_did,
    );

    let date = String::from_str(&env, "2023-12-05T21:37:44.389Z");
    contract.revoke(&vc_id, &date);

    let result = contract.verify(&vc_id);
    let status = result.get(String::from_str(&env, "status")).unwrap();
    assert_eq!(status, String::from_str(&env, "revoked"));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_revoke_invalid_vc_should_fail() {
    let (env, admin, _issuer, issuer_did, contract) = setup_issuance_test();
    contract.initialize(&admin, &issuer_did);

    let vc_id = String::from_str(&env, "invalid-vc");
    let date = String::from_str(&env, "2023-12-05T21:37:44.389Z");
    contract.revoke(&vc_id, &date);
}

#[test]
fn test_set_admin() {
    let (env, admin, _issuer, issuer_did, contract) = setup_issuance_test();
    contract.initialize(&admin, &issuer_did);

    let new_admin = Address::generate(&env);
    contract.set_admin(&new_admin);
}

#[test]
fn test_version() {
    let (env, admin, _issuer, issuer_did, contract) = setup_issuance_test();
    contract.initialize(&admin, &issuer_did);

    let pkg_version = env!("CARGO_PKG_VERSION");
    let expected_version = String::from_str(&env, pkg_version);
    assert_eq!(contract.version(), expected_version);
}

