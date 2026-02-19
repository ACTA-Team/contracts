//! Storage layout and helpers. Instance = global config; persistent = per-owner and per-VC.

use crate::model::{VCStatus, VerifiableCredential};
use soroban_sdk::{contracttype, Address, Env, Map, String, Vec};

/// TTL: extend when remaining < threshold, set to extend_to (ledger counts).
/// Max per network: ~31_536_000 ledgers (~6 months). Extend to max so credentials
/// stay accessible as long as possible without access.
const INSTANCE_TTL_THRESHOLD: u32 = 30_000_000;
const INSTANCE_TTL_EXTEND_TO: u32 = 31_536_000;
const PERSISTENT_TTL_THRESHOLD: u32 = 30_000_000;
const PERSISTENT_TTL_EXTEND_TO: u32 = 31_536_000;

/// Storage keys. Instance = admin, fees. Persistent = vault metadata, VCs, status.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    ContractAdmin,
    DefaultIssuerDid,
    FeeEnabled,
    FeeTokenContract,
    FeeDest,
    FeeAmount,
    FeeAdmin,
    FeeStandard,
    FeeEarly,
    FeeCustom(Address),
    VaultAdmin(Address),
    VaultDid(Address),
    VaultRevoked(Address),
    VaultIssuers(Address),
    VaultVC(Address, String),
    VaultVCIds(Address),
    VCStatus(String),
    VCOwner(String),
    LegacyIssuanceRevocations,
    LegacyIssuanceVCs,
    LegacyVaultVCs(Address),
}

/// Legacy revocation record for migration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyRevocation {
    /// VC ID.
    pub vc_id: String,
    /// Revocation date (ISO-8601).
    pub date: String,
}

// --- Global config (instance) ---

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

/// Fee config status returned by fee_config().
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeConfig {
    /// Whether fees are enabled.
    pub enabled: bool,
    /// Whether token, dest, amount are all set.
    pub configured: bool,
    /// Token contract address (if configured).
    pub token_contract: Option<Address>,
    /// Fee destination address (if configured).
    pub fee_dest: Option<Address>,
    /// Fee amount (if configured).
    pub fee_amount: Option<i128>,
}

pub fn try_read_fee_token_contract(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::FeeTokenContract)
}

pub fn try_read_fee_dest(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::FeeDest)
}

pub fn try_read_fee_amount(e: &Env) -> Option<i128> {
    e.storage().instance().get(&DataKey::FeeAmount)
}

pub fn read_fee_config(e: &Env) -> FeeConfig {
    let enabled = read_fee_enabled(e);
    let token_contract = try_read_fee_token_contract(e);
    let fee_dest = try_read_fee_dest(e);
    let fee_amount = try_read_fee_amount(e);
    let configured = token_contract.is_some() && fee_dest.is_some() && fee_amount.is_some();
    FeeConfig {
        enabled,
        configured,
        token_contract,
        fee_dest,
        fee_amount,
    }
}

pub fn write_fee_admin(e: &Env, amount: &i128) {
    e.storage().instance().set(&DataKey::FeeAdmin, amount);
}

pub fn try_read_fee_admin(e: &Env) -> Option<i128> {
    e.storage().instance().get(&DataKey::FeeAdmin)
}

pub fn read_fee_admin(e: &Env) -> i128 {
    try_read_fee_admin(e).unwrap_or(0)
}

pub fn write_fee_standard(e: &Env, amount: &i128) {
    e.storage().instance().set(&DataKey::FeeStandard, amount);
}

pub fn try_read_fee_standard(e: &Env) -> Option<i128> {
    e.storage().instance().get(&DataKey::FeeStandard)
}

pub fn read_fee_standard(e: &Env) -> i128 {
    try_read_fee_standard(e).unwrap_or(1_000_000)
}

pub fn write_fee_early(e: &Env, amount: &i128) {
    e.storage().instance().set(&DataKey::FeeEarly, amount);
}

pub fn try_read_fee_early(e: &Env) -> Option<i128> {
    e.storage().instance().get(&DataKey::FeeEarly)
}

pub fn read_fee_early(e: &Env) -> i128 {
    try_read_fee_early(e).unwrap_or(400_000)
}

pub fn write_fee_custom(e: &Env, issuer: &Address, amount: &i128) {
    e.storage().instance().set(&DataKey::FeeCustom(issuer.clone()), amount);
}

pub fn try_read_fee_custom(e: &Env, issuer: &Address) -> Option<i128> {
    e.storage().instance().get(&DataKey::FeeCustom(issuer.clone()))
}

pub fn read_fee_custom(e: &Env, issuer: &Address) -> i128 {
    try_read_fee_custom(e, issuer).unwrap_or_else(|| read_fee_amount(e))
}

// --- Vault metadata (persistent) ---

pub fn has_vault_admin(e: &Env, owner: &Address) -> bool {
    e.storage().persistent().has(&DataKey::VaultAdmin(owner.clone()))
}

pub fn read_vault_admin(e: &Env, owner: &Address) -> Address {
    e.storage()
        .persistent()
        .get(&DataKey::VaultAdmin(owner.clone()))
        .unwrap()
}

pub fn write_vault_admin(e: &Env, owner: &Address, admin: &Address) {
    e.storage()
        .persistent()
        .set(&DataKey::VaultAdmin(owner.clone()), admin);
}

pub fn write_vault_did(e: &Env, owner: &Address, did: &String) {
    e.storage()
        .persistent()
        .set(&DataKey::VaultDid(owner.clone()), did);
}

pub fn read_vault_did(e: &Env, owner: &Address) -> Option<String> {
    e.storage().persistent().get(&DataKey::VaultDid(owner.clone()))
}

pub fn read_vault_revoked(e: &Env, owner: &Address) -> bool {
    e.storage()
        .persistent()
        .get(&DataKey::VaultRevoked(owner.clone()))
        .unwrap_or(false)
}

pub fn write_vault_revoked(e: &Env, owner: &Address, revoked: &bool) {
    e.storage()
        .persistent()
        .set(&DataKey::VaultRevoked(owner.clone()), revoked);
}

// --- Vault issuers (persistent) ---

pub fn read_vault_issuers(e: &Env, owner: &Address) -> Vec<Address> {
    e.storage().persistent().get(&DataKey::VaultIssuers(owner.clone())).unwrap()
}

pub fn write_vault_issuers(e: &Env, owner: &Address, issuers: &Vec<Address>) {
    e.storage().persistent().set(&DataKey::VaultIssuers(owner.clone()), issuers)
}

// --- VC payloads (persistent) ---

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

// --- TTL extensions ---

/// Extend instance TTL (admin, fees). Call from handlers that touch global state.
pub fn extend_instance_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

/// Extend TTL of vault keys. Call when reading/writing vault.
pub fn extend_vault_ttl(e: &Env, owner: &Address) {
    let keys = [
        DataKey::VaultAdmin(owner.clone()),
        DataKey::VaultDid(owner.clone()),
        DataKey::VaultRevoked(owner.clone()),
        DataKey::VaultIssuers(owner.clone()),
        DataKey::VaultVCIds(owner.clone()),
    ];
    for key in keys {
        if e.storage().persistent().has(&key) {
            e.storage()
                .persistent()
                .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
        }
    }
}

/// Extend TTL of VC payload, index, status, owner. Call when touching a VC.
pub fn extend_vc_ttl(e: &Env, owner: &Address, vc_id: &String) {
    let vc_key = DataKey::VaultVC(owner.clone(), vc_id.clone());
    let ids_key = DataKey::VaultVCIds(owner.clone());
    let status_key = DataKey::VCStatus(vc_id.clone());
    let owner_key = DataKey::VCOwner(vc_id.clone());
    for key in [&vc_key, &ids_key, &status_key, &owner_key] {
        if e.storage().persistent().has(key) {
            e.storage()
                .persistent()
                .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
        }
    }
}

/// Extend TTL of VC status/owner only. Call from revoke flow.
pub fn extend_vc_status_ttl(e: &Env, vc_id: &String) {
    for key in [
        DataKey::VCStatus(vc_id.clone()),
        DataKey::VCOwner(vc_id.clone()),
    ] {
        if e.storage().persistent().has(&key) {
            e.storage()
                .persistent()
                .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
        }
    }
}

// --- Legacy (migration) ---

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
