//! Sponsored vault config storage.

use crate::constants::{PERSISTENT_TTL_EXTEND_TO, PERSISTENT_TTL_THRESHOLD};
use super::DataKey;
use soroban_sdk::{Address, Env};

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
