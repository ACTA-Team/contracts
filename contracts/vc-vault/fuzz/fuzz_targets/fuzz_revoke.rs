//! Fuzzes revoke() with arbitrary vc_id and date values, with and without
//! a pre-issued VC sharing that vc_id.
//!
//! Invariants checked:
//! - If revoke() succeeds, verify_vc() must return Revoked(date).
//! - Re-revoking the same vc_id must fail (VCAlreadyRevoked / VCNotFound).

#![no_main]

mod common;

use arbitrary::Arbitrary;
use common::{s, setup};
use libfuzzer_sys::fuzz_target;
use soroban_sdk::Env;
use vc_vault_contract::model::VCStatus;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    vc_id: String,
    date: String,
    /// If true, issue a VC with the same vc_id before revoking.
    pre_issue: bool,
}

fuzz_target!(|input: FuzzInput| {
    let env = Env::default();
    let (_admin, issuer, owner, cid, client) = setup(&env);

    let vc_id = s(&env, &input.vc_id);
    let date = s(&env, &input.date);

    if input.pre_issue {
        let _ = client.try_issue(
            &owner,
            &vc_id,
            &s(&env, "data"),
            &cid,
            &issuer,
            &s(&env, "did:fuzz:issuer"),
            &0_i128,
        );
    }

    let result = client.try_revoke(&owner, &vc_id, &date);

    if let Ok(Ok(())) = result {
        // Revoke succeeded: status must reflect the revocation.
        assert_eq!(
            client.verify_vc(&owner, &vc_id),
            VCStatus::Revoked(date.clone())
        );

        // Re-revoking the same vc_id must not succeed.
        let second = client.try_revoke(&owner, &vc_id, &date);
        assert!(matches!(second, Ok(Err(_)) | Err(_)));
    }
});
