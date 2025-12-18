use crate::vc_status::VCStatus;
use crate::verifiable_credential::VerifiableCredential;
use soroban_sdk::{contracttype, Address, Env, Map, String, Vec};

/// Unified storage keys.
///
/// Instance storage:
/// - Small config and per-owner vault metadata.
/// Persistent storage:
/// - VC payloads, VC id indexes, issuer lists, and issuance status registry.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    // -----------------
    // Global config
    // -----------------
    ContractAdmin,          // Address
    DefaultIssuerDid,       // String

    // Global fee configuration (instance storage)
    FeeEnabled,             // bool
    FeeTokenContract,       // Address
    FeeDest,                // Address
    FeeAmount,              // i128

    // -----------------
    // Vault (per owner)
    // -----------------
    VaultAdmin(Address),    // Address
    VaultDid(Address),      // String
    VaultRevoked(Address),  // bool

    // Issuer list per owner (persistent)
    VaultIssuers(Address),  // Vec<Address>

    // VC payload per owner (persistent)
    VaultVC(Address, String), // VerifiableCredential
    VaultVCIds(Address),      // Vec<String>

    // -----------------
    // Issuance registry
    // -----------------
    VCStatus(String),       // VCStatus
    VCOwner(String),        // Address

    // -----------------
    // Legacy keys (for migration)
    // -----------------
    LegacyIssuanceRevocations, // Map<String, LegacyRevocation>
    LegacyIssuanceVCs,         // Vec<String>
    LegacyVaultVCs(Address),   // Vec<VerifiableCredential>
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyRevocation {
    pub vc_id: String,
    pub date: String,
}

// -----------------
// Global config
// -----------------

pub fn has_contract_admin(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::ContractAdmin)
}

pub fn read_contract_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::ContractAdmin).unwrap()
}

pub fn write_contract_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::ContractAdmin, admin);
}

pub fn read_default_issuer_did(e: &Env) -> Option<String> {
    e.storage().instance().get(&DataKey::DefaultIssuerDid)
}

pub fn write_default_issuer_did(e: &Env, did: &String) {
    e.storage().instance().set(&DataKey::DefaultIssuerDid, did);
}

// -----------------
// Fee config
// -----------------

pub fn read_fee_enabled(e: &Env) -> bool {
    match e.storage().instance().get(&DataKey::FeeEnabled) {
        Some(v) => v,
        None => false,
    }
}

pub fn write_fee_enabled(e: &Env, enabled: &bool) {
    e.storage().instance().set(&DataKey::FeeEnabled, enabled);
}

pub fn write_fee_token_contract(e: &Env, addr: &Address) {
    e.storage().instance().set(&DataKey::FeeTokenContract, addr);
}

pub fn read_fee_token_contract(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::FeeTokenContract).unwrap()
}

pub fn write_fee_dest(e: &Env, addr: &Address) {
    e.storage().instance().set(&DataKey::FeeDest, addr);
}

pub fn read_fee_dest(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::FeeDest).unwrap()
}

pub fn write_fee_amount(e: &Env, amount: &i128) {
    e.storage().instance().set(&DataKey::FeeAmount, amount);
}

pub fn read_fee_amount(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::FeeAmount).unwrap()
}

// -----------------
// Vault metadata (instance)
// -----------------

pub fn has_vault_admin(e: &Env, owner: &Address) -> bool {
    e.storage().instance().has(&DataKey::VaultAdmin(owner.clone()))
}

pub fn read_vault_admin(e: &Env, owner: &Address) -> Address {
    e.storage().instance().get(&DataKey::VaultAdmin(owner.clone())).unwrap()
}

pub fn write_vault_admin(e: &Env, owner: &Address, admin: &Address) {
    e.storage().instance().set(&DataKey::VaultAdmin(owner.clone()), admin);
}

pub fn write_vault_did(e: &Env, owner: &Address, did: &String) {
    e.storage().instance().set(&DataKey::VaultDid(owner.clone()), did);
}

pub fn read_vault_did(e: &Env, owner: &Address) -> Option<String> {
    e.storage().instance().get(&DataKey::VaultDid(owner.clone()))
}

pub fn read_vault_revoked(e: &Env, owner: &Address) -> bool {
    e.storage().instance().get(&DataKey::VaultRevoked(owner.clone())).unwrap()
}

pub fn write_vault_revoked(e: &Env, owner: &Address, revoked: &bool) {
    e.storage().instance().set(&DataKey::VaultRevoked(owner.clone()), revoked);
}

// -----------------
// Vault issuers (persistent)
// -----------------

pub fn read_vault_issuers(e: &Env, owner: &Address) -> Vec<Address> {
    e.storage().persistent().get(&DataKey::VaultIssuers(owner.clone())).unwrap()
}

pub fn write_vault_issuers(e: &Env, owner: &Address, issuers: &Vec<Address>) {
    e.storage().persistent().set(&DataKey::VaultIssuers(owner.clone()), issuers)
}

// -----------------
// Vault VC payloads (persistent)
// -----------------

pub fn write_vault_vc(e: &Env, owner: &Address, vc_id: &String, vc: &VerifiableCredential) {
    e.storage().persistent().set(&DataKey::VaultVC(owner.clone(), vc_id.clone()), vc)
}

pub fn read_vault_vc(e: &Env, owner: &Address, vc_id: &String) -> Option<VerifiableCredential> {
    e.storage().persistent().get(&DataKey::VaultVC(owner.clone(), vc_id.clone()))
}

pub fn remove_vault_vc(e: &Env, owner: &Address, vc_id: &String) {
    e.storage().persistent().remove(&DataKey::VaultVC(owner.clone(), vc_id.clone()));
}

pub fn read_vault_vc_ids(e: &Env, owner: &Address) -> Vec<String> {
    match e.storage().persistent().get(&DataKey::VaultVCIds(owner.clone())) {
        Some(v) => v,
        None => Vec::new(e),
    }
}

pub fn write_vault_vc_ids(e: &Env, owner: &Address, ids: &Vec<String>) {
    e.storage().persistent().set(&DataKey::VaultVCIds(owner.clone()), ids)
}

pub fn append_vault_vc_id(e: &Env, owner: &Address, vc_id: &String) {
    let mut ids = read_vault_vc_ids(e, owner);
    if !ids.contains(vc_id.clone()) {
        ids.push_front(vc_id.clone());
        write_vault_vc_ids(e, owner, &ids);
    }
}

pub fn remove_vault_vc_id(e: &Env, owner: &Address, vc_id: &String) {
    let mut ids = read_vault_vc_ids(e, owner);
    if let Some(idx) = ids.first_index_of(vc_id.clone()) {
        ids.remove(idx);
        write_vault_vc_ids(e, owner, &ids);
    }
}

// -----------------
// Issuance status registry (persistent)
// -----------------

pub fn write_vc_status(e: &Env, vc_id: &String, status: &VCStatus) {
    e.storage().persistent().set(&DataKey::VCStatus(vc_id.clone()), status)
}

pub fn read_vc_status(e: &Env, vc_id: &String) -> VCStatus {
    e.storage()
        .persistent()
        .get(&DataKey::VCStatus(vc_id.clone()))
        .unwrap_or(VCStatus::Invalid)
}

pub fn write_vc_owner(e: &Env, vc_id: &String, owner: &Address) {
    e.storage().persistent().set(&DataKey::VCOwner(vc_id.clone()), owner)
}

pub fn read_vc_owner(e: &Env, vc_id: &String) -> Option<Address> {
    e.storage().persistent().get(&DataKey::VCOwner(vc_id.clone()))
}

// -----------------
// Legacy migrations
// -----------------

pub fn read_legacy_issuance_vcs(e: &Env) -> Option<Vec<String>> {
    e.storage().persistent().get(&DataKey::LegacyIssuanceVCs)
}

pub fn remove_legacy_issuance_vcs(e: &Env) {
    e.storage().persistent().remove(&DataKey::LegacyIssuanceVCs);
}

pub fn read_legacy_issuance_revocations(e: &Env) -> Map<String, LegacyRevocation> {
    e.storage().persistent().get(&DataKey::LegacyIssuanceRevocations).unwrap()
}

pub fn remove_legacy_issuance_revocations(e: &Env) {
    e.storage().persistent().remove(&DataKey::LegacyIssuanceRevocations);
}

pub fn read_legacy_vault_vcs(e: &Env, owner: &Address) -> Option<Vec<VerifiableCredential>> {
    e.storage().persistent().get(&DataKey::LegacyVaultVCs(owner.clone()))
}

pub fn remove_legacy_vault_vcs(e: &Env, owner: &Address) {
    e.storage().persistent().remove(&DataKey::LegacyVaultVCs(owner.clone()));
}
