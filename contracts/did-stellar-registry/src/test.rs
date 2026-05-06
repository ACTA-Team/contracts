//! Unit tests for `did-stellar-registry`.
//!
//! Test classes:
//!  - register / get
//!  - update with optimistic concurrency
//!  - transfer_controller
//!  - deactivate (tombstone)
//!  - authorization enforcement (controller.require_auth)
//!  - boundary validation (key counts, lengths, URL formats, duplicate keys)
//!  - events (smoke check that mutations emit something)
//!  - normative test vectors (matches `docs/did-spec/test-vectors/vectors.json`)

#![cfg(test)]

extern crate std;

use crate::contract::{DidStellarRegistry, DidStellarRegistryClient};
use crate::errors::RegistryError;
use crate::events::{DidControllerTransferred, DidDeactivated, DidRegistered, DidUpdated};
use crate::model::{DidKey, DidRecord, DidService};
use soroban_sdk::{
    testutils::{Address as _, BytesN as _, Events},
    vec, Address, BytesN, Env, Event, IntoVal, String, Vec,
};

// --- helpers ---------------------------------------------------------------

/// Returns `(env, controller, did_id, contract_id, client)`.
fn setup() -> (
    Env,
    Address,
    BytesN<16>,
    Address,
    DidStellarRegistryClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();
    let controller = Address::generate(&env);
    let did_id = BytesN::<16>::random(&env);
    let contract_id = env.register(DidStellarRegistry, ());
    let client = DidStellarRegistryClient::new(&env, &contract_id);
    (env, controller, did_id, contract_id, client)
}

fn s(e: &Env, v: &str) -> String {
    String::from_str(e, v)
}

fn key(e: &Env, multibase: &str) -> DidKey {
    DidKey {
        public_key_multibase: s(e, multibase),
    }
}

fn empty_keys(e: &Env) -> Vec<DidKey> {
    Vec::<DidKey>::new(e)
}

fn empty_services(e: &Env) -> Vec<DidService> {
    Vec::<DidService>::new(e)
}

fn minimal_record(e: &Env, controller: &Address) -> DidRecord {
    let mut auth = empty_keys(e);
    auth.push_back(key(e, "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doY"));
    DidRecord {
        controller: controller.clone(),
        authentication: auth,
        assertion_method: empty_keys(e),
        key_agreement: empty_keys(e),
        services: empty_services(e),
        metadata_uri: None,
        metadata_hash: None,
        // These four fields are owned by the contract and overwritten on
        // register/update.
        version: 0,
        created_ledger: 0,
        updated_ledger: 0,
        deactivated: false,
    }
}

// --- register / get --------------------------------------------------------

#[test]
fn test_register_basic() {
    let (env, controller, did_id, _id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));

    let r = client.get(&did_id).unwrap();
    assert_eq!(r.controller, controller);
    assert_eq!(r.version, 1);
    assert_eq!(r.deactivated, false);
    assert_eq!(r.authentication.len(), 1);
    // created_ledger == updated_ledger right after register.
    assert_eq!(r.created_ledger, r.updated_ledger);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")] // DidAlreadyExists
fn test_register_duplicate() {
    let (env, controller, did_id, _id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));
    client.register(&did_id, &minimal_record(&env, &controller));
}

#[test]
fn test_get_nonexistent() {
    let (env, _ctrl, _did, _id, client) = setup();
    let other = BytesN::<16>::random(&env);
    assert!(client.get(&other).is_none());
}

// --- update ----------------------------------------------------------------

#[test]
fn test_update_success() {
    let (env, controller, did_id, _id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));
    let v1 = client.get(&did_id).unwrap();

    // Add an assertion key.
    let mut next = v1.clone();
    let mut assert_keys = empty_keys(&env);
    assert_keys.push_back(key(&env, "z6Mkff3F4VMDGbMbMtgRyXMrgr7qyxaKsPo7QEPQ2AkNrx2X"));
    next.assertion_method = assert_keys;

    client.update(&did_id, &v1.version, &next);

    let v2 = client.get(&did_id).unwrap();
    assert_eq!(v2.version, 2);
    assert_eq!(v2.assertion_method.len(), 1);
    // created_ledger preserved across update.
    assert_eq!(v2.created_ledger, v1.created_ledger);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")] // VersionMismatch
fn test_update_version_conflict() {
    let (env, controller, did_id, _id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));
    // Use a stale version.
    client.update(&did_id, &99u32, &minimal_record(&env, &controller));
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")] // DidNotFound
fn test_update_nonexistent() {
    let (env, controller, did_id, _id, client) = setup();
    client.update(&did_id, &1u32, &minimal_record(&env, &controller));
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")] // DidDeactivated
fn test_update_deactivated() {
    let (env, controller, did_id, _id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));
    client.deactivate(&did_id, &1u32);
    // Now version is 2 and deactivated; update must fail with DidDeactivated.
    client.update(&did_id, &2u32, &minimal_record(&env, &controller));
}

// --- transfer_controller ---------------------------------------------------

#[test]
fn test_transfer_controller_success() {
    let (env, controller, did_id, _id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));

    let new_controller = Address::generate(&env);
    client.transfer_controller(&did_id, &1u32, &new_controller);

    let r = client.get(&did_id).unwrap();
    assert_eq!(r.controller, new_controller);
    assert_eq!(r.version, 2);
    // Keys and metadata preserved.
    assert_eq!(r.authentication.len(), 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")] // VersionMismatch
fn test_transfer_controller_version_conflict() {
    let (env, controller, did_id, _id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));
    let other = Address::generate(&env);
    client.transfer_controller(&did_id, &99u32, &other);
}

// --- deactivate ------------------------------------------------------------

#[test]
fn test_deactivate_success() {
    let (env, controller, did_id, _id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));
    client.deactivate(&did_id, &1u32);

    let r = client.get(&did_id).unwrap();
    assert!(r.deactivated);
    assert_eq!(r.version, 2);
    assert_eq!(r.authentication.len(), 0);
    assert_eq!(r.assertion_method.len(), 0);
    assert_eq!(r.key_agreement.len(), 0);
    assert_eq!(r.services.len(), 0);
    // Controller preserved for audit.
    assert_eq!(r.controller, controller);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")] // DidDeactivated
fn test_deactivate_twice() {
    let (env, controller, did_id, _id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));
    client.deactivate(&did_id, &1u32);
    client.deactivate(&did_id, &2u32);
}

// --- authorization ---------------------------------------------------------
//
// `setup()` uses `env.mock_all_auths()` which makes everything pass.
// For auth-rejection tests we use `set_auths(&[])` to clear authorizations.

#[test]
#[should_panic]
fn test_auth_register_requires_controller() {
    let env = Env::default();
    let controller = Address::generate(&env);
    let did_id = BytesN::<16>::random(&env);
    let contract_id = env.register(DidStellarRegistry, ());
    let client = DidStellarRegistryClient::new(&env, &contract_id);
    client.register(&did_id, &minimal_record(&env, &controller));
}

#[test]
#[should_panic]
fn test_auth_update_requires_controller() {
    let env = Env::default();
    env.mock_all_auths();
    let controller = Address::generate(&env);
    let did_id = BytesN::<16>::random(&env);
    let contract_id = env.register(DidStellarRegistry, ());
    let client = DidStellarRegistryClient::new(&env, &contract_id);
    client.register(&did_id, &minimal_record(&env, &controller));
    // Strip auths — update must fail.
    env.set_auths(&[]);
    client.update(&did_id, &1u32, &minimal_record(&env, &controller));
}

#[test]
#[should_panic]
fn test_auth_transfer_requires_controller() {
    let env = Env::default();
    env.mock_all_auths();
    let controller = Address::generate(&env);
    let did_id = BytesN::<16>::random(&env);
    let contract_id = env.register(DidStellarRegistry, ());
    let client = DidStellarRegistryClient::new(&env, &contract_id);
    client.register(&did_id, &minimal_record(&env, &controller));
    env.set_auths(&[]);
    let other = Address::generate(&env);
    client.transfer_controller(&did_id, &1u32, &other);
}

#[test]
#[should_panic]
fn test_auth_deactivate_requires_controller() {
    let env = Env::default();
    env.mock_all_auths();
    let controller = Address::generate(&env);
    let did_id = BytesN::<16>::random(&env);
    let contract_id = env.register(DidStellarRegistry, ());
    let client = DidStellarRegistryClient::new(&env, &contract_id);
    client.register(&did_id, &minimal_record(&env, &controller));
    env.set_auths(&[]);
    client.deactivate(&did_id, &1u32);
}

// --- boundary validation ---------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #5)")] // InvalidAuthKeyCount
fn test_boundary_auth_keys_empty() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    r.authentication = empty_keys(&env);
    client.register(&did_id, &r);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")] // InvalidAuthKeyCount
fn test_boundary_auth_keys_over_max() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    let mut auth = empty_keys(&env);
    auth.push_back(key(&env, "z6Mk111111111111111111111111111111111111111111"));
    auth.push_back(key(&env, "z6Mk222222222222222222222222222222222222222222"));
    auth.push_back(key(&env, "z6Mk333333333333333333333333333333333333333333"));
    auth.push_back(key(&env, "z6Mk444444444444444444444444444444444444444444"));
    r.authentication = auth;
    client.register(&did_id, &r);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")] // InvalidAssertionKeyCount
fn test_boundary_assertion_over_max() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    let mut a = empty_keys(&env);
    a.push_back(key(&env, "z6Mk111111111111111111111111111111111111111111"));
    a.push_back(key(&env, "z6Mk222222222222222222222222222222222222222222"));
    a.push_back(key(&env, "z6Mk333333333333333333333333333333333333333333"));
    a.push_back(key(&env, "z6Mk444444444444444444444444444444444444444444"));
    r.assertion_method = a;
    client.register(&did_id, &r);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")] // InvalidKeyAgreementCount
fn test_boundary_key_agreement_over_max() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    let mut ka = empty_keys(&env);
    ka.push_back(key(&env, "z6LS1111111111111111111111111111111111111111"));
    ka.push_back(key(&env, "z6LS2222222222222222222222222222222222222222"));
    r.key_agreement = ka;
    client.register(&did_id, &r);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")] // InvalidServiceCount
fn test_boundary_services_over_max() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    let mut svcs = empty_services(&env);
    for i in 0..4 {
        let suffix = match i {
            0 => "svc-a",
            1 => "svc-b",
            2 => "svc-c",
            _ => "svc-d",
        };
        svcs.push_back(DidService {
            id_suffix: s(&env, suffix),
            service_type: s(&env, "LinkedDomains"),
            service_endpoint: s(&env, "https://example.com"),
        });
    }
    r.services = svcs;
    client.register(&did_id, &r);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")] // KeyTooLong
fn test_boundary_key_too_long() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    // 129 chars > MAX_KEY_MULTIBASE_LEN (128).
    let long: std::string::String = core::iter::repeat('x').take(129).collect();
    let mut auth = empty_keys(&env);
    auth.push_back(DidKey {
        public_key_multibase: String::from_str(&env, &long),
    });
    r.authentication = auth;
    client.register(&did_id, &r);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")] // KeyEmpty
fn test_boundary_key_empty() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    let mut auth = empty_keys(&env);
    auth.push_back(key(&env, ""));
    r.authentication = auth;
    client.register(&did_id, &r);
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")] // ServiceEndpointInvalid
fn test_boundary_service_endpoint_http() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    let mut svcs = empty_services(&env);
    svcs.push_back(DidService {
        id_suffix: s(&env, "issuer"),
        service_type: s(&env, "LinkedDomains"),
        service_endpoint: s(&env, "http://example.com"),
    });
    r.services = svcs;
    client.register(&did_id, &r);
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")] // ServiceIdInvalidFormat
fn test_boundary_service_id_uppercase() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    let mut svcs = empty_services(&env);
    svcs.push_back(DidService {
        id_suffix: s(&env, "Issuer"),
        service_type: s(&env, "LinkedDomains"),
        service_endpoint: s(&env, "https://example.com"),
    });
    r.services = svcs;
    client.register(&did_id, &r);
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")] // ServiceIdInvalidFormat
fn test_boundary_service_id_underscore() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    let mut svcs = empty_services(&env);
    svcs.push_back(DidService {
        id_suffix: s(&env, "iss_uer"),
        service_type: s(&env, "LinkedDomains"),
        service_endpoint: s(&env, "https://example.com"),
    });
    r.services = svcs;
    client.register(&did_id, &r);
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")] // ServiceTypeTooLong
fn test_boundary_service_type_too_long() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    // 65 chars > MAX_SERVICE_TYPE_LEN (64).
    let long: std::string::String = core::iter::repeat('x').take(65).collect();
    let mut svcs = empty_services(&env);
    svcs.push_back(DidService {
        id_suffix: s(&env, "issuer"),
        service_type: String::from_str(&env, &long),
        service_endpoint: s(&env, "https://example.com"),
    });
    r.services = svcs;
    client.register(&did_id, &r);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")] // ServiceIdTooLong
fn test_boundary_service_id_too_long() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    // 33 chars > MAX_SERVICE_ID_LEN (32). Use only valid `[a-z]` so the
    // length check fires before the format check.
    let long: std::string::String = core::iter::repeat('a').take(33).collect();
    let mut svcs = empty_services(&env);
    svcs.push_back(DidService {
        id_suffix: String::from_str(&env, &long),
        service_type: s(&env, "LinkedDomains"),
        service_endpoint: s(&env, "https://example.com"),
    });
    r.services = svcs;
    client.register(&did_id, &r);
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")] // MetadataUriInvalid
fn test_boundary_metadata_uri_http() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    r.metadata_uri = Some(s(&env, "http://example.com/metadata.json"));
    client.register(&did_id, &r);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")] // DuplicateKey
fn test_duplicate_keys_same_relation() {
    let (env, controller, did_id, _id, client) = setup();
    let mut r = minimal_record(&env, &controller);
    let mut auth = empty_keys(&env);
    let same = "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doY";
    auth.push_back(key(&env, same));
    auth.push_back(key(&env, same));
    r.authentication = auth;
    client.register(&did_id, &r);
}

// --- events ---------------------------------------------------------------
// These tests verify the full event payload (topics + data), not just that
// "something" was emitted. `env.events().all()` returns events from the
// last contract invocation only — each test executes a single mutation
// after setup so the event under inspection is unambiguous.

#[test]
fn test_events_register_emits_payload() {
    let (env, controller, did_id, contract_id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));

    let expected = DidRegistered {
        did_id: did_id.clone(),
        controller: controller.clone(),
        version: 1,
    };
    // `ContractEvents` impls `PartialEq<Vec<(Address, Vec<Val>, Val)>>` via
    // XDR comparison — the only structural way to assert event payloads.
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (contract_id.clone(), expected.topics(&env), expected.data(&env))
        ]
    );
}

#[test]
fn test_events_update_emits_payload() {
    let (env, controller, did_id, contract_id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));
    // `env.events().all()` only returns events from the most recent
    // contract invocation, so the register event is dropped here.
    client.update(&did_id, &1u32, &minimal_record(&env, &controller));

    let expected = DidUpdated {
        did_id: did_id.clone(),
        version: 2,
    };
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (contract_id.clone(), expected.topics(&env), expected.data(&env))
        ]
    );
}

#[test]
fn test_events_transfer_emits_payload() {
    let (env, controller, did_id, contract_id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));
    let new_ctrl = Address::generate(&env);
    client.transfer_controller(&did_id, &1u32, &new_ctrl);

    let expected = DidControllerTransferred {
        did_id: did_id.clone(),
        old_controller: controller.clone(),
        new_controller: new_ctrl.clone(),
        version: 2,
    };
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (contract_id.clone(), expected.topics(&env), expected.data(&env))
        ]
    );
}

#[test]
fn test_events_deactivate_emits_payload() {
    let (env, controller, did_id, contract_id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));
    client.deactivate(&did_id, &1u32);

    let expected = DidDeactivated {
        did_id: did_id.clone(),
        version: 2,
    };
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (contract_id.clone(), expected.topics(&env), expected.data(&env))
        ]
    );
}

// --- normative test vectors ------------------------------------------------
// These match `docs/did-spec/test-vectors/vectors.json`.

/// did_id_bytes_hex from vectors.json vector 1: 000102…0f.
fn vector_did_id(env: &Env) -> BytesN<16> {
    let raw: [u8; 16] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f,
    ];
    BytesN::<16>::from_array(env, &raw)
}

#[test]
fn test_vector_1_minimal_did() {
    let env = Env::default();
    env.mock_all_auths();
    let controller = Address::generate(&env);
    let did_id = vector_did_id(&env);
    let contract_id = env.register(DidStellarRegistry, ());
    let client = DidStellarRegistryClient::new(&env, &contract_id);

    let mut auth = empty_keys(&env);
    auth.push_back(key(&env, "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doY"));
    let r = DidRecord {
        controller: controller.clone(),
        authentication: auth,
        assertion_method: empty_keys(&env),
        key_agreement: empty_keys(&env),
        services: empty_services(&env),
        metadata_uri: None,
        metadata_hash: None,
        version: 0,
        created_ledger: 0,
        updated_ledger: 0,
        deactivated: false,
    };
    client.register(&did_id, &r);

    let stored = client.get(&did_id).unwrap();
    assert_eq!(stored.version, 1);
    assert_eq!(stored.deactivated, false);
    assert_eq!(stored.authentication.len(), 1);
    assert_eq!(stored.assertion_method.len(), 0);
    assert_eq!(stored.key_agreement.len(), 0);
    assert_eq!(stored.services.len(), 0);
    let stored_key: DidKey = stored.authentication.get_unchecked(0);
    assert_eq!(
        stored_key.public_key_multibase,
        s(&env, "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doY")
    );
}

#[test]
fn test_vector_2_full_did() {
    let env = Env::default();
    env.mock_all_auths();
    let controller = Address::generate(&env);
    let did_id = vector_did_id(&env);
    let contract_id = env.register(DidStellarRegistry, ());
    let client = DidStellarRegistryClient::new(&env, &contract_id);

    let mut auth = empty_keys(&env);
    auth.push_back(key(&env, "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doY"));
    let mut assert_keys = empty_keys(&env);
    assert_keys.push_back(key(&env, "z6Mkff3F4VMDGbMbMtgRyXMrgr7qyxaKsPo7QEPQ2AkNrx2X"));
    let mut ka = empty_keys(&env);
    ka.push_back(key(&env, "z6LSnGSQaEk7SBZMmMLHTCqz6YUuiVVCmBNdAqSVdepqYAW1"));
    let mut svcs = empty_services(&env);
    svcs.push_back(DidService {
        id_suffix: s(&env, "issuer"),
        service_type: s(&env, "LinkedDomains"),
        service_endpoint: s(&env, "https://issuer.example.com"),
    });

    let r = DidRecord {
        controller,
        authentication: auth,
        assertion_method: assert_keys,
        key_agreement: ka,
        services: svcs,
        metadata_uri: None,
        metadata_hash: None,
        version: 0,
        created_ledger: 0,
        updated_ledger: 0,
        deactivated: false,
    };
    client.register(&did_id, &r);

    let stored = client.get(&did_id).unwrap();
    assert_eq!(stored.authentication.len(), 1);
    assert_eq!(stored.assertion_method.len(), 1);
    assert_eq!(stored.key_agreement.len(), 1);
    assert_eq!(stored.services.len(), 1);
    let svc: DidService = stored.services.get_unchecked(0);
    assert_eq!(svc.id_suffix, s(&env, "issuer"));
    assert_eq!(svc.service_type, s(&env, "LinkedDomains"));
    assert_eq!(svc.service_endpoint, s(&env, "https://issuer.example.com"));
}

#[test]
fn test_vector_3_deactivated_tombstone() {
    let env = Env::default();
    env.mock_all_auths();
    let controller = Address::generate(&env);
    let did_id = vector_did_id(&env);
    let contract_id = env.register(DidStellarRegistry, ());
    let client = DidStellarRegistryClient::new(&env, &contract_id);

    client.register(&did_id, &minimal_record(&env, &controller));
    client.deactivate(&did_id, &1u32);

    let r = client.get(&did_id).unwrap();
    // Tombstone state per spec §6.6.
    assert_eq!(r.deactivated, true);
    assert_eq!(r.version, 2);
    assert_eq!(r.authentication.len(), 0);
    assert_eq!(r.assertion_method.len(), 0);
    assert_eq!(r.key_agreement.len(), 0);
    assert_eq!(r.services.len(), 0);
    // Controller preserved for audit.
    assert_eq!(r.controller, controller);
}

#[test]
fn test_vector_4_concurrent_update_conflict() {
    // Both callers observe version=1 and try to update.
    // First wins → version becomes 2; second is rejected with VersionMismatch.
    let (env, controller, did_id, _id, client) = setup();
    client.register(&did_id, &minimal_record(&env, &controller));
    let v1 = client.get(&did_id).unwrap().version;
    assert_eq!(v1, 1);

    // Caller A succeeds.
    client.update(&did_id, &1u32, &minimal_record(&env, &controller));
    assert_eq!(client.get(&did_id).unwrap().version, 2);

    // Caller B tries with the stale version.
    let result =
        client.try_update(&did_id, &1u32, &minimal_record(&env, &controller));
    assert!(result.is_err());
    let err = result.err().unwrap().unwrap();
    assert_eq!(err, RegistryError::VersionMismatch.into());
}

// --- IntoVal sanity checks for the macros ----------------------------------

#[test]
fn test_record_round_trip_via_intoval() {
    // Ensures `DidRecord` contracttype derive works through the SDK pipeline.
    let env = Env::default();
    let controller = Address::generate(&env);
    let r = minimal_record(&env, &controller);
    let _val: soroban_sdk::Val = (&r).into_val(&env);
}
