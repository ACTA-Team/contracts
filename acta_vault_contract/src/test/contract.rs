use super::setup::{get_vc_setup, VCVaultContractTest};
use crate::test::setup::VaultContractTest;
use soroban_sdk::{testutils::Address as _, vec, Address, String};

#[test]
fn test_initialize() {
    let VaultContractTest {
        env: _env,
        owner,
        issuer: _issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();
    contract.initialize(&owner, &did_uri);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_initialize_an_already_initialized_contract() {
    let VaultContractTest {
        env: _,
        owner,
        issuer: _issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();
    contract.initialize(&owner, &did_uri);
    contract.initialize(&owner, &did_uri);
}

#[test]
fn test_authorize_issuer() {
    let VaultContractTest {
        env: _env,
        owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();
    contract.initialize(&owner, &did_uri);
    contract.authorize_issuer(&owner, &issuer);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_authorize_issuer_with_already_authorized_issuer() {
    let VaultContractTest {
        env: _,
        owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();
    contract.initialize(&owner, &did_uri);
    contract.authorize_issuer(&owner, &issuer);
    contract.authorize_issuer(&owner, &issuer);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_authorize_issuer_with_revoked_vault() {
    let VaultContractTest {
        env: _,
        owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();
    contract.initialize(&owner, &did_uri);
    contract.revoke_vault(&owner);
    contract.authorize_issuer(&owner, &issuer);
}

#[test]
fn test_authorize_issuers() {
    let VaultContractTest {
        env,
        owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();
    let issuers = vec![&env, issuer.clone()];
    contract.initialize(&owner, &did_uri);
    contract.authorize_issuers(&owner, &issuers);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_authorize_issuers_with_revoked_vault() {
    let VaultContractTest {
        env,
        owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();
    let issuers = vec![&env, issuer.clone()];
    contract.initialize(&owner, &did_uri);
    contract.revoke_vault(&owner);
    contract.authorize_issuers(&owner, &issuers);
}

#[test]
fn test_revoke_issuer() {
    let VaultContractTest {
        env: _env,
        owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();
    contract.initialize(&owner, &did_uri);
    contract.authorize_issuer(&owner, &issuer);
    contract.revoke_issuer(&owner, &issuer);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_revoke_issuer_when_issuer_is_not_found() {
    let VaultContractTest {
        env,
        owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();
    contract.initialize(&owner, &did_uri);
    contract.authorize_issuer(&owner, &issuer);

    let invalid_issuer = Address::generate(&env);
    contract.revoke_issuer(&owner, &invalid_issuer);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_revoke_issuer_with_revoked_vault() {
    let VaultContractTest {
        env: _,
        owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();
    contract.initialize(&owner, &did_uri);
    contract.revoke_vault(&owner);
    contract.revoke_issuer(&owner, &issuer);
}

#[test]
fn test_store_vc() {
    let VaultContractTest {
        env,
        owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();

    let VCVaultContractTest {
        vc_id,
        vc_data,
        issuance_contract_address,
        issuer_did,
    } = get_vc_setup(&env);

    contract.initialize(&owner, &did_uri);
    contract.authorize_issuer(&owner, &issuer);
    contract.store_vc(
        &owner,
        &vc_id,
        &vc_data,
        &issuer,
        &issuer_did,
        &issuance_contract_address,
    )
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_store_vc_with_empty_issuers() {
    let VaultContractTest {
        env,
        owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();

    let VCVaultContractTest {
        vc_id,
        vc_data,
        issuance_contract_address,
        issuer_did,
    } = get_vc_setup(&env);

    contract.initialize(&owner, &did_uri);
    contract.store_vc(
        &owner,
        &vc_id,
        &vc_data,
        &issuer,
        &issuer_did,
        &issuance_contract_address,
    )
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_store_vc_with_issuer_not_found() {
    let VaultContractTest {
        env,
        owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();

    let invalid_issuer = Address::generate(&env);

    let VCVaultContractTest {
        vc_id,
        vc_data,
        issuance_contract_address,
        issuer_did,
    } = get_vc_setup(&env);

    contract.initialize(&owner, &did_uri);
    contract.authorize_issuer(&owner, &issuer);
    contract.store_vc(
        &owner,
        &vc_id,
        &vc_data,
        &invalid_issuer,
        &issuer_did,
        &issuance_contract_address,
    )
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_store_vc_with_revoked_issuer() {
    let VaultContractTest {
        env,
        owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();

    let VCVaultContractTest {
        vc_id,
        vc_data,
        issuance_contract_address,
        issuer_did,
    } = get_vc_setup(&env);

    contract.initialize(&owner, &did_uri);
    contract.authorize_issuer(&owner, &issuer);
    contract.revoke_issuer(&owner, &issuer);

    contract.store_vc(
        &owner,
        &vc_id,
        &vc_data,
        &issuer,
        &issuer_did,
        &issuance_contract_address,
    )
}

#[test]
fn test_revoke_vault() {
    let VaultContractTest {
        env: _,
        owner,
        issuer: _,
        did_uri,
        contract,
    } = VaultContractTest::setup();

    contract.initialize(&owner, &did_uri);
    contract.revoke_vault(&owner);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_migrate_should_fail_without_vcs() {
    let VaultContractTest {
        env: _,
        owner,
        issuer: _,
        did_uri,
        contract,
    } = VaultContractTest::setup();

    contract.initialize(&owner, &did_uri);
    contract.migrate(&owner);
}

#[test]
fn test_set_admin() {
    let VaultContractTest {
        env,
        owner,
        issuer: _issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();

    contract.initialize(&owner, &did_uri);

    let new_admin = Address::generate(&env);

    contract.set_admin(&owner, &new_admin);
}

#[test]
fn test_version() {
    let VaultContractTest {
        env,
        owner,
        issuer: _issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();

    contract.initialize(&owner, &did_uri);
    let pkg_version = env!("CARGO_PKG_VERSION");
    let expected_version = String::from_str(&env, pkg_version);
    assert_eq!(contract.version(), expected_version)
}

#[test]
fn test_push_vc_moves_record_and_ids() {
    let VaultContractTest {
        env,
        owner: from_owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();

    // Segundo owner (destino)
    let to_owner = Address::generate(&env);

    // Inicializa ambos vaults
    contract.initialize(&from_owner, &did_uri);
    contract.initialize(&to_owner, &did_uri);

    // Autoriza al issuer solo en el origen
    contract.authorize_issuer(&from_owner, &issuer);

    // Crea VC en el vault de origen
    let VCVaultContractTest {
        vc_id,
        vc_data,
        issuance_contract_address,
        issuer_did,
    } = get_vc_setup(&env);

    contract.store_vc(
        &from_owner,
        &vc_id,
        &vc_data,
        &issuer,
        &issuer_did,
        &issuance_contract_address,
    );

    // Empuja al destino
    contract.push(&from_owner, &to_owner, &vc_id, &issuer);

    // Verifica que se movió
    let from_ids = contract.list_vc_ids(&from_owner);
    assert_eq!(from_ids.len(), 0);

    let to_ids = contract.list_vc_ids(&to_owner);
    assert!(to_ids.contains(vc_id.clone()));

    let to_vc = contract.get_vc(&to_owner, &vc_id);
    assert!(to_vc.is_some());

    let from_vc = contract.get_vc(&from_owner, &vc_id);
    assert!(from_vc.is_none());
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")]
fn test_push_vc_not_found_should_panic() {
    let VaultContractTest {
        env,
        owner: from_owner,
        issuer,
        did_uri,
        contract,
    } = VaultContractTest::setup();

    let to_owner = Address::generate(&env);

    contract.initialize(&from_owner, &did_uri);
    contract.initialize(&to_owner, &did_uri);
    contract.authorize_issuer(&from_owner, &issuer);

    let missing_vc_id = String::from_str(&env, "missing-vc");
    contract.push(&from_owner, &to_owner, &missing_vc_id, &issuer);
}
