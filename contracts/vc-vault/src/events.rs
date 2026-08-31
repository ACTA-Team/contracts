//! Contract events. Published on key state transitions for on-chain observability.

use soroban_sdk::{contractevent, Address, Env, String};

// --- Vault lifecycle ---

#[contractevent]
pub struct VaultCreated {
    pub owner: Address,
    pub did_uri: String,
}

#[contractevent]
pub struct VaultRevoked {}

#[contractevent]
pub struct VaultAdminChanged {
    pub old_admin: Address,
    pub new_admin: Address,
}

// --- Issuer management ---

#[contractevent]
pub struct IssuerDenied {
    pub issuer: Address,
}

#[contractevent]
pub struct IssuerAllowed {
    pub issuer: Address,
}

#[contractevent]
pub struct VaultDidChanged {
    pub did_uri: String,
}

// --- Credential lifecycle ---

#[contractevent]
pub struct VCIssued {
    pub vc_id: String,
    pub issuer: Address,
}

#[contractevent]
pub struct VCRevoked {
    pub vc_id: String,
    pub date: String,
}

#[contractevent]
pub struct VCPushed {
    pub vc_id: String,
    pub dest_vault: Address,
}

// --- Admin / governance ---

#[contractevent]
pub struct ContractInitialized {
    pub admin: Address,
}

#[contractevent]
pub struct AdminNominated {
    pub current_admin: Address,
    pub nominee: Address,
}

#[contractevent]
pub struct AdminTransferred {
    pub old_admin: Address,
    pub new_admin: Address,
}

// --- Publishers ---

pub fn vault_created(e: &Env, owner: &Address, did_uri: &String) {
    VaultCreated {
        owner: owner.clone(),
        did_uri: did_uri.clone(),
    }
    .publish(e);
}

pub fn vault_revoked(e: &Env) {
    VaultRevoked {}.publish(e);
}

pub fn vault_admin_changed(e: &Env, old_admin: &Address, new_admin: &Address) {
    VaultAdminChanged {
        old_admin: old_admin.clone(),
        new_admin: new_admin.clone(),
    }
    .publish(e);
}

pub fn issuer_denied(e: &Env, issuer: &Address) {
    IssuerDenied {
        issuer: issuer.clone(),
    }
    .publish(e);
}

pub fn issuer_allowed(e: &Env, issuer: &Address) {
    IssuerAllowed {
        issuer: issuer.clone(),
    }
    .publish(e);
}

pub fn vault_did_changed(e: &Env, did_uri: &String) {
    VaultDidChanged {
        did_uri: did_uri.clone(),
    }
    .publish(e);
}

pub fn vc_issued(e: &Env, vc_id: &String, issuer: &Address) {
    VCIssued {
        vc_id: vc_id.clone(),
        issuer: issuer.clone(),
    }
    .publish(e);
}

pub fn vc_revoked(e: &Env, vc_id: &String, date: &String) {
    VCRevoked {
        vc_id: vc_id.clone(),
        date: date.clone(),
    }
    .publish(e);
}

pub fn vc_pushed(e: &Env, vc_id: &String, dest_vault: &Address) {
    VCPushed {
        vc_id: vc_id.clone(),
        dest_vault: dest_vault.clone(),
    }
    .publish(e);
}

pub fn contract_initialized(e: &Env, admin: &Address) {
    ContractInitialized {
        admin: admin.clone(),
    }
    .publish(e);
}

pub fn admin_nominated(e: &Env, current_admin: &Address, nominee: &Address) {
    AdminNominated {
        current_admin: current_admin.clone(),
        nominee: nominee.clone(),
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
