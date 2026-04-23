//! Contract entry points for vc-issuer-registry.

use crate::error::ContractError;
use crate::storage::{self, IssuerRecord};
use soroban_sdk::{contract, contractimpl, contractmeta, panic_with_error, Address, Bytes, Env, Symbol};

const VERSION: &str = env!("CARGO_PKG_VERSION");

contractmeta!(
    key = "Description",
    val = "VC Issuer Registry: on-chain allowlist and metadata registry for VC issuers",
);

#[contract]
pub struct VcIssuerRegistryContract;

#[contractimpl]
impl VcIssuerRegistryContract {
    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------

    /// Initialize the registry. Can only be called once.
    /// `admin` must sign; it becomes the sole authority for admin-gated methods.
    pub fn initialize(e: Env, admin: Address) {
        if storage::has_admin(&e) {
            panic_with_error!(&e, ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        storage::write_admin(&e, &admin);
        storage::extend_instance_ttl(&e);
    }

    // -----------------------------------------------------------------------
    // Issuer management (admin-only)
    // -----------------------------------------------------------------------

    /// Register a new issuer with initial metadata. Fails if already registered.
    pub fn add_issuer(
        e: Env,
        issuer: Address,
        name: Option<Symbol>,
        did: Option<Bytes>,
        url: Option<Bytes>,
    ) {
        require_admin(&e);
        if storage::has_issuer(&e, &issuer) {
            panic_with_error!(&e, ContractError::IssuerAlreadyExists);
        }
        let record = IssuerRecord { allowed: true, name, did, url };
        storage::write_issuer(&e, &issuer, &record);
        storage::extend_instance_ttl(&e);
    }

    /// Update metadata for an existing issuer. Fails if not registered.
    pub fn update_issuer(
        e: Env,
        issuer: Address,
        name: Option<Symbol>,
        did: Option<Bytes>,
        url: Option<Bytes>,
    ) {
        require_admin(&e);
        let mut record = storage::read_issuer(&e, &issuer)
            .unwrap_or_else(|| panic_with_error!(&e, ContractError::IssuerNotFound));
        record.name = name;
        record.did = did;
        record.url = url;
        storage::write_issuer(&e, &issuer, &record);
        storage::extend_instance_ttl(&e);
    }

    /// Set the `allowed` flag for an issuer (enable / disable without removing).
    pub fn set_issuer_allowed(e: Env, issuer: Address, allowed: bool) {
        require_admin(&e);
        let mut record = storage::read_issuer(&e, &issuer)
            .unwrap_or_else(|| panic_with_error!(&e, ContractError::IssuerNotFound));
        record.allowed = allowed;
        storage::write_issuer(&e, &issuer, &record);
        storage::extend_instance_ttl(&e);
    }

    /// Remove an issuer from the registry entirely.
    pub fn remove_issuer(e: Env, issuer: Address) {
        require_admin(&e);
        if !storage::has_issuer(&e, &issuer) {
            panic_with_error!(&e, ContractError::IssuerNotFound);
        }
        storage::remove_issuer(&e, &issuer);
        storage::extend_instance_ttl(&e);
    }

    // -----------------------------------------------------------------------
    // Read-only queries
    // -----------------------------------------------------------------------

    /// Returns the full record for an issuer, or panics with IssuerNotFound.
    pub fn get_issuer(e: Env, issuer: Address) -> IssuerRecord {
        storage::extend_instance_ttl(&e);
        storage::read_issuer(&e, &issuer)
            .unwrap_or_else(|| panic_with_error!(&e, ContractError::IssuerNotFound))
    }

    /// Returns true if the issuer is registered and currently allowed.
    pub fn is_allowed(e: Env, issuer: Address) -> bool {
        storage::extend_instance_ttl(&e);
        storage::read_issuer(&e, &issuer)
            .map(|r| r.allowed)
            .unwrap_or(false)
    }

    /// Returns the current admin address.
    pub fn admin(e: Env) -> Address {
        if !storage::has_admin(&e) {
            panic_with_error!(&e, ContractError::NotInitialized);
        }
        storage::extend_instance_ttl(&e);
        storage::read_admin(&e)
    }

    /// Returns the contract version string.
    pub fn version(e: Env) -> soroban_sdk::String {
        soroban_sdk::String::from_str(&e, VERSION)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Panics with `Unauthorized` if the caller is not the stored admin.
/// Also panics with `NotInitialized` if initialize() was never called.
fn require_admin(e: &Env) {
    if !storage::has_admin(e) {
        panic_with_error!(e, ContractError::NotInitialized);
    }
    let admin = storage::read_admin(e);
    admin.require_auth();
}
