#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::contract::{VcIssuerRegistryContract, VcIssuerRegistryContractClient};
use crate::error::ContractError;

fn setup() -> (Env, VcIssuerRegistryContractClient<'static>) {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(VcIssuerRegistryContract, ());
    let client = VcIssuerRegistryContractClient::new(&e, &contract_id);
    (e, client)
}

// ---------------------------------------------------------------------------
// test_initialize_sets_admin
// ---------------------------------------------------------------------------
#[test]
fn test_initialize_sets_admin() {
    let (e, client) = setup();
    let admin = Address::generate(&e);

    client.initialize(&admin);

    assert_eq!(client.admin(), admin);
}

// ---------------------------------------------------------------------------
// test_initialize_only_once
// ---------------------------------------------------------------------------
#[test]
#[should_panic]
fn test_initialize_only_once() {
    let (e, client) = setup();
    let admin = Address::generate(&e);

    client.initialize(&admin);
    client.initialize(&admin); // must panic
}

// ---------------------------------------------------------------------------
// test_admin_gate_rejects_non_admin
// ---------------------------------------------------------------------------
#[test]
fn test_admin_gate_rejects_non_admin() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let non_admin = Address::generate(&e);

    // Initialize so admin is stored.
    client.initialize(&admin);

    // Clear mocks so no valid auth is provided for add_issuer
    e.mock_auths(&[]);
    let result = client.try_add_issuer(&non_admin, &None, &None, &None);

    assert!(result.is_err(), "non-admin call must fail");
}

// ---------------------------------------------------------------------------
// Additional happy-path smoke tests
// ---------------------------------------------------------------------------

#[test]
fn test_add_and_get_issuer() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);
    client.add_issuer(&issuer, &None, &None, &None);

    let record = client.get_issuer(&issuer);
    assert!(record.allowed);
}

#[test]
#[should_panic]
fn test_add_issuer_duplicate_panics() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);
    client.add_issuer(&issuer, &None, &None, &None);
    client.add_issuer(&issuer, &None, &None, &None); // must panic
}

#[test]
fn test_set_issuer_allowed_false() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);
    client.add_issuer(&issuer, &None, &None, &None);
    assert!(client.is_allowed(&issuer));

    client.set_issuer_allowed(&issuer, &false);
    assert!(!client.is_allowed(&issuer));
}

#[test]
#[should_panic]
fn test_get_issuer_not_found_panics() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);
    client.get_issuer(&issuer); // must panic
}
