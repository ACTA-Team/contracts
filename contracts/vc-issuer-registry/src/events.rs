//! Contract events. Published on key state transitions for on-chain observability.

use soroban_sdk::{contractevent, Address, Env};

#[contractevent]
pub struct Initialized {
    pub admin: Address,
}

#[contractevent]
pub struct IssuerAdded {
    pub issuer: Address,
}

#[contractevent]
pub struct MetadataUpdated {
    pub issuer: Address,
}

#[contractevent]
pub struct IssuerAllowedSet {
    pub issuer: Address,
    pub allowed: bool,
}

#[contractevent]
pub struct IssuerRemoved {
    pub issuer: Address,
}

pub fn initialized(e: &Env, admin: &Address) {
    Initialized {
        admin: admin.clone(),
    }
    .publish(e);
}

pub fn issuer_added(e: &Env, issuer: &Address) {
    IssuerAdded {
        issuer: issuer.clone(),
    }
    .publish(e);
}

pub fn metadata_updated(e: &Env, issuer: &Address) {
    MetadataUpdated {
        issuer: issuer.clone(),
    }
    .publish(e);
}

pub fn issuer_allowed_set(e: &Env, issuer: &Address, allowed: bool) {
    IssuerAllowedSet {
        issuer: issuer.clone(),
        allowed,
    }
    .publish(e);
}

pub fn issuer_removed(e: &Env, issuer: &Address) {
    IssuerRemoved {
        issuer: issuer.clone(),
    }
    .publish(e);
}
