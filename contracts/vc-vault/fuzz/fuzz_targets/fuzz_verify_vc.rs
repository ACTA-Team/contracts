//! Fuzzes verify_vc() with arbitrary vc_id values.
//!
//! verify_vc() must never panic regardless of input. For an id that was never
//! issued it must return Invalid; for one that was issued it must return Valid
//! (before revocation).

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
    /// If true, issue a VC with vc_id before verifying.
    pre_issue: bool,
}

fuzz_target!(|input: FuzzInput| {
    let env = Env::default();
    let (_admin, issuer, owner, cid, client) = setup(&env);

    let vc_id = s(&env, &input.vc_id);

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

    // verify_vc must never panic. If issued, must return Valid.
    let status = client.verify_vc(&owner, &vc_id);

    if input.pre_issue {
        // Issue may have failed (e.g. empty vc_id edge case), so only assert
        // Valid if the VC actually appears in the id list.
        if client.list_vc_ids(&owner).contains(vc_id.clone()) {
            assert_eq!(status, VCStatus::Valid);
        }
    } else {
        assert_eq!(status, VCStatus::Invalid);
    }
});
