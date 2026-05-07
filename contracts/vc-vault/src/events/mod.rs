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

#[contractevent]
pub struct VCPushed {
    pub from_owner: Address,
    pub to_owner: Address,
    pub vc_id: String,
}

#[contractevent]
pub struct VaultAdminChanged {
    pub owner: Address,
    pub old_admin: Address,
    pub new_admin: Address,
}

#[contractevent]
pub struct LinkedVCIssued {
    pub issuer: Address,
    pub owner: Address,
    pub vc_id: String,
    pub parent_owner: Address,
    pub parent_vc_id: String,
}

pub fn vc_pushed(e: &Env, from_owner: &Address, to_owner: &Address, vc_id: &String) {
    VCPushed {
        from_owner: from_owner.clone(),
        to_owner: to_owner.clone(),
        vc_id: vc_id.clone(),
    }
    .publish(e);
}

pub fn vault_admin_changed(e: &Env, owner: &Address, old_admin: &Address, new_admin: &Address) {
    VaultAdminChanged {
        owner: owner.clone(),
        old_admin: old_admin.clone(),
        new_admin: new_admin.clone(),
    }
    .publish(e);
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

pub fn linked_vc_issued(
    e: &Env,
    issuer: &Address,
    owner: &Address,
    vc_id: &String,
    parent_owner: &Address,
    parent_vc_id: &String,
) {
    LinkedVCIssued {
        issuer: issuer.clone(),
        owner: owner.clone(),
        vc_id: vc_id.clone(),
        parent_owner: parent_owner.clone(),
        parent_vc_id: parent_vc_id.clone(),
    }
    .publish(e);
}
