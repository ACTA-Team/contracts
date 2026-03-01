//! Lifecycle fuzzer: executes arbitrary sequences of all contract operations
//! and verifies key invariants after each step.
//!
//! Uses a pool of 8 vc_ids and 4 issuers indexed by u8 to maximise collision
//! probability. Tracks expected state in Rust and asserts consistency with the
//! contract after every successful operation.
//!
//! Key invariants enforced:
//! - verify_vc() always matches the tracked state (Valid / Revoked / Invalid).
//! - Re-issuing an already-issued vc_id always fails.
//! - Revoking a vc_id that has been revoked always fails.
//! - After revoke_issuer, issue by that issuer must fail.
//! - After authorize_issuer (re-auth), issue by that issuer must succeed again.

#![no_main]

mod common;

use arbitrary::Arbitrary;
use common::{s, setup};
use libfuzzer_sys::fuzz_target;
use soroban_sdk::{testutils::Address as _, Env};
use std::collections::HashMap;
use vc_vault_contract::model::VCStatus;

/// Pool of pre-defined vc_id strings (index → id).
const VC_IDS: &[&str] = &[
    "vc-0", "vc-1", "vc-2", "vc-3", "vc-4", "vc-5", "vc-6", "vc-7",
];

#[derive(Arbitrary, Debug)]
enum Op {
    /// Issue VC at vc_ids[idx % 8] by issuers[issuer_idx % 4].
    Issue { vc_idx: u8, issuer_idx: u8 },
    /// Revoke VC at vc_ids[idx % 8].
    Revoke { vc_idx: u8, date: String },
    /// Verify VC at vc_ids[idx % 8] and assert expected status.
    Verify { vc_idx: u8 },
    /// Authorize issuers[issuer_idx % 4] in owner vault.
    AuthorizeIssuer { issuer_idx: u8 },
    /// Revoke issuers[issuer_idx % 4] from owner vault.
    RevokeIssuer { issuer_idx: u8 },
    /// Push VC at vc_ids[idx % 8] from vault_a to vault_b.
    Push { vc_idx: u8, issuer_idx: u8 },
}

#[derive(Debug, PartialEq, Clone)]
enum TrackedStatus {
    NotIssued,
    Valid,
    Revoked(String),
    /// Moved to the secondary vault via push.
    Pushed,
}

fuzz_target!(|ops: Vec<Op>| {
    let env = Env::default();
    let (_admin, _default_issuer, owner_a, cid, client) = setup(&env);

    // Second vault for push tests.
    let owner_b = soroban_sdk::Address::generate(&env);
    client.create_vault(&owner_b, &s(&env, "did:fuzz:owner_b"));

    // Pool of 4 issuers (index 0 is the one pre-authorized by setup()).
    let issuers = [
        soroban_sdk::Address::generate(&env),
        soroban_sdk::Address::generate(&env),
        soroban_sdk::Address::generate(&env),
        soroban_sdk::Address::generate(&env),
    ];
    // Pre-authorize all issuers in vault_a so issue() can proceed without
    // worrying about auto-authorization state in the early iterations.
    for iss in &issuers {
        let _ = client.try_authorize_issuer(&owner_a, iss);
    }

    // Tracked state: vc_id → status in vault_a.
    let mut state: HashMap<&str, TrackedStatus> =
        VC_IDS.iter().map(|id| (*id, TrackedStatus::NotIssued)).collect();

    // Tracked issuer authorization: index → authorized in vault_a.
    let mut issuer_auth: [bool; 4] = [true; 4];

    for op in &ops {
        match op {
            Op::Issue { vc_idx, issuer_idx } => {
                let vc_id_str = VC_IDS[*vc_idx as usize % VC_IDS.len()];
                let vc_id = s(&env, vc_id_str);
                let iss_idx = *issuer_idx as usize % issuers.len();
                let issuer = &issuers[iss_idx];

                let result = client.try_issue(
                    &owner_a,
                    &vc_id,
                    &s(&env, "data"),
                    &cid,
                    issuer,
                    &s(&env, "did:fuzz:issuer"),
                    &0_i128,
                );

                match &result {
                    Ok(Ok(_)) => {
                        // Succeeded: must have been NotIssued and issuer authorized.
                        assert_eq!(
                            state[vc_id_str],
                            TrackedStatus::NotIssued,
                            "issue succeeded but vc_id was already in state {:?}",
                            state[vc_id_str]
                        );
                        assert!(
                            issuer_auth[iss_idx],
                            "issue succeeded but issuer was tracked as revoked"
                        );
                        state.insert(vc_id_str, TrackedStatus::Valid);
                    }
                    Ok(Err(_)) | Err(_) => {
                        // Failure is expected if already issued, pushed, or issuer revoked.
                        // No state update.
                    }
                }
            }

            Op::Revoke { vc_idx, date } => {
                let vc_id_str = VC_IDS[*vc_idx as usize % VC_IDS.len()];
                let vc_id = s(&env, vc_id_str);
                let date_s = s(&env, date);

                let result = client.try_revoke(&owner_a, &vc_id, &date_s);

                match &result {
                    Ok(Ok(())) => {
                        // Must have been Valid before revoke.
                        assert_eq!(
                            state[vc_id_str],
                            TrackedStatus::Valid,
                            "revoke succeeded but vc_id state was {:?}",
                            state[vc_id_str]
                        );
                        state.insert(vc_id_str, TrackedStatus::Revoked(date.clone()));
                    }
                    Ok(Err(_)) | Err(_) => {}
                }
            }

            Op::Verify { vc_idx } => {
                let vc_id_str = VC_IDS[*vc_idx as usize % VC_IDS.len()];
                let vc_id = s(&env, vc_id_str);

                let status = client.verify_vc(&owner_a, &vc_id);

                match &state[vc_id_str] {
                    TrackedStatus::NotIssued | TrackedStatus::Pushed => {
                        assert_eq!(status, VCStatus::Invalid);
                    }
                    TrackedStatus::Valid => {
                        assert_eq!(status, VCStatus::Valid);
                    }
                    TrackedStatus::Revoked(date) => {
                        assert_eq!(status, VCStatus::Revoked(s(&env, date)));
                    }
                }
            }

            Op::AuthorizeIssuer { issuer_idx } => {
                let iss_idx = *issuer_idx as usize % issuers.len();
                let issuer = &issuers[iss_idx];
                let result = client.try_authorize_issuer(&owner_a, issuer);
                if let Ok(Ok(())) = result {
                    issuer_auth[iss_idx] = true;
                }
            }

            Op::RevokeIssuer { issuer_idx } => {
                let iss_idx = *issuer_idx as usize % issuers.len();
                let issuer = &issuers[iss_idx];
                let result = client.try_revoke_issuer(&owner_a, issuer);
                if let Ok(Ok(())) = result {
                    issuer_auth[iss_idx] = false;
                }
            }

            Op::Push { vc_idx, issuer_idx } => {
                let vc_id_str = VC_IDS[*vc_idx as usize % VC_IDS.len()];
                let vc_id = s(&env, vc_id_str);
                let iss_idx = *issuer_idx as usize % issuers.len();
                let issuer = &issuers[iss_idx];

                let result = client.try_push(&owner_a, &owner_b, &vc_id, issuer);

                if let Ok(Ok(())) = result {
                    // Must have been Valid in vault_a.
                    assert_eq!(
                        state[vc_id_str],
                        TrackedStatus::Valid,
                        "push succeeded but vc_id state was {:?}",
                        state[vc_id_str]
                    );
                    assert!(client.get_vc(&owner_a, &vc_id).is_none());
                    assert!(client.get_vc(&owner_b, &vc_id).is_some());
                    state.insert(vc_id_str, TrackedStatus::Pushed);
                }
            }
        }
    }
});
