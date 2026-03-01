//! Fuzzes push() with arbitrary vc_id values across two vaults.
//!
//! Invariants checked:
//! - If push() succeeds, the VC must no longer exist in the source vault
//!   and must exist in the destination vault.
//! - The VC count across both vaults is conserved (no duplication or loss).

#![no_main]

mod common;

use arbitrary::Arbitrary;
use common::{s, setup};
use libfuzzer_sys::fuzz_target;
use soroban_sdk::{testutils::Address as _, Env};

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    vc_id: String,
    /// If true, the VC is pre-issued in the source vault before push.
    pre_issue: bool,
}

fuzz_target!(|input: FuzzInput| {
    let env = Env::default();
    let (_admin, issuer, from_owner, cid, client) = setup(&env);

    // Set up a second (destination) vault.
    let to_owner = soroban_sdk::Address::generate(&env);
    client.create_vault(&to_owner, &s(&env, "did:fuzz:to_owner"));

    let vc_id = s(&env, &input.vc_id);

    if input.pre_issue {
        let _ = client.try_issue(
            &from_owner,
            &vc_id,
            &s(&env, "data"),
            &cid,
            &issuer,
            &s(&env, "did:fuzz:issuer"),
            &0_i128,
        );
    }

    let from_before = client.list_vc_ids(&from_owner).len();
    let to_before = client.list_vc_ids(&to_owner).len();

    let result = client.try_push(&from_owner, &to_owner, &vc_id, &issuer);

    if let Ok(Ok(())) = result {
        // VC moved: no longer in source, now in destination.
        assert!(client.get_vc(&from_owner, &vc_id).is_none());
        assert!(client.get_vc(&to_owner, &vc_id).is_some());

        // Total VC count is conserved (one moved, none created or destroyed).
        let from_after = client.list_vc_ids(&from_owner).len();
        let to_after = client.list_vc_ids(&to_owner).len();
        assert_eq!(from_before + to_before, from_after + to_after);
    }
});
