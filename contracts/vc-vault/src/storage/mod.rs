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

/// Storage keys. Instance = admin, flags. Persistent = vault metadata, VCs, status.
#[derive(Clone)]
#[contracttype]
pub enum VcVaultDataKey {
    // --- Contract-level ---
    ContractAdmin,
    PendingAdmin,

    // --- Vault owner ---
    VaultOwner,

    // --- Factory that deployed this vault ---
    VaultFactory,

    // --- Vault metadata ---
    VaultAdmin,
    VaultDid,
    VaultRevoked,

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
