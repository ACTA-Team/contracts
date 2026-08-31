//! Data model for the `did-stellar-registry` contract.
//!
//! Defines the on-chain types - `DidKey`, `DidService`, `DidRecord` - and the
//! validation bounds. Bounds are enforced in `contract::validate_record` and
//! kept here so they are co-located with the types they constrain.

use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

// --- Validation bounds ---

/// Maximum length of a `publicKeyMultibase` string (chars).
pub const MAX_KEY_MULTIBASE_LEN: u32 = 128;

/// Minimum number of authentication keys (REQUIRED ≥ 1).
pub const MIN_KEY_COUNT_AUTH: u32 = 1;
/// Maximum number of authentication keys.
pub const MAX_KEY_COUNT_AUTH: u32 = 3;

/// Maximum number of assertion-method keys.
pub const MAX_KEY_COUNT_ASSERT: u32 = 3;

/// Maximum number of key-agreement keys.
pub const MAX_KEY_COUNT_AGREEMENT: u32 = 1;

/// Maximum number of services.
pub const MAX_SERVICE_COUNT: u32 = 3;

/// Maximum length of `service.id_suffix` (chars).
pub const MAX_SERVICE_ID_LEN: u32 = 32;

/// Maximum length of `service.service_type` (chars).
pub const MAX_SERVICE_TYPE_LEN: u32 = 64;

/// Maximum length of any URL stored on-chain (`service_endpoint`, `metadata_uri`).
pub const MAX_URL_LEN: u32 = 255;

// --- Types ---

/// A single public key entry in a `DidRecord`.
///
/// `public_key_multibase` follows the W3C Multikey encoding (for example,
/// `z6Mk...` for Ed25519, `z6LS...` for X25519).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DidKey {
    /// Multikey-encoded public key.
    pub public_key_multibase: String,
}

/// A service entry in a `DidRecord`.
///
/// In the resolved DID Document each service becomes
/// `{did}#service-{id_suffix}`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DidService {
    /// Lowercase alphanumeric + hyphen. Used as the fragment in the DID URL.
    pub id_suffix: String,
    /// Free string. Mapped to `type` in the resolved DID Document.
    pub service_type: String,
    /// Absolute HTTPS URL.
    pub service_endpoint: String,
}

/// On-chain DID record. One entry per DID, keyed by a 16-byte `did_id`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DidRecord {
    /// Stellar account that authorizes mutations on this DID.
    pub controller: Address,
    /// Authentication keys (1–3).
    pub authentication: Vec<DidKey>,
    /// Assertion-method keys (0–3).
    pub assertion_method: Vec<DidKey>,
    /// Key-agreement keys (0–1).
    pub key_agreement: Vec<DidKey>,
    /// Service endpoints (0–3).
    pub services: Vec<DidService>,
    /// Optional URI to extended off-chain metadata (HTTPS only).
    pub metadata_uri: Option<String>,
    /// Optional SHA-256 digest of the off-chain metadata payload.
    pub metadata_hash: Option<BytesN<32>>,
    /// Mutation counter. Starts at 1 and is incremented on every successful
    /// mutation. Used for optimistic concurrency.
    pub version: u32,
    /// Ledger number at registration. Never changes after `register`.
    pub created_ledger: u32,
    /// Ledger number of the most recent successful mutation.
    pub updated_ledger: u32,
    /// One-way deactivation flag. Once `true` it can never be reset.
    pub deactivated: bool,
}
