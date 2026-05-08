//! Contract entry points for vc-issuer-registry.

use crate::error::ContractError;
use crate::events;
use crate::storage::{self, IssuerRecord};
use soroban_sdk::{contract, contractimpl, contractmeta, panic_with_error, Address, Bytes, Env, Symbol};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum allowed byte length for `did` and `url` metadata fields.
const MAX_METADATA_BYTES: u32 = 256;

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

    /// One-time initializer. Stores the admin address. Panics if already called.
    pub fn initialize(e: Env, admin: Address) {
        if storage::has_admin(&e) {
            panic_with_error!(&e, ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        storage::write_admin(&e, &admin);
        storage::extend_instance_ttl(&e);
        events::initialized(&e, &admin);
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
        validate_metadata(&e, &did, &url);
        if storage::has_issuer(&e, &issuer) {
            panic_with_error!(&e, ContractError::IssuerAlreadyExists);
        }
        let record = IssuerRecord { allowed: true, name, did, url };
        storage::write_issuer(&e, &issuer, &record);
        storage::extend_instance_ttl(&e);
        events::issuer_added(&e, &issuer);
    }

    /// Update metadata for an existing issuer. Fails if not registered.
    /// Does **not** re-add a removed issuer — the `allowed` flag is preserved.
    pub fn set_issuer_metadata(
        e: Env,
        issuer: Address,
        name: Option<Symbol>,
        did: Option<Bytes>,
        url: Option<Bytes>,
    ) {
        require_admin(&e);
        validate_metadata(&e, &did, &url);
        let mut record = storage::read_issuer(&e, &issuer)
            .unwrap_or_else(|| panic_with_error!(&e, ContractError::IssuerNotFound));
        record.name = name;
        record.did = did;
        record.url = url;
        storage::write_issuer(&e, &issuer, &record);
        storage::extend_instance_ttl(&e);
        events::metadata_updated(&e, &issuer);
    }

    /// Set the `allowed` flag for an issuer (enable / disable without removing).
    pub fn set_issuer_allowed(e: Env, issuer: Address, allowed: bool) {
        require_admin(&e);
        let mut record = storage::read_issuer(&e, &issuer)
            .unwrap_or_else(|| panic_with_error!(&e, ContractError::IssuerNotFound));
        record.allowed = allowed;
        storage::write_issuer(&e, &issuer, &record);
        storage::extend_instance_ttl(&e);
        events::issuer_allowed_set(&e, &issuer, allowed);
    }

    /// Remove an issuer from the registry entirely.
    /// **Behavior:** the record is deleted (not soft-disabled). After removal,
    /// `is_issuer_allowed` returns `false` and `get_issuer` panics with
    /// `IssuerNotFound`.
    pub fn remove_issuer(e: Env, issuer: Address) {
        require_admin(&e);
        if !storage::has_issuer(&e, &issuer) {
            panic_with_error!(&e, ContractError::IssuerNotFound);
        }
        storage::remove_issuer(&e, &issuer);
        storage::extend_instance_ttl(&e);
        events::issuer_removed(&e, &issuer);
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
    pub fn is_issuer_allowed(e: Env, issuer: Address) -> bool {
        storage::extend_instance_ttl(&e);
        storage::read_issuer(&e, &issuer)
            .map(|r| r.allowed)
            .unwrap_or(false)
    }

    /// Returns the current admin address. Panics with NotInitialized if not set.
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

/// Panics with `NotInitialized` if no admin is stored, or with a host auth
/// error if the caller is not the stored admin.
fn require_admin(e: &Env) {
    if !storage::has_admin(e) {
        panic_with_error!(e, ContractError::NotInitialized);
    }
    let admin = storage::read_admin(e);
    admin.require_auth();
}

/// Validates metadata field sizes. Panics with `InvalidMetadata` if `did` or
/// `url` exceeds [`MAX_METADATA_BYTES`].
fn validate_metadata(e: &Env, did: &Option<Bytes>, url: &Option<Bytes>) {
    if let Some(d) = did {
        if d.len() > MAX_METADATA_BYTES {
            panic_with_error!(e, ContractError::InvalidMetadata);
        }
    }
    if let Some(u) = url {
        if u.len() > MAX_METADATA_BYTES {
            panic_with_error!(e, ContractError::InvalidMetadata);
        }
    }
}
