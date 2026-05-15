//! Global config storage: admin, fee settings. All instance-scoped keys.

use crate::constants::PERSISTENT_TTL_EXTEND_TO;
use crate::constants::PERSISTENT_TTL_THRESHOLD;
use super::VcVaultDataKey;
use soroban_sdk::{contracttype, Address, Env};

// --- Admin ---

pub fn has_contract_admin(e: &Env) -> bool {
    e.storage().instance().has(&VcVaultDataKey::ContractAdmin)
}

pub fn read_contract_admin(e: &Env) -> Address {
    e.storage().instance().get(&VcVaultDataKey::ContractAdmin).unwrap()
}

pub fn write_contract_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&VcVaultDataKey::ContractAdmin, admin);
}

pub fn has_pending_admin(e: &Env) -> bool {
    e.storage().instance().has(&VcVaultDataKey::PendingAdmin)
}

pub fn read_pending_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&VcVaultDataKey::PendingAdmin)
}

pub fn write_pending_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&VcVaultDataKey::PendingAdmin, admin);
}

pub fn remove_pending_admin(e: &Env) {
    e.storage().instance().remove(&VcVaultDataKey::PendingAdmin);
}

// --- Fee config ---

/// Fee config status returned by `fee_config()`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeConfig {
    pub enabled: bool,
    pub configured: bool,
    pub token_contract: Option<Address>,
    pub fee_dest: Option<Address>,
    pub fee_amount: Option<i128>,
}

pub fn read_fee_enabled(e: &Env) -> bool {
    match e.storage().instance().get(&VcVaultDataKey::FeeEnabled) {
        Some(v) => v,
        None => false,
    }
}

pub fn write_fee_enabled(e: &Env, enabled: &bool) {
    e.storage().instance().set(&VcVaultDataKey::FeeEnabled, enabled);
}

pub fn write_fee_token_contract(e: &Env, addr: &Address) {
    e.storage().instance().set(&VcVaultDataKey::FeeTokenContract, addr);
}

pub fn read_fee_token_contract(e: &Env) -> Address {
    e.storage().instance().get(&VcVaultDataKey::FeeTokenContract).unwrap()
}

pub fn write_fee_dest(e: &Env, addr: &Address) {
    e.storage().instance().set(&VcVaultDataKey::FeeDest, addr);
}

pub fn read_fee_dest(e: &Env) -> Address {
    e.storage().instance().get(&VcVaultDataKey::FeeDest).unwrap()
}

pub fn write_fee_amount(e: &Env, amount: &i128) {
    e.storage().instance().set(&VcVaultDataKey::FeeAmount, amount);
}

pub fn read_fee_amount(e: &Env) -> i128 {
    e.storage().instance().get(&VcVaultDataKey::FeeAmount).unwrap()
}

pub fn try_read_fee_token_contract(e: &Env) -> Option<Address> {
    e.storage().instance().get(&VcVaultDataKey::FeeTokenContract)
}

pub fn try_read_fee_dest(e: &Env) -> Option<Address> {
    e.storage().instance().get(&VcVaultDataKey::FeeDest)
}

pub fn try_read_fee_amount(e: &Env) -> Option<i128> {
    e.storage().instance().get(&VcVaultDataKey::FeeAmount)
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
    e.storage().instance().set(&VcVaultDataKey::FeeAdmin, amount);
}

pub fn try_read_fee_admin(e: &Env) -> Option<i128> {
    e.storage().instance().get(&VcVaultDataKey::FeeAdmin)
}

pub fn read_fee_admin(e: &Env) -> i128 {
    try_read_fee_admin(e).unwrap_or(0)
}

pub fn write_fee_standard(e: &Env, amount: &i128) {
    e.storage().instance().set(&VcVaultDataKey::FeeStandard, amount);
}

pub fn try_read_fee_standard(e: &Env) -> Option<i128> {
    e.storage().instance().get(&VcVaultDataKey::FeeStandard)
}

pub fn read_fee_standard(e: &Env) -> i128 {
    try_read_fee_standard(e).unwrap_or(1_000_000)
}

pub fn write_fee_early(e: &Env, amount: &i128) {
    e.storage().instance().set(&VcVaultDataKey::FeeEarly, amount);
}

pub fn try_read_fee_early(e: &Env) -> Option<i128> {
    e.storage().instance().get(&VcVaultDataKey::FeeEarly)
}

pub fn read_fee_early(e: &Env) -> i128 {
    try_read_fee_early(e).unwrap_or(400_000)
}

pub fn write_fee_custom(e: &Env, issuer: &Address, amount: &i128) {
    let key = VcVaultDataKey::FeeCustom(issuer.clone());
    e.storage().persistent().set(&key, amount);
    e.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

pub fn try_read_fee_custom(e: &Env, issuer: &Address) -> Option<i128> {
    e.storage().persistent().get(&VcVaultDataKey::FeeCustom(issuer.clone()))
}

pub fn read_fee_custom(e: &Env, issuer: &Address) -> i128 {
    try_read_fee_custom(e, issuer).unwrap_or_else(|| read_fee_amount(e))
}
