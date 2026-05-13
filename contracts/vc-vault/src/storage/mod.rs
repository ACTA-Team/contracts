//! Storage layout and helpers. Instance = global config; persistent = per-owner and per-VC.

use crate::error::ContractError;
use crate::model::{VCStatus, VerifiableCredential};
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Map, String, Vec};

// TTL constants at ~5-second ledger close: 518_400 ≈ 30 days, 3_110_400 ≈ 180 days.
const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
const INSTANCE_TTL_EXTEND_TO: u32 = 3_110_400;
const PERSISTENT_TTL_THRESHOLD: u32 = 518_400;
const PERSISTENT_TTL_EXTEND_TO: u32 = 3_110_400;

/// Maximum number of active VCs that may be tracked in a single vault. Hitting
/// this cap prevents new issuance until VCs are revoked or pushed away.
pub const MAX_VCS_PER_VAULT: u32 = 1_000;

/// Maximum number of vc_ids that may be returned by a single `list_vc_ids`
/// call. Each slot read costs ~3-5k instructions in Soroban; capping at 200
/// keeps the worst-case enumeration well under the 1.4M instruction budget
/// while still allowing the full `MAX_VCS_PER_VAULT` to be retrieved in a
/// handful of paginated calls. Callers should request `vc_count(owner)` to
/// size their iteration.
pub const MAX_LIST_LIMIT: u32 = 200;

/// Maximum number of VCs that may be issued in a single `batch_issue` call.
/// Each VC writes 4 ledger entries (`VaultVC`, `VaultVCIndex`,
/// `VaultVCPosition`, `VCStatus`); plus 1 shared write to `VaultVCCount`.
/// At the cap of 5 the batch touches 21 ledger entries, leaving headroom
/// for the optional fee transfer (token, source-balance, dest-balance ≈ 3
/// entries) under Soroban's ~25 entries-per-transaction default.
pub const MAX_BATCH_SIZE: u32 = 5;

// --- Per-field input length caps ---
//
// Caps user-controlled string inputs at every write entrypoint to bound
// storage rent and CPU cost. Reads that take a vc_id are also capped so an
// attacker can't force the contract to spend instructions on a 1MB key
// before the lookup misses.
//
// Numbers chosen with a 4-10× safety margin over realistic values:
// - vc_id: typical UUIDs are 36 chars; 64 covers prefixed schemes like
//   `urn:uuid:...`.
// - vc_data: encrypted credential payloads typically 1-5KB; 10KB allows
//   complex schemas without inviting state bloat.
// - did_uri / issuer_did: longest realistic DIDs (`did:pkh:stellar:...:G...`)
//   are ~60 chars; 256 is a comfortable upper bound aligned with the
//   did:stellar v0.1 spec.
// - date: ISO 8601 timestamps are 20-30 chars; 64 is sufficient.

/// Maximum bytes for `vc_id` strings.
pub const MAX_VC_ID_LEN: u32 = 64;
/// Maximum bytes for `vc_data` payloads.
pub const MAX_VC_DATA_LEN: u32 = 10_000;
/// Maximum bytes for vault `did_uri`.
pub const MAX_DID_URI_LEN: u32 = 256;
/// Maximum bytes for `issuer_did`.
pub const MAX_ISSUER_DID_LEN: u32 = 256;
/// Maximum bytes for revocation `date` strings (ISO 8601).
pub const MAX_DATE_LEN: u32 = 64;
/// Maximum number of addresses accepted by `authorize_issuers(list)`.
pub const MAX_ISSUERS_LIST: u32 = 100;

/// Storage keys. Instance = admin, fees, flags. Persistent = vault metadata, VCs, status.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    ContractAdmin,
    PendingAdmin,
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
    /// **Legacy.** `Vec<Address>` issuer list; read only by `migrate_issuer_index`.
    VaultIssuers(Address),
    /// **Legacy.** `Vec<Address>` denied issuer list; read only by `migrate_issuer_index`.
    VaultDeniedIssuers(Address),
    /// Number of authorized issuers. O(1) read.
    VaultIssuerCount(Address),
    /// Authorized issuer at a given position (0-indexed).
    VaultIssuerIndex(Address, u32),
    /// Position of a given authorized issuer. Used for O(1) existence and removal.
    VaultIssuerPosition(Address, Address),
    /// Number of denied issuers. O(1) read.
    VaultDeniedIssuerCount(Address),
    /// Denied issuer at a given position (0-indexed).
    VaultDeniedIssuerIndex(Address, u32),
    /// Position of a given denied issuer. Used for O(1) existence and removal.
    VaultDeniedIssuerPosition(Address, Address),
    VaultVC(Address, String),
    /// **Legacy v0.1 index.** A monolithic `Vec<String>` of vc_ids per vault.
    /// New issuance writes to the O(1) index keys below; this entry is read
    /// only by `migrate_vc_index(owner)` to bridge data forward.
    VaultVCIds(Address),
    /// Number of active VCs in this vault. O(1) read.
    VaultVCCount(Address),
    /// vc_id at a given position (0-indexed). Used to enumerate the index.
    VaultVCIndex(Address, u32),
    /// Position of a given vc_id in the index. Used for O(1) existence and removal.
    VaultVCPosition(Address, String),
    VCStatus(Address, String),
    VCParent(Address, String),
    LegacyIssuanceRevocations,
    LegacyIssuanceVCs,
    LegacyVaultVCs(Address),
    SponsoredVaultOpenToAll,
    SponsoredVaultSponsor(Address),
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

pub fn has_pending_admin(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::PendingAdmin)
}

pub fn read_pending_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::PendingAdmin)
}

pub fn write_pending_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::PendingAdmin, admin);
}

pub fn remove_pending_admin(e: &Env) {
    e.storage().instance().remove(&DataKey::PendingAdmin);
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
    let key = DataKey::FeeCustom(issuer.clone());
    e.storage().persistent().set(&key, amount);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn try_read_fee_custom(e: &Env, issuer: &Address) -> Option<i128> {
    e.storage().persistent().get(&DataKey::FeeCustom(issuer.clone()))
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
// Expected maximum: ~20 issuers per vault. Reads and existence checks are O(n);
// larger lists increase CPU cost linearly. Enforce a cap at the call-site if needed.

pub fn read_vault_issuers(e: &Env, owner: &Address) -> Vec<Address> {
    e.storage().persistent().get(&DataKey::VaultIssuers(owner.clone())).unwrap()
}

pub fn write_vault_issuers(e: &Env, owner: &Address, issuers: &Vec<Address>) {
    e.storage().persistent().set(&DataKey::VaultIssuers(owner.clone()), issuers)
}

// --- Vault denied issuers (persistent) ---

pub fn read_vault_denied_issuers(e: &Env, owner: &Address) -> Vec<Address> {
    e.storage()
        .persistent()
        .get(&DataKey::VaultDeniedIssuers(owner.clone()))
        .unwrap_or_else(|| Vec::new(e))
}

pub fn write_vault_denied_issuers(e: &Env, owner: &Address, denied: &Vec<Address>) {
    e.storage()
        .persistent()
        .set(&DataKey::VaultDeniedIssuers(owner.clone()), denied);
}

pub fn is_issuer_denied(e: &Env, owner: &Address, issuer: &Address) -> bool {
    read_vault_denied_issuers(e, owner).contains(issuer.clone())
}

pub fn add_denied_issuer(e: &Env, owner: &Address, issuer: &Address) {
    let mut denied = read_vault_denied_issuers(e, owner);
    if !denied.contains(issuer.clone()) {
        denied.push_front(issuer.clone());
        write_vault_denied_issuers(e, owner, &denied);
    }
}

pub fn remove_denied_issuer(e: &Env, owner: &Address, issuer: &Address) {
    let mut denied = read_vault_denied_issuers(e, owner);
    if let Some(idx) = denied.first_index_of(issuer.clone()) {
        denied.remove(idx);
        write_vault_denied_issuers(e, owner, &denied);
    }
}

pub fn has_legacy_vault_issuers(e: &Env, owner: &Address) -> bool {
    e.storage().persistent().has(&DataKey::VaultIssuers(owner.clone()))
}

pub fn remove_legacy_vault_issuers(e: &Env, owner: &Address) {
    e.storage().persistent().remove(&DataKey::VaultIssuers(owner.clone()));
}

pub fn has_legacy_vault_denied_issuers(e: &Env, owner: &Address) -> bool {
    e.storage().persistent().has(&DataKey::VaultDeniedIssuers(owner.clone()))
}

pub fn remove_legacy_vault_denied_issuers(e: &Env, owner: &Address) {
    e.storage().persistent().remove(&DataKey::VaultDeniedIssuers(owner.clone()));
}

// --- Authorized issuer index ---

pub fn read_issuer_count(e: &Env, owner: &Address) -> u32 {
    e.storage()
        .persistent()
        .get(&DataKey::VaultIssuerCount(owner.clone()))
        .unwrap_or(0)
}

pub fn write_issuer_count(e: &Env, owner: &Address, count: u32) {
    let key = DataKey::VaultIssuerCount(owner.clone());
    e.storage().persistent().set(&key, &count);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn read_issuer_at(e: &Env, owner: &Address, position: u32) -> Option<Address> {
    e.storage()
        .persistent()
        .get(&DataKey::VaultIssuerIndex(owner.clone(), position))
}

pub fn read_issuer_at_extend(e: &Env, owner: &Address, position: u32) -> Option<Address> {
    let key = DataKey::VaultIssuerIndex(owner.clone(), position);
    if !e.storage().persistent().has(&key) {
        return None;
    }
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    e.storage().persistent().get(&key)
}

pub fn write_issuer_at(e: &Env, owner: &Address, position: u32, issuer: &Address) {
    let key = DataKey::VaultIssuerIndex(owner.clone(), position);
    e.storage().persistent().set(&key, issuer);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_issuer_at(e: &Env, owner: &Address, position: u32) {
    e.storage()
        .persistent()
        .remove(&DataKey::VaultIssuerIndex(owner.clone(), position));
}

pub fn read_issuer_position(e: &Env, owner: &Address, issuer: &Address) -> Option<u32> {
    e.storage()
        .persistent()
        .get(&DataKey::VaultIssuerPosition(owner.clone(), issuer.clone()))
}

pub fn write_issuer_position(e: &Env, owner: &Address, issuer: &Address, position: u32) {
    let key = DataKey::VaultIssuerPosition(owner.clone(), issuer.clone());
    e.storage().persistent().set(&key, &position);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_issuer_position(e: &Env, owner: &Address, issuer: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::VaultIssuerPosition(owner.clone(), issuer.clone()));
}

pub fn issuer_index_contains(e: &Env, owner: &Address, issuer: &Address) -> bool {
    e.storage()
        .persistent()
        .has(&DataKey::VaultIssuerPosition(owner.clone(), issuer.clone()))
}

pub fn append_issuer_to_index(e: &Env, owner: &Address, issuer: &Address) {
    let count = read_issuer_count(e, owner);
    if count >= MAX_ISSUERS_LIST {
        panic_with_error!(e, ContractError::IssuerListTooLong);
    }
    write_issuer_at(e, owner, count, issuer);
    write_issuer_position(e, owner, issuer, count);
    write_issuer_count(e, owner, count + 1);
}

pub fn remove_issuer_from_index(e: &Env, owner: &Address, issuer: &Address) {
    let position = match read_issuer_position(e, owner, issuer) {
        Some(p) => p,
        None => return,
    };
    let count = read_issuer_count(e, owner);
    if count == 0 {
        return;
    }
    let last = count - 1;
    if position != last {
        if let Some(last_addr) = read_issuer_at(e, owner, last) {
            write_issuer_at(e, owner, position, &last_addr);
            write_issuer_position(e, owner, &last_addr, position);
        }
    }
    remove_issuer_at(e, owner, last);
    remove_issuer_position(e, owner, issuer);
    write_issuer_count(e, owner, last);
}

pub fn clear_issuer_index(e: &Env, owner: &Address) {
    let count = read_issuer_count(e, owner);
    for i in 0..count {
        if let Some(addr) = read_issuer_at(e, owner, i) {
            remove_issuer_position(e, owner, &addr);
        }
        remove_issuer_at(e, owner, i);
    }
    write_issuer_count(e, owner, 0);
}

// --- Denied issuer index ---

pub fn read_denied_issuer_count(e: &Env, owner: &Address) -> u32 {
    e.storage()
        .persistent()
        .get(&DataKey::VaultDeniedIssuerCount(owner.clone()))
        .unwrap_or(0)
}

pub fn write_denied_issuer_count(e: &Env, owner: &Address, count: u32) {
    let key = DataKey::VaultDeniedIssuerCount(owner.clone());
    e.storage().persistent().set(&key, &count);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn read_denied_issuer_at(e: &Env, owner: &Address, position: u32) -> Option<Address> {
    e.storage()
        .persistent()
        .get(&DataKey::VaultDeniedIssuerIndex(owner.clone(), position))
}

pub fn read_denied_issuer_at_extend(e: &Env, owner: &Address, position: u32) -> Option<Address> {
    let key = DataKey::VaultDeniedIssuerIndex(owner.clone(), position);
    if !e.storage().persistent().has(&key) {
        return None;
    }
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    e.storage().persistent().get(&key)
}

pub fn write_denied_issuer_at(e: &Env, owner: &Address, position: u32, issuer: &Address) {
    let key = DataKey::VaultDeniedIssuerIndex(owner.clone(), position);
    e.storage().persistent().set(&key, issuer);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_denied_issuer_at(e: &Env, owner: &Address, position: u32) {
    e.storage()
        .persistent()
        .remove(&DataKey::VaultDeniedIssuerIndex(owner.clone(), position));
}

pub fn read_denied_issuer_position(e: &Env, owner: &Address, issuer: &Address) -> Option<u32> {
    e.storage()
        .persistent()
        .get(&DataKey::VaultDeniedIssuerPosition(owner.clone(), issuer.clone()))
}

pub fn write_denied_issuer_position(e: &Env, owner: &Address, issuer: &Address, position: u32) {
    let key = DataKey::VaultDeniedIssuerPosition(owner.clone(), issuer.clone());
    e.storage().persistent().set(&key, &position);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_denied_issuer_position(e: &Env, owner: &Address, issuer: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::VaultDeniedIssuerPosition(owner.clone(), issuer.clone()));
}

pub fn denied_issuer_index_contains(e: &Env, owner: &Address, issuer: &Address) -> bool {
    e.storage()
        .persistent()
        .has(&DataKey::VaultDeniedIssuerPosition(owner.clone(), issuer.clone()))
}

/// Append an issuer to the denied index. O(1). No cap enforced (mirrors add_denied_issuer).
pub fn append_denied_issuer_to_index(e: &Env, owner: &Address, issuer: &Address) {
    if denied_issuer_index_contains(e, owner, issuer) {
        return;
    }
    let count = read_denied_issuer_count(e, owner);
    write_denied_issuer_at(e, owner, count, issuer);
    write_denied_issuer_position(e, owner, issuer, count);
    write_denied_issuer_count(e, owner, count + 1);
}

/// Remove an issuer from the denied index using swap-and-pop. O(1).
pub fn remove_denied_issuer_from_index(e: &Env, owner: &Address, issuer: &Address) {
    let position = match read_denied_issuer_position(e, owner, issuer) {
        Some(p) => p,
        None => return,
    };
    let count = read_denied_issuer_count(e, owner);
    if count == 0 {
        return;
    }
    let last = count - 1;
    if position != last {
        if let Some(last_addr) = read_denied_issuer_at(e, owner, last) {
            write_denied_issuer_at(e, owner, position, &last_addr);
            write_denied_issuer_position(e, owner, &last_addr, position);
        }
    }
    remove_denied_issuer_at(e, owner, last);
    remove_denied_issuer_position(e, owner, issuer);
    write_denied_issuer_count(e, owner, last);
}

/// Clear the entire denied issuer index. O(n).
pub fn clear_denied_issuer_index(e: &Env, owner: &Address) {
    let count = read_denied_issuer_count(e, owner);
    for i in 0..count {
        if let Some(addr) = read_denied_issuer_at(e, owner, i) {
            remove_denied_issuer_position(e, owner, &addr);
        }
        remove_denied_issuer_at(e, owner, i);
    }
    write_denied_issuer_count(e, owner, 0);
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

// --- O(1) VC index (v0.2+) ---
//
// Three persistent keys per vault back the index:
//   VaultVCCount(owner)                 -> u32 of active VCs
//   VaultVCIndex(owner, position)       -> vc_id at that slot
//   VaultVCPosition(owner, vc_id)       -> slot of that vc_id
//
// Append, remove, and existence-check are all O(1); enumeration is O(n) but
// each step is a single ledger-entry read (constant per item) instead of a
// monolithic Vec deserialization. Removal uses swap-and-pop to keep slots dense.

pub fn read_vc_count(e: &Env, owner: &Address) -> u32 {
    e.storage()
        .persistent()
        .get(&DataKey::VaultVCCount(owner.clone()))
        .unwrap_or(0)
}

pub fn write_vc_count(e: &Env, owner: &Address, count: u32) {
    let key = DataKey::VaultVCCount(owner.clone());
    e.storage().persistent().set(&key, &count);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn read_vc_id_at(e: &Env, owner: &Address, position: u32) -> Option<String> {
    e.storage()
        .persistent()
        .get(&DataKey::VaultVCIndex(owner.clone(), position))
}

/// Read a slot and refresh its TTL in a single call. Use this from enumeration
/// paths (e.g. `list_vc_ids`) so that callers who only ever list — without
/// touching individual VCs — keep the index alive. Returns None if the slot
/// has been archived/never written.
pub fn read_vc_id_at_extend(e: &Env, owner: &Address, position: u32) -> Option<String> {
    let key = DataKey::VaultVCIndex(owner.clone(), position);
    if !e.storage().persistent().has(&key) {
        return None;
    }
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    e.storage().persistent().get(&key)
}

pub fn write_vc_id_at(e: &Env, owner: &Address, position: u32, vc_id: &String) {
    let key = DataKey::VaultVCIndex(owner.clone(), position);
    e.storage().persistent().set(&key, vc_id);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_vc_id_at(e: &Env, owner: &Address, position: u32) {
    e.storage()
        .persistent()
        .remove(&DataKey::VaultVCIndex(owner.clone(), position));
}

pub fn read_vc_position(e: &Env, owner: &Address, vc_id: &String) -> Option<u32> {
    e.storage()
        .persistent()
        .get(&DataKey::VaultVCPosition(owner.clone(), vc_id.clone()))
}

pub fn write_vc_position(e: &Env, owner: &Address, vc_id: &String, position: u32) {
    let key = DataKey::VaultVCPosition(owner.clone(), vc_id.clone());
    e.storage().persistent().set(&key, &position);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_vc_position(e: &Env, owner: &Address, vc_id: &String) {
    e.storage()
        .persistent()
        .remove(&DataKey::VaultVCPosition(owner.clone(), vc_id.clone()));
}

/// Returns true when vc_id has a recorded position in the active index.
pub fn vc_index_contains(e: &Env, owner: &Address, vc_id: &String) -> bool {
    e.storage()
        .persistent()
        .has(&DataKey::VaultVCPosition(owner.clone(), vc_id.clone()))
}

/// Append vc_id to the index. O(1). Panics with `VaultFull` if the cap is hit.
/// Caller must guarantee vc_id is not already indexed (issuance/push paths
/// already check `read_vault_vc(...).is_some()`).
pub fn append_vc_to_index(e: &Env, owner: &Address, vc_id: &String) {
    let count = read_vc_count(e, owner);
    if count >= MAX_VCS_PER_VAULT {
        panic_with_error!(e, ContractError::VaultFull);
    }
    write_vc_id_at(e, owner, count, vc_id);
    write_vc_position(e, owner, vc_id, count);
    write_vc_count(e, owner, count + 1);
}

/// Remove vc_id from the index using swap-and-pop. O(1). No-op when vc_id is
/// not indexed (defensive — callers may invoke this on already-cleared VCs).
pub fn remove_vc_from_index(e: &Env, owner: &Address, vc_id: &String) {
    let position = match read_vc_position(e, owner, vc_id) {
        Some(p) => p,
        None => return,
    };
    let count = read_vc_count(e, owner);
    // Defensive: count must be > 0 if we found a position.
    if count == 0 {
        return;
    }
    let last = count - 1;
    if position != last {
        // Move the last element into the freed slot so positions stay dense.
        if let Some(last_id) = read_vc_id_at(e, owner, last) {
            write_vc_id_at(e, owner, position, &last_id);
            write_vc_position(e, owner, &last_id, position);
        }
    }
    remove_vc_id_at(e, owner, last);
    remove_vc_position(e, owner, vc_id);
    write_vc_count(e, owner, last);
}

// --- Legacy v0.1 index (read-only; consumed by migrate_vc_index) ---

/// Read the legacy `Vec<String>` index for a vault. Returns an empty Vec when
/// no legacy entry exists (either never written or already migrated).
pub fn read_legacy_vault_vc_ids(e: &Env, owner: &Address) -> Vec<String> {
    match e.storage().persistent().get(&DataKey::VaultVCIds(owner.clone())) {
        Some(v) => v,
        None => Vec::new(e),
    }
}

/// Returns true if the legacy index entry exists for this vault.
pub fn has_legacy_vault_vc_ids(e: &Env, owner: &Address) -> bool {
    e.storage()
        .persistent()
        .has(&DataKey::VaultVCIds(owner.clone()))
}

/// Delete the legacy `VaultVCIds` entry. Called by `migrate_vc_index` after
/// the new O(1) index has been populated.
pub fn remove_legacy_vault_vc_ids(e: &Env, owner: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::VaultVCIds(owner.clone()));
}

// --- VC parent links (persistent) ---

/// Write a parent link: (owner, vc_id) → (parent_owner, parent_vc_id).
pub fn write_vc_parent(
    e: &Env,
    owner: &Address,
    vc_id: &String,
    parent_owner: &Address,
    parent_vc_id: &String,
) {
    let key = DataKey::VCParent(owner.clone(), vc_id.clone());
    e.storage()
        .persistent()
        .set(&key, &(parent_owner.clone(), parent_vc_id.clone()));
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

/// Read the parent link for a VC. Returns None if the VC has no parent.
pub fn read_vc_parent(e: &Env, owner: &Address, vc_id: &String) -> Option<(Address, String)> {
    e.storage()
        .persistent()
        .get(&DataKey::VCParent(owner.clone(), vc_id.clone()))
}

/// Return true if the VC has a recorded parent link.
pub fn has_vc_parent(e: &Env, owner: &Address, vc_id: &String) -> bool {
    e.storage()
        .persistent()
        .has(&DataKey::VCParent(owner.clone(), vc_id.clone()))
}

/// Remove a parent link entry. Used by `push` so the link follows the VC into
/// the destination vault and doesn't get orphaned at the source.
pub fn remove_vc_parent(e: &Env, owner: &Address, vc_id: &String) {
    e.storage()
        .persistent()
        .remove(&DataKey::VCParent(owner.clone(), vc_id.clone()));
}

/// VC status keyed by (owner, vc_id) to prevent cross-vault collisions.
pub fn write_vc_status(e: &Env, owner: &Address, vc_id: &String, status: &VCStatus) {
    e.storage()
        .persistent()
        .set(&DataKey::VCStatus(owner.clone(), vc_id.clone()), status)
}

pub fn read_vc_status(e: &Env, owner: &Address, vc_id: &String) -> VCStatus {
    e.storage()
        .persistent()
        .get(&DataKey::VCStatus(owner.clone(), vc_id.clone()))
        .unwrap_or(VCStatus::Invalid)
}

/// Remove the status entry. After removal the default `Invalid` is returned by
/// `read_vc_status`. Used by `push` so the source vault no longer reports a
/// `Valid` status for a VC whose payload has moved away — otherwise stale
/// status reads can spoof `issue_linked` into accepting orphan parents.
pub fn remove_vc_status(e: &Env, owner: &Address, vc_id: &String) {
    e.storage()
        .persistent()
        .remove(&DataKey::VCStatus(owner.clone(), vc_id.clone()));
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
        DataKey::VaultIssuerCount(owner.clone()),
        DataKey::VaultDeniedIssuerCount(owner.clone()),
        DataKey::VaultVCCount(owner.clone()),
    ];
    for key in keys {
        if e.storage().persistent().has(&key) {
            e.storage()
                .persistent()
                .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
        }
    }
}

/// Extend TTL of VC payload, status, and the per-VC index entries. Call when
/// touching a VC. Index entries (`VaultVCIndex`, `VaultVCPosition`) are kept
/// alive only via reads/writes of the VC itself; they are not extended in
/// `extend_vault_ttl` because each is a distinct ledger entry per position.
pub fn extend_vc_ttl(e: &Env, owner: &Address, vc_id: &String) {
    let vc_key = DataKey::VaultVC(owner.clone(), vc_id.clone());
    let status_key = DataKey::VCStatus(owner.clone(), vc_id.clone());
    let position_key = DataKey::VaultVCPosition(owner.clone(), vc_id.clone());
    for key in [&vc_key, &status_key, &position_key] {
        if e.storage().persistent().has(key) {
            e.storage()
                .persistent()
                .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
        }
    }
    // Extend the index slot entry that points to this vc_id, if present.
    if let Some(pos) = read_vc_position(e, owner, vc_id) {
        let index_key = DataKey::VaultVCIndex(owner.clone(), pos);
        if e.storage().persistent().has(&index_key) {
            e.storage()
                .persistent()
                .extend_ttl(&index_key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
        }
    }
}

/// Extend TTL of VC status only. Call from revoke flow.
pub fn extend_vc_status_ttl(e: &Env, owner: &Address, vc_id: &String) {
    let key = DataKey::VCStatus(owner.clone(), vc_id.clone());
    if e.storage().persistent().has(&key) {
        e.storage()
            .persistent()
            .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    }
}

// --- Sponsored vault config ---

pub fn read_sponsored_vault_open_to_all(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&DataKey::SponsoredVaultOpenToAll)
        .unwrap_or(false)
}

pub fn write_sponsored_vault_open_to_all(e: &Env, open: &bool) {
    e.storage()
        .instance()
        .set(&DataKey::SponsoredVaultOpenToAll, open);
}

/// Check if an address is an authorized sponsor.
pub fn is_authorized_sponsor(e: &Env, sponsor: &Address) -> bool {
    e.storage()
        .persistent()
        .has(&DataKey::SponsoredVaultSponsor(sponsor.clone()))
}

pub fn add_sponsored_vault_sponsor(e: &Env, sponsor: &Address) {
    let key = DataKey::SponsoredVaultSponsor(sponsor.clone());
    e.storage().persistent().set(&key, &true);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_sponsored_vault_sponsor(e: &Env, sponsor: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::SponsoredVaultSponsor(sponsor.clone()));
}

// --- Legacy (migration) ---

pub fn read_legacy_issuance_vcs(e: &Env) -> Option<Vec<String>> {
    e.storage().persistent().get(&DataKey::LegacyIssuanceVCs)
}

pub fn remove_legacy_issuance_vcs(e: &Env) {
    e.storage().persistent().remove(&DataKey::LegacyIssuanceVCs);
}

pub fn read_legacy_issuance_revocations(e: &Env) -> Map<String, LegacyRevocation> {
    e.storage()
        .persistent()
        .get(&DataKey::LegacyIssuanceRevocations)
        .unwrap_or_else(|| Map::new(e))
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
