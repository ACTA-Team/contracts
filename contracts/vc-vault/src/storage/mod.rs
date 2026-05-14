//! Storage layout. Instance = global config; persistent = per-owner and per-VC.
//!
//! The DataKey enum and FeeConfig struct live here. All domain helpers are in
//! sub-modules, re-exported flat so callers use `storage::read_vault_vc(...)`.

mod config;
mod credential;
mod issuer;
mod sponsor;
mod ttl;
mod vault;

pub use crate::constants::*;
pub use config::*;
pub use credential::*;
pub use issuer::*;
pub use sponsor::*;
pub use ttl::*;
pub use vault::*;

use soroban_sdk::{contracttype, Address, String};

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
    /// Number of active VCs in this vault. O(1) read.
    VaultVCCount(Address),
    /// vc_id at a given position (0-indexed). Used to enumerate the index.
    VaultVCIndex(Address, u32),
    /// Position of a given vc_id in the index. Used for O(1) existence and removal.
    VaultVCPosition(Address, String),
    VCStatus(Address, String),
    VCParent(Address, String),
    SponsoredVaultOpenToAll,
    SponsoredVaultSponsor(Address),
}
