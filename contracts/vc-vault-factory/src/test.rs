#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env, String,
};

use crate::{storage::VaultInitMeta, VaultFactoryContract, VaultFactoryContractClient};

mod vc_vault {
    // Use the unoptimized WASM in tests — the optimized variant requires the
    // `stellar` CLI (wasm-opt) which adds heavy CI dependencies. Soroban's
    // sandbox executes either binary identically.
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/vc_vault_contract.wasm"
    );
}

fn setup(e: &Env) -> (Address, Address, VaultFactoryContractClient<'_>) {
    e.mock_all_auths_allowing_non_root_auth();
    let vault_wasm_hash = e.deployer().upload_contract_wasm(vc_vault::WASM);
    let admin = Address::generate(e);
    let meta = VaultInitMeta {
        vault_hash: vault_wasm_hash,
        contract_admin: admin.clone(),
    };
    let factory_id = e.register(VaultFactoryContract, (meta,));
    let client = VaultFactoryContractClient::new(e, &factory_id);
    (admin, factory_id, client)
}

#[test]
fn test_factory_admin_two_step_transfer() {
    let e = Env::default();
    let (admin, factory_id, client) = setup(&e);
    let new_admin = Address::generate(&e);

    client.nominate_admin(&new_admin);
    client.accept_admin();

    assert_eq!(client.get_admin(), new_admin);
    let _ = (admin, factory_id);
}

#[test]
#[should_panic]
fn test_factory_accept_admin_without_nomination_panics() {
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);
    client.accept_admin();
}

#[test]
fn test_is_vault_returns_false_for_unknown() {
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);
    let random = Address::generate(&e);
    assert!(!client.is_vault(&random));
}

#[test]
fn test_deploy_registers_vault() {
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);

    let owner = Address::generate(&e);
    let did_uri = String::from_str(&e, "did:pkh:stellar:testnet:OWNER");
    let salt = BytesN::random(&e);

    let vault_addr = client.deploy(&owner, &did_uri, &salt);
    assert!(client.is_vault(&vault_addr));
}

#[test]
fn test_two_owners_get_different_vault_addresses() {
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);

    let owner1 = Address::generate(&e);
    let owner2 = Address::generate(&e);
    let did_uri = String::from_str(&e, "did:test");
    let salt = BytesN::random(&e);

    let vault1 = client.deploy(&owner1, &did_uri, &salt);
    // Same salt, different owner → different address (keccak mixes in owner bytes).
    let vault2 = client.deploy(&owner2, &did_uri, &salt);

    assert_ne!(vault1, vault2);
    assert!(client.is_vault(&vault1));
    assert!(client.is_vault(&vault2));
}

#[test]
fn test_same_owner_different_salts_get_different_addresses() {
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);

    let owner = Address::generate(&e);
    let did_uri = String::from_str(&e, "did:test");

    let vault1 = client.deploy(&owner, &did_uri, &BytesN::random(&e));
    let vault2 = client.deploy(&owner, &did_uri, &BytesN::random(&e));

    assert_ne!(vault1, vault2);
}

#[test]
fn test_deploy_integration_issue_and_verify() {
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);

    let owner = Address::generate(&e);
    let did_uri = String::from_str(&e, "did:pkh:stellar:testnet:OWNER");
    let salt = BytesN::random(&e);

    let vault_addr = client.deploy(&owner, &did_uri, &salt);
    let vault = vc_vault::Client::new(&e, &vault_addr);

    let issuer = Address::generate(&e);
    vault.authorize_issuer(&issuer);

    let vc_id = String::from_str(&e, "vc-1");
    let vc_data = String::from_str(&e, "<data>");
    let issuer_did = String::from_str(&e, "did:issuer");
    let returned = vault.issue(&vc_id, &vc_data, &vault_addr, &issuer, &issuer_did, &0_i128);
    assert_eq!(returned, vc_id);

    assert_eq!(vault.verify_vc(&vc_id), vc_vault::VCStatus::Valid);
}

#[test]
fn test_deploy_emits_event() {
    use soroban_sdk::testutils::Events;
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);

    let owner = Address::generate(&e);
    let did_uri = String::from_str(&e, "did:test");
    let salt = BytesN::random(&e);

    client.deploy(&owner, &did_uri, &salt);

    // deploy_v2 triggers the vault constructor (2 events: contract_initialized + vault_created)
    // plus the factory itself emits 1 "deploy" event = 3 total.
    assert_eq!(e.events().all().len(), 3);
}

#[test]
fn test_deploy_sponsored_registers_vault_for_owner() {
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);

    let deployer = Address::generate(&e);
    let owner = Address::generate(&e);
    let did_uri = String::from_str(&e, "did:pkh:stellar:testnet:OWNER");
    let salt = BytesN::random(&e);

    let vault_addr = client.deploy_sponsored(&deployer, &owner, &did_uri, &salt);
    assert!(client.is_vault(&vault_addr));
}

#[test]
fn test_deploy_sponsored_vault_belongs_to_owner_not_deployer() {
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);

    let deployer = Address::generate(&e);
    let owner = Address::generate(&e);
    let did_uri = String::from_str(&e, "did:test");
    let salt = BytesN::random(&e);

    let vault_addr = client.deploy_sponsored(&deployer, &owner, &did_uri, &salt);
    let vault = vc_vault::Client::new(&e, &vault_addr);

    let issuer = Address::generate(&e);
    vault.authorize_issuer(&issuer);

    let vc_id = String::from_str(&e, "vc-1");
    let vc_data = String::from_str(&e, "<data>");
    let issuer_did = String::from_str(&e, "did:issuer");
    vault.issue(&vc_id, &vc_data, &vault_addr, &issuer, &issuer_did, &0_i128);

    assert_eq!(vault.verify_vc(&vc_id), vc_vault::VCStatus::Valid);
}

/// Both entrypoints derive the vault address from (owner, salt), so two
/// different owners reusing the same user salt land on different addresses —
/// whether deployed via `deploy` or `deploy_sponsored`. (The same-owner +
/// same-salt collapse to one address can't be asserted here because a second
/// deploy of the same pair panics with "already exists".)
#[test]
fn test_deploy_and_deploy_sponsored_different_owners_get_different_addresses() {
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);

    let owner = Address::generate(&e);
    let did_uri = String::from_str(&e, "did:test");
    let salt = BytesN::random(&e);

    let addr_normal = client.deploy(&owner, &did_uri, &salt);
    assert!(client.is_vault(&addr_normal));

    let deployer = Address::generate(&e);
    let owner2 = Address::generate(&e);
    let addr_sponsored = client.deploy_sponsored(&deployer, &owner2, &did_uri, &salt);
    assert!(client.is_vault(&addr_sponsored));
    assert_ne!(addr_normal, addr_sponsored);
}

#[test]
fn test_deploy_sponsored_emits_event() {
    use soroban_sdk::testutils::Events;
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);

    let deployer = Address::generate(&e);
    let owner = Address::generate(&e);
    let did_uri = String::from_str(&e, "did:test");
    let salt = BytesN::random(&e);

    client.deploy_sponsored(&deployer, &owner, &did_uri, &salt);

    assert_eq!(e.events().all().len(), 3);
}

#[test]
fn test_push_transfers_vc_between_vaults() {
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);

    let owner_a = Address::generate(&e);
    let owner_b = Address::generate(&e);
    let did_uri = String::from_str(&e, "did:test");

    let vault_a = client.deploy(&owner_a, &did_uri, &BytesN::random(&e));
    let vault_b = client.deploy(&owner_b, &did_uri, &BytesN::random(&e));

    let vault_a_client = vc_vault::Client::new(&e, &vault_a);
    let vault_b_client = vc_vault::Client::new(&e, &vault_b);

    let issuer = Address::generate(&e);
    let vc_id = String::from_str(&e, "vc-1");
    let vc_data = String::from_str(&e, "<data>");
    let issuer_did = String::from_str(&e, "did:issuer");

    vault_a_client.issue(&vc_id, &vc_data, &vault_a, &issuer, &issuer_did, &0_i128);
    assert_eq!(vault_a_client.vc_count(), 1);
    assert_eq!(vault_b_client.vc_count(), 0);

    vault_a_client.push(&vc_id, &vault_b);

    assert_eq!(vault_a_client.vc_count(), 0);
    assert_eq!(vault_b_client.vc_count(), 1);
    assert_eq!(vault_b_client.verify_vc(&vc_id), vc_vault::VCStatus::Valid);
}

#[test]
fn test_push_removes_vc_from_source() {
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);

    let owner_a = Address::generate(&e);
    let owner_b = Address::generate(&e);
    let did_uri = String::from_str(&e, "did:test");

    let vault_a = client.deploy(&owner_a, &did_uri, &BytesN::random(&e));
    let vault_b = client.deploy(&owner_b, &did_uri, &BytesN::random(&e));

    let vault_a_client = vc_vault::Client::new(&e, &vault_a);

    let issuer = Address::generate(&e);
    let vc_id = String::from_str(&e, "vc-1");
    vault_a_client.issue(&vc_id, &String::from_str(&e, "<data>"), &vault_a, &issuer, &String::from_str(&e, "did:issuer"), &0_i128);

    vault_a_client.push(&vc_id, &vault_b);

    assert!(vault_a_client.get_vc(&vc_id).is_none());
    assert_eq!(vault_a_client.verify_vc(&vc_id), vc_vault::VCStatus::Invalid);
}

#[test]
#[should_panic]
fn test_receive_push_from_non_vault_panics() {
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);

    let owner = Address::generate(&e);
    let did_uri = String::from_str(&e, "did:test");
    let vault_addr = client.deploy(&owner, &did_uri, &BytesN::random(&e));
    let vault_client = vc_vault::Client::new(&e, &vault_addr);

    let fake_vault = Address::generate(&e);
    vault_client.receive_push(
        &fake_vault,
        &String::from_str(&e, "vc-1"),
        &String::from_str(&e, "<data>"),
        &String::from_str(&e, "did:issuer"),
    );
}

/// Regression for the revocation-bypass via push: a VC revoked in the
/// destination vault must not be silently restored to Valid by a later
/// `push` reusing the same vc_id. The duplicate guard in `receive_push`
/// checks the persisted Revoked status, not just the (removed) index.
#[test]
#[should_panic]
fn test_push_cannot_revive_revoked_vc_in_destination() {
    let e = Env::default();
    let (_admin, _factory_id, client) = setup(&e);

    let owner_a = Address::generate(&e);
    let owner_b = Address::generate(&e);
    let did_uri = String::from_str(&e, "did:test");

    let vault_a = client.deploy(&owner_a, &did_uri, &BytesN::random(&e));
    let vault_b = client.deploy(&owner_b, &did_uri, &BytesN::random(&e));

    let vault_a_client = vc_vault::Client::new(&e, &vault_a);
    let vault_b_client = vc_vault::Client::new(&e, &vault_b);

    let issuer = Address::generate(&e);
    let vc_id = String::from_str(&e, "vc-1");
    let vc_data = String::from_str(&e, "<data>");
    let issuer_did = String::from_str(&e, "did:issuer");

    // Destination vault B issues then revokes the credential — index entry is
    // dropped but the Revoked status persists.
    vault_b_client.issue(&vc_id, &vc_data, &vault_b, &issuer, &issuer_did, &0_i128);
    vault_b_client.revoke(&vc_id, &String::from_str(&e, "2026-06-03"));
    assert!(matches!(
        vault_b_client.verify_vc(&vc_id),
        vc_vault::VCStatus::Revoked(_)
    ));

    // Source vault A issues the same vc_id (allowed — independent vault) and
    // pushes to B. This must panic with VCAlreadyExists, not overwrite the
    // revoked status.
    vault_a_client.issue(&vc_id, &vc_data, &vault_a, &issuer, &issuer_did, &0_i128);
    vault_a_client.push(&vc_id, &vault_b);
}
