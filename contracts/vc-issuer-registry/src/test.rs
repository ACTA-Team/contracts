#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, Symbol};

use crate::contract::{VcIssuerRegistryContract, VcIssuerRegistryContractClient};

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
// test_add_issuer_happy_path
//   — Verifies add_issuer stores the record, is_issuer_allowed returns true,
//     and get_issuer returns the correct metadata.
// ---------------------------------------------------------------------------
#[test]
fn test_add_issuer_happy_path() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);

    let name = Symbol::new(&e, "TestIssuer");
    let did = Bytes::from_slice(&e, b"did:example:123");
    let url = Bytes::from_slice(&e, b"https://example.com");

    client.add_issuer(&issuer, &Some(name.clone()), &Some(did.clone()), &Some(url.clone()));

    assert!(client.is_issuer_allowed(&issuer));

    let record = client.get_issuer(&issuer);
    assert!(record.allowed);
    assert_eq!(record.name, Some(name));
    assert_eq!(record.did, Some(did));
    assert_eq!(record.url, Some(url));
}

// ---------------------------------------------------------------------------
// test_add_issuer_duplicate_rejected
//   — Second add_issuer with same address must fail with IssuerAlreadyExists.
// ---------------------------------------------------------------------------
#[test]
#[should_panic]
fn test_add_issuer_duplicate_rejected() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);
    client.add_issuer(&issuer, &None, &None, &None);
    client.add_issuer(&issuer, &None, &None, &None); // must panic
}

// ---------------------------------------------------------------------------
// test_remove_issuer_then_disallowed
//   — After removal is_issuer_allowed must return false, and get_issuer
//     must panic with IssuerNotFound.
// ---------------------------------------------------------------------------
#[test]
fn test_remove_issuer_then_disallowed() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);
    client.add_issuer(&issuer, &None, &None, &None);
    assert!(client.is_issuer_allowed(&issuer));

    client.remove_issuer(&issuer);

    // is_issuer_allowed returns false for a removed issuer.
    assert!(!client.is_issuer_allowed(&issuer));

    // get_issuer must fail for a removed issuer.
    let result = client.try_get_issuer(&issuer);
    assert!(result.is_err(), "get_issuer after remove must fail");
}

// ---------------------------------------------------------------------------
// test_remove_issuer_not_found
//   — Removing a non-existent issuer must fail.
// ---------------------------------------------------------------------------
#[test]
#[should_panic]
fn test_remove_issuer_not_found() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);
    client.remove_issuer(&issuer); // must panic
}

// ---------------------------------------------------------------------------
// test_set_metadata_updates_fields
//   — set_issuer_metadata replaces name/did/url, and get_issuer reflects
//     the new values.
// ---------------------------------------------------------------------------
#[test]
fn test_set_metadata_updates_fields() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);

    let name_v1 = Symbol::new(&e, "V1");
    let did_v1 = Bytes::from_slice(&e, b"did:v1");
    let url_v1 = Bytes::from_slice(&e, b"https://v1.example.com");
    client.add_issuer(&issuer, &Some(name_v1), &Some(did_v1), &Some(url_v1));

    // Update metadata
    let name_v2 = Symbol::new(&e, "V2");
    let did_v2 = Bytes::from_slice(&e, b"did:v2");
    let url_v2 = Bytes::from_slice(&e, b"https://v2.example.com");
    client.set_issuer_metadata(&issuer, &Some(name_v2.clone()), &Some(did_v2.clone()), &Some(url_v2.clone()));

    let record = client.get_issuer(&issuer);
    assert_eq!(record.name, Some(name_v2));
    assert_eq!(record.did, Some(did_v2));
    assert_eq!(record.url, Some(url_v2));
    // allowed flag must not change
    assert!(record.allowed);
}

// ---------------------------------------------------------------------------
// test_set_metadata_preserves_allowed_false
//   — set_issuer_metadata must NOT re-enable a disabled issuer.
// ---------------------------------------------------------------------------
#[test]
fn test_set_metadata_preserves_allowed_false() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);
    client.add_issuer(&issuer, &None, &None, &None);

    // Disable
    client.set_issuer_allowed(&issuer, &false);
    assert!(!client.is_issuer_allowed(&issuer));

    // Update metadata — must keep allowed == false
    let name = Symbol::new(&e, "Updated");
    client.set_issuer_metadata(&issuer, &Some(name), &None, &None);

    let record = client.get_issuer(&issuer);
    assert!(!record.allowed, "set_issuer_metadata must not re-enable a disabled issuer");
}

// ---------------------------------------------------------------------------
// test_set_metadata_on_nonexistent_fails
// ---------------------------------------------------------------------------
#[test]
#[should_panic]
fn test_set_metadata_on_nonexistent_fails() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);
    client.set_issuer_metadata(&issuer, &None, &None, &None); // must panic
}

// ---------------------------------------------------------------------------
// test_non_admin_cannot_mutate
//   — A non-admin caller must be rejected when calling add_issuer.
// ---------------------------------------------------------------------------
#[test]
fn test_non_admin_cannot_mutate() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let _non_admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);

    // Clear mocks so no valid auth is provided for add_issuer
    e.mock_auths(&[]);
    let result = client.try_add_issuer(&issuer, &None, &None, &None);

    assert!(result.is_err(), "non-admin call must fail");
}

// ---------------------------------------------------------------------------
// test_set_issuer_allowed_toggle
//   — Toggle allowed flag and verify is_issuer_allowed tracks correctly.
// ---------------------------------------------------------------------------
#[test]
fn test_set_issuer_allowed_toggle() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);
    client.add_issuer(&issuer, &None, &None, &None);
    assert!(client.is_issuer_allowed(&issuer));

    client.set_issuer_allowed(&issuer, &false);
    assert!(!client.is_issuer_allowed(&issuer));

    client.set_issuer_allowed(&issuer, &true);
    assert!(client.is_issuer_allowed(&issuer));
}

// ---------------------------------------------------------------------------
// test_get_issuer_not_found_panics
// ---------------------------------------------------------------------------
#[test]
#[should_panic]
fn test_get_issuer_not_found_panics() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);
    client.get_issuer(&issuer); // must panic
}

// ---------------------------------------------------------------------------
// test_is_issuer_allowed_returns_false_for_unknown
//   — is_issuer_allowed gracefully returns false for an unknown address.
// ---------------------------------------------------------------------------
#[test]
fn test_is_issuer_allowed_returns_false_for_unknown() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let unknown = Address::generate(&e);

    client.initialize(&admin);
    assert!(!client.is_issuer_allowed(&unknown));
}

// ---------------------------------------------------------------------------
// test_metadata_validation_did_too_long
//   — Adding an issuer with a `did` exceeding 256 bytes must fail.
// ---------------------------------------------------------------------------
#[test]
#[should_panic]
fn test_metadata_validation_did_too_long() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);

    let long_did = Bytes::from_slice(&e, &[0u8; 257]);
    client.add_issuer(&issuer, &None, &Some(long_did), &None); // must panic
}

// ---------------------------------------------------------------------------
// test_metadata_validation_url_too_long
//   — Adding an issuer with a `url` exceeding 256 bytes must fail.
// ---------------------------------------------------------------------------
#[test]
#[should_panic]
fn test_metadata_validation_url_too_long() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);

    let long_url = Bytes::from_slice(&e, &[0u8; 257]);
    client.add_issuer(&issuer, &None, &None, &Some(long_url)); // must panic
}

// ---------------------------------------------------------------------------
// test_metadata_validation_boundary_ok
//   — 256-byte did/url should be accepted (boundary check).
// ---------------------------------------------------------------------------
#[test]
fn test_metadata_validation_boundary_ok() {
    let (e, client) = setup();
    let admin = Address::generate(&e);
    let issuer = Address::generate(&e);

    client.initialize(&admin);

    let max_did = Bytes::from_slice(&e, &[b'x'; 256]);
    let max_url = Bytes::from_slice(&e, &[b'y'; 256]);
    client.add_issuer(&issuer, &None, &Some(max_did), &Some(max_url));

    assert!(client.is_issuer_allowed(&issuer));
}
