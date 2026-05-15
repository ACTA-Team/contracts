//! Contract-wide constants: TTL windows and per-field input limits.

// TTL constants at ~5-second ledger close: 518_400 ≈ 30 days, 3_110_400 ≈ 180 days.
pub const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
pub const INSTANCE_TTL_EXTEND_TO: u32 = 3_110_400;
pub const PERSISTENT_TTL_THRESHOLD: u32 = 518_400;
pub const PERSISTENT_TTL_EXTEND_TO: u32 = 3_110_400;

/// Maximum number of vc_ids that may be returned by a single `list_vc_ids`
/// call. Each slot read costs ~3-5k instructions in Soroban; capping at 200
/// keeps the worst-case enumeration well under the 1.4M instruction budget
/// while still allowing the full vault to be retrieved in a handful of
/// paginated calls. Callers should request `vc_count(owner)` to size their
/// iteration.
pub const MAX_LIST_LIMIT: u32 = 200;

/// Maximum number of VCs that may be issued in a single `batch_issue` call.
/// Each VC writes 4 ledger entries (`VaultVC`, `VaultVCIndex`,
/// `VaultVCPosition`, `VCStatus`); plus 1 shared write to `VaultVCCount`.
/// At the cap of 5 the batch touches 21 ledger entries, leaving headroom
/// for the optional fee transfer (token, source-balance, dest-balance ≈ 3
/// entries) under Soroban's ~25 entries-per-transaction default.
pub const MAX_BATCH_SIZE: u32 = 5;

// Per-field input length caps.
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
/// Maximum accepted fee amount in stroops (10^18).
pub const MAX_FEE_AMOUNT: i128 = 1_000_000_000_000_000_000;
