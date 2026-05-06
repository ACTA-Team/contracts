//! Contract events. Published on each successful mutation for off-chain
//! observability (resolvers, audit pipelines, indexing services).

use soroban_sdk::{contractevent, Address, BytesN, Env};

#[contractevent]
pub struct DidRegistered {
    pub did_id: BytesN<16>,
    pub controller: Address,
    pub version: u32,
}

#[contractevent]
pub struct DidUpdated {
    pub did_id: BytesN<16>,
    pub version: u32,
}

#[contractevent]
pub struct DidControllerTransferred {
    pub did_id: BytesN<16>,
    pub old_controller: Address,
    pub new_controller: Address,
    pub version: u32,
}

#[contractevent]
pub struct DidDeactivated {
    pub did_id: BytesN<16>,
    pub version: u32,
}

pub fn did_registered(e: &Env, did_id: &BytesN<16>, controller: &Address, version: u32) {
    DidRegistered {
        did_id: did_id.clone(),
        controller: controller.clone(),
        version,
    }
    .publish(e);
}

pub fn did_updated(e: &Env, did_id: &BytesN<16>, version: u32) {
    DidUpdated {
        did_id: did_id.clone(),
        version,
    }
    .publish(e);
}

pub fn did_controller_transferred(
    e: &Env,
    did_id: &BytesN<16>,
    old_controller: &Address,
    new_controller: &Address,
    version: u32,
) {
    DidControllerTransferred {
        did_id: did_id.clone(),
        old_controller: old_controller.clone(),
        new_controller: new_controller.clone(),
        version,
    }
    .publish(e);
}

pub fn did_deactivated(e: &Env, did_id: &BytesN<16>, version: u32) {
    DidDeactivated {
        did_id: did_id.clone(),
        version,
    }
    .publish(e);
}
