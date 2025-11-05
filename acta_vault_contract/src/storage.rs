use crate::verifiable_credential::VerifiableCredential;
use soroban_sdk::{contracttype, Address, Env, String, Vec};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    ContractAdmin,         // Address
    Admin(Address),        // Address
    Did(Address),          // String
    Revoked(Address),      // Boolean
    Issuers(Address),      // Vec<Address>
    VC(Address, String),   // VerifiableCredential
    VCs(Address),          // Vec<VerifiableCredential>
    VCIds(Address),        // Vec<String>
    // Global fee configuration (instance storage)
    FeeEnabled,            // bool
    FeeTokenContract,      // Address
    FeeDest,               // Address
    FeeAmount,             // i128 (scaled to token decimals)
}

pub fn has_admin(e: &Env, owner: &Address) -> bool {
    let key = DataKey::Admin(owner.clone());
    e.storage().instance().has(&key)
}

pub fn has_contract_admin(e: &Env) -> bool {
    let key = DataKey::ContractAdmin;
    e.storage().instance().has(&key)
}

pub fn read_admin(e: &Env, owner: &Address) -> Address {
    let key = DataKey::Admin(owner.clone());
    e.storage().instance().get(&key).unwrap()
}

pub fn read_contract_admin(e: &Env) -> Address {
    let key = DataKey::ContractAdmin;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_admin(e: &Env, owner: &Address, admin: &Address) {
    let key = DataKey::Admin(owner.clone());
    e.storage().instance().set(&key, admin);
}

pub fn write_contract_admin(e: &Env, admin: &Address) {
    let key = DataKey::ContractAdmin;
    e.storage().instance().set(&key, admin);
}

pub fn write_did(e: &Env, owner: &Address, did: &String) {
    let key = DataKey::Did(owner.clone());
    e.storage().instance().set(&key, did);
}

// DID generativo: el address del contrato DID ya no se guarda.

pub fn read_revoked(e: &Env, owner: &Address) -> bool {
    let key = DataKey::Revoked(owner.clone());
    e.storage().instance().get(&key).unwrap()
}

pub fn write_revoked(e: &Env, owner: &Address, revoked: &bool) {
    let key = DataKey::Revoked(owner.clone());
    e.storage().instance().set(&key, revoked);
}

// --- Fee configuration helpers (instance storage) ---
pub fn has_fee_enabled(e: &Env) -> bool {
    let key = DataKey::FeeEnabled;
    e.storage().instance().has(&key)
}

pub fn read_fee_enabled(e: &Env) -> bool {
    let key = DataKey::FeeEnabled;
    match e.storage().instance().get(&key) {
        Some(v) => v,
        None => false,
    }
}

pub fn write_fee_enabled(e: &Env, enabled: &bool) {
    let key = DataKey::FeeEnabled;
    e.storage().instance().set(&key, enabled);
}

pub fn write_fee_token_contract(e: &Env, addr: &Address) {
    let key = DataKey::FeeTokenContract;
    e.storage().instance().set(&key, addr);
}

pub fn read_fee_token_contract(e: &Env) -> Address {
    let key = DataKey::FeeTokenContract;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_fee_dest(e: &Env, addr: &Address) {
    let key = DataKey::FeeDest;
    e.storage().instance().set(&key, addr);
}

pub fn read_fee_dest(e: &Env) -> Address {
    let key = DataKey::FeeDest;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_fee_amount(e: &Env, amount: &i128) {
    let key = DataKey::FeeAmount;
    e.storage().instance().set(&key, amount);
}

pub fn read_fee_amount(e: &Env) -> i128 {
    let key = DataKey::FeeAmount;
    e.storage().instance().get(&key).unwrap()
}

pub fn read_issuers(e: &Env, owner: &Address) -> Vec<Address> {
    let key = DataKey::Issuers(owner.clone());
    e.storage().persistent().get(&key).unwrap()
}

pub fn write_issuers(e: &Env, owner: &Address, issuers: &Vec<Address>) {
    let key = DataKey::Issuers(owner.clone());
    e.storage().persistent().set(&key, issuers)
}

pub fn write_vc(e: &Env, owner: &Address, vc_id: &String, vc: &VerifiableCredential) {
    let key = DataKey::VC(owner.clone(), vc_id.clone());
    e.storage().persistent().set(&key, vc)
}

pub fn read_vc(e: &Env, owner: &Address, vc_id: &String) -> Option<VerifiableCredential> {
    let key = DataKey::VC(owner.clone(), vc_id.clone());
    e.storage().persistent().get(&key)
}

pub fn read_old_vcs(e: &Env, owner: &Address) -> Option<Vec<VerifiableCredential>> {
    let key = DataKey::VCs(owner.clone());
    e.storage().persistent().get(&key)
}

pub fn remove_old_vcs(e: &Env, owner: &Address) {
    let key = DataKey::VCs(owner.clone());
    e.storage().persistent().remove(&key);
}

pub fn read_vc_ids(e: &Env, owner: &Address) -> Vec<String> {
    let key = DataKey::VCIds(owner.clone());
    match e.storage().persistent().get(&key) {
        Some(v) => v,
        None => Vec::new(e),
    }
}

pub fn write_vc_ids(e: &Env, owner: &Address, ids: &Vec<String>) {
    let key = DataKey::VCIds(owner.clone());
    e.storage().persistent().set(&key, ids)
}

pub fn append_vc_id(e: &Env, owner: &Address, vc_id: &String) {
    let mut ids = read_vc_ids(e, owner);
    if !ids.contains(vc_id.clone()) {
        ids.push_front(vc_id.clone());
        write_vc_ids(e, owner, &ids);
    }
}

pub fn remove_vc(e: &Env, owner: &Address, vc_id: &String) {
    let key = DataKey::VC(owner.clone(), vc_id.clone());
    e.storage().persistent().remove(&key);
}

pub fn remove_vc_id(e: &Env, owner: &Address, vc_id: &String) {
    let mut ids = read_vc_ids(e, owner);
    if let Some(idx) = ids.first_index_of(vc_id.clone()) {
        ids.remove(idx);
        write_vc_ids(e, owner, &ids);
    }
}
