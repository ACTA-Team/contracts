//! Fuzzes issue() and issue_linked() with arbitrary inputs.
//!
//! Invariants checked:
//! - If issue() succeeds, verify_vc() must return Valid and vc_id must be indexed.
//! - If issue_linked() succeeds, get_vc_parent() must return the correct parent link.

#![no_main]

mod common;

use arbitrary::Arbitrary;
use common::{s, setup};
use libfuzzer_sys::fuzz_target;
use soroban_sdk::{testutils::Address as _, Address, Env};
use vc_vault_contract::model::VCStatus;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    vc_id: String,
    vc_data: String,
    issuer_did: String,
    /// i64 used because arbitrary does not cover all of i128; cast on use.
    fee_override: i64,
    /// When true, exercise issue_linked instead of issue.
    use_linked: bool,
    /// vc_id to use for the parent VC (may or may not be valid).
    parent_vc_id: String,
    /// When true, actually issue the parent VC first so it is Valid.
    use_valid_parent: bool,
}

fuzz_target!(|input: FuzzInput| {
    let env = Env::default();
    let (admin, issuer, owner, cid, client) = setup(&env);

    let vc_id = s(&env, &input.vc_id);
    let vc_data = s(&env, &input.vc_data);
    let issuer_did = s(&env, &input.issuer_did);
    let fee = input.fee_override as i128;

    if !input.use_linked {
        let result =
            client.try_issue(&owner, &vc_id, &vc_data, &cid, &issuer, &issuer_did, &fee);
        if let Ok(Ok(_)) = result {
            assert_eq!(client.verify_vc(&owner, &vc_id), VCStatus::Valid);
            assert!(client.list_vc_ids(&owner).contains(vc_id.clone()));
        }
    } else {
        // Set up a separate parent vault (foundation).
        let _ = admin;
        let parent_owner = Address::generate(&env);
        client.create_vault(&parent_owner, &s(&env, "did:fuzz:parent"));

        let parent_vc_id = s(&env, &input.parent_vc_id);
        let parent_data = s(&env, "parent-data");

        if input.use_valid_parent {
            // Issue the parent VC so it exists and is Valid.
            let _ = client.try_issue(
                &parent_owner,
                &parent_vc_id,
                &parent_data,
                &cid,
                &issuer,
                &issuer_did,
                &0_i128,
            );
        }

        let result = client.try_issue_linked(
            &issuer,
            &owner,
            &vc_id,
            &vc_data,
            &cid,
            &issuer_did,
            &parent_owner,
            &parent_vc_id,
        );

        if let Ok(Ok(())) = result {
            // issue_linked succeeded: verify_vc must be Valid and parent link must be correct.
            assert_eq!(client.verify_vc(&owner, &vc_id), VCStatus::Valid);
            assert!(client.list_vc_ids(&owner).contains(vc_id.clone()));
            let link = client.get_vc_parent(&owner, &vc_id);
            assert!(link.is_some());
            let (ret_owner, ret_id) = link.unwrap();
            assert_eq!(ret_owner, parent_owner);
            assert_eq!(ret_id, parent_vc_id);
        }
    }
});
