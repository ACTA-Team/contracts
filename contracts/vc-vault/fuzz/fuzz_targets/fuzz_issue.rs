//! Fuzzes issue() with arbitrary vc_id, vc_data, issuer_did, and fee_override.
//!
//! Invariant checked: if issue() succeeds, verify_vc() must return Valid
//! and list_vc_ids() must contain the vc_id.

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
    vc_data: String,
    issuer_did: String,
    /// i64 used because arbitrary does not cover all of i128; cast on use.
    fee_override: i64,
}

fuzz_target!(|input: FuzzInput| {
    let env = Env::default();
    let (_admin, issuer, owner, cid, client) = setup(&env);

    let vc_id = s(&env, &input.vc_id);
    let vc_data = s(&env, &input.vc_data);
    let issuer_did = s(&env, &input.issuer_did);
    let fee = input.fee_override as i128;

    let result = client.try_issue(&owner, &vc_id, &vc_data, &cid, &issuer, &issuer_did, &fee);

    if let Ok(Ok(_)) = result {
        // Issue succeeded: verify_vc must return Valid and vc_id must be indexed.
        assert_eq!(client.verify_vc(&owner, &vc_id), VCStatus::Valid);
        assert!(client.list_vc_ids(&owner).contains(vc_id.clone()));
    }
});
