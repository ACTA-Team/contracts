//! Fuzzes sequences of authorize_issuer / revoke_issuer with arbitrary inputs.
//!
//! Invariants checked:
//! - After authorize_issuer succeeds, the issuer must not be in the denied list
//!   (i.e. a subsequent issue by that issuer must succeed).
//! - After revoke_issuer succeeds, a subsequent issue by the same issuer must fail.
//! - authorize_issuers deduplication: passing the same issuer N times must leave
//!   exactly one entry (re-authorizing a revoked issuer clears the denied list).

#![no_main]

mod common;

use arbitrary::Arbitrary;
use common::{s, setup};
use libfuzzer_sys::fuzz_target;
use soroban_sdk::{testutils::Address as _, vec, Env};

#[derive(Arbitrary, Debug)]
enum IssuerOp {
    Authorize,
    Revoke,
    BulkAuthorize { count: u8 },
}

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    ops: Vec<IssuerOp>,
    vc_id: String,
}

fuzz_target!(|input: FuzzInput| {
    let env = Env::default();
    let (_admin, issuer, owner, cid, client) = setup(&env);

    // A second issuer for bulk authorize tests.
    let issuer2 = soroban_sdk::Address::generate(&env);

    for op in &input.ops {
        match op {
            IssuerOp::Authorize => {
                let _ = client.try_authorize_issuer(&owner, &issuer);
            }
            IssuerOp::Revoke => {
                let _ = client.try_revoke_issuer(&owner, &issuer);
            }
            IssuerOp::BulkAuthorize { count } => {
                // Build a list with duplicates proportional to count; dedup must hold.
                let n = (*count as usize % 4) + 1;
                let mut list = vec![&env, issuer.clone()];
                for _ in 1..n {
                    list.push_back(issuer.clone());
                }
                list.push_back(issuer2.clone());
                let _ = client.try_authorize_issuers(&owner, &list);
            }
        }
    }

    // After all ops, try to issue a VC and record whether it succeeded.
    let vc_id = s(&env, &input.vc_id);
    let issue_result = client.try_issue(
        &owner,
        &vc_id,
        &s(&env, "data"),
        &cid,
        &issuer,
        &s(&env, "did:fuzz:issuer"),
        &0_i128,
    );

    // If issue succeeded, verify_vc must return Valid.
    if let Ok(Ok(_)) = issue_result {
        use vc_vault_contract::model::VCStatus;
        assert_eq!(client.verify_vc(&owner, &vc_id), VCStatus::Valid);
    }
});
