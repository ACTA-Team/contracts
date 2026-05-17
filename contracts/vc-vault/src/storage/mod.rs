//! Storage layout. Instance = global config; persistent = per-vault and per-VC.

mod config;
mod credential;
mod issuer;
mod ttl;
mod vault;

pub use crate::constants::*;
pub use config::*;
pub use credential::*;
pub use issuer::*;
pub use ttl::*;
pub use vault::*;

use soroban_sdk::{contracttype, Address, String};

/// Storage keys. Instance = admin, fees, flags. Persistent = vault metadata, VCs, status.
#[derive(Clone)]
#[contracttype]
pub enum VcVaultDataKey {
    // --- Contract-level (unchanged) ---
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

    // --- Vault owner ---
    VaultOwner,

    // --- Vault metadata ---
    VaultAdmin,
    VaultDid,
    VaultRevoked,

    // --- Authorized issuer O(1) index ---
    /// Number of authorized issuers.
    VaultIssuerCount,
    /// Authorized issuer at a given position (0-indexed).
    VaultIssuerIndex(u32),
    /// Position of a given authorized issuer.
    VaultIssuerPosition(Address),

    // --- Denied issuer O(1) index ---
    /// Number of denied issuers.
    VaultDeniedIssuerCount,
    /// Denied issuer at a given position (0-indexed).
    VaultDeniedIssuerIndex(u32),
    /// Position of a given denied issuer.
    VaultDeniedIssuerPosition(Address),

    // --- VC storage ---
    VaultVC(String),
    /// Number of active VCs in this vault.
    VaultVCCount,
    /// vc_id at a given position (0-indexed).
    VaultVCIndex(u32),
    /// Position of a given vc_id in the index.
    VaultVCPosition(String),
    VCStatus(String),
    VCParent(String),

}
