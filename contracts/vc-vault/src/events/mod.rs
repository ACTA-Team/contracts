//! Contract events. Published on key state transitions for on-chain observability.

use soroban_sdk::{contractevent, Address, Env, String};

#[contractevent]
pub struct VaultCreated {
    pub owner: Address,
    pub did_uri: String,
}

#[contractevent]
pub struct SponsoredVaultCreated {
    pub sponsor: Address,
    pub owner: Address,
    pub did_uri: String,
}

#[contractevent]
pub struct VaultRevoked {
    pub owner: Address,
}

#[contractevent]
pub struct IssuerAuthorized {
    pub owner: Address,
    pub issuer: Address,
}

#[contractevent]
pub struct IssuerRevoked {
    pub owner: Address,
    pub issuer: Address,
}

#[contractevent]
pub struct VCIssued {
    pub owner: Address,
    pub vc_id: String,
    pub issuer: Address,
}

#[contractevent]
pub struct VCRevoked {
    pub owner: Address,
    pub vc_id: String,
    pub date: String,
}

pub fn vault_created(e: &Env, owner: &Address, did_uri: &String) {
    VaultCreated {
        owner: owner.clone(),
        did_uri: did_uri.clone(),
    }
    .publish(e);
}

pub fn sponsored_vault_created(e: &Env, sponsor: &Address, owner: &Address, did_uri: &String) {
    SponsoredVaultCreated {
        sponsor: sponsor.clone(),
        owner: owner.clone(),
        did_uri: did_uri.clone(),
    }
    .publish(e);
}

pub fn vault_revoked(e: &Env, owner: &Address) {
    VaultRevoked {
        owner: owner.clone(),
    }
    .publish(e);
}

pub fn issuer_authorized(e: &Env, owner: &Address, issuer: &Address) {
    IssuerAuthorized {
        owner: owner.clone(),
        issuer: issuer.clone(),
    }
    .publish(e);
}

pub fn issuer_revoked(e: &Env, owner: &Address, issuer: &Address) {
    IssuerRevoked {
        owner: owner.clone(),
        issuer: issuer.clone(),
    }
    .publish(e);
}

pub fn vc_issued(e: &Env, owner: &Address, vc_id: &String, issuer: &Address) {
    VCIssued {
        owner: owner.clone(),
        vc_id: vc_id.clone(),
        issuer: issuer.clone(),
    }
    .publish(e);
}

pub fn vc_revoked(e: &Env, owner: &Address, vc_id: &String, date: &String) {
    VCRevoked {
        owner: owner.clone(),
        vc_id: vc_id.clone(),
        date: date.clone(),
    }
    .publish(e);
}
