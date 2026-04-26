//! Storage layout and helpers.
//! Instance storage  → admin (global config, low-frequency reads).
//! Persistent storage → per-issuer records (long-lived, keyed by Address).

use soroban_sdk::{contracttype, Address, Bytes, Env, Symbol};

// TTL constants (~5 s ledger close): 518_400 ≈ 30 days, 3_110_400 ≈ 180 days.
const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
const INSTANCE_TTL_EXTEND_TO: u32 = 3_110_400;
const PERSISTENT_TTL_THRESHOLD: u32 = 518_400;
const PERSISTENT_TTL_EXTEND_TO: u32 = 3_110_400;

/// Storage keys separated by role (explicit role isolation).
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Global admin (singleton, instance storage)
    Admin,

    /// Issuer registry (per-address persistent storage)
    Issuer(Address),
}

/// On-chain metadata for a registered issuer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerRecord {
    /// Whether this issuer is currently on the allowlist.
    pub allowed: bool,
    /// Human-readable name (optional).
    pub name: Option<Symbol>,
    /// DID document bytes (optional).
    pub did: Option<Bytes>,
    /// URL bytes (optional).
    pub url: Option<Bytes>,
}

// --- Admin (instance) ---

pub fn has_admin(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Admin)
}

pub fn read_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn write_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

// --- Issuer records (persistent) ---

pub fn has_issuer(e: &Env, issuer: &Address) -> bool {
    e.storage().persistent().has(&DataKey::Issuer(issuer.clone()))
}

pub fn read_issuer(e: &Env, issuer: &Address) -> Option<IssuerRecord> {
    e.storage().persistent().get(&DataKey::Issuer(issuer.clone()))
}

pub fn write_issuer(e: &Env, issuer: &Address, record: &IssuerRecord) {
    let key = DataKey::Issuer(issuer.clone());
    e.storage().persistent().set(&key, record);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn remove_issuer(e: &Env, issuer: &Address) {
    e.storage().persistent().remove(&DataKey::Issuer(issuer.clone()));
}

// --- TTL helpers ---

pub fn extend_instance_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}
