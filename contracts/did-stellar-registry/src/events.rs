//! Contract events. Published on each successful mutation for off-chain
//! observability (resolvers, audit pipelines, indexing services).

use soroban_sdk::{contractevent, Address, BytesN, Env};

#[contractevent]
pub struct DidRegistered {
    pub did_id: BytesN<16>,
    pub controller: Address,
    pub version: u32,
}

/// Emitted by `register_sponsored`. A distinct type rather than a field on
/// `DidRegistered` so consumers can filter sponsored registrations by topic.
#[contractevent]
pub struct DidRegisteredSponsored {
    pub did_id: BytesN<16>,
    pub sponsor: Address,
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

/// Emitted exactly once when the contract's `__constructor` runs.
#[contractevent]
pub struct ContractInitialized {
    pub admin: Address,
}

/// Emitted when the proposed admin successfully accepts the role.
#[contractevent]
pub struct AdminTransferred {
    pub old_admin: Address,
    pub new_admin: Address,
}

pub fn did_registered(e: &Env, did_id: &BytesN<16>, controller: &Address, version: u32) {
    DidRegistered {
        did_id: did_id.clone(),
        controller: controller.clone(),
        version,
    }
    .publish(e);
}

pub fn did_registered_sponsored(
    e: &Env,
    did_id: &BytesN<16>,
    sponsor: &Address,
    controller: &Address,
    version: u32,
) {
    DidRegisteredSponsored {
        did_id: did_id.clone(),
        sponsor: sponsor.clone(),
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

pub fn contract_initialized(e: &Env, admin: &Address) {
    ContractInitialized {
        admin: admin.clone(),
    }
    .publish(e);
}

pub fn admin_transferred(e: &Env, old_admin: &Address, new_admin: &Address) {
    AdminTransferred {
        old_admin: old_admin.clone(),
        new_admin: new_admin.clone(),
    }
    .publish(e);
}
