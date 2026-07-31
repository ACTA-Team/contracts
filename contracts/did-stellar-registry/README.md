# did-stellar-registry

On-chain Soroban registry for the [`did:stellar`](../../docs/did-spec/did-stellar-v0.1.md) DID method (v0.1).

This contract is the canonical source of truth for the state of every `did:stellar` identifier on a given network. The DID resolver reads `DidRecord` directly from this contract via Stellar RPC; no indexer is required.

---

## Public ABI

### Constructor

| Function | Purpose |
|---|---|
| `__constructor(admin: Address)` | Runs once at deploy. Sets the contract admin. Deployer MUST sign as `admin`. Emits `ContractInitialized`. |

### Per-DID operations

| Function | Purpose |
|---|---|
| `register(did_id: BytesN<16>, initial_record: DidRecord)` | Create a new DID. Fails if `did_id` is already taken. |
| `register_sponsored(sponsor: Address, did_id: BytesN<16>, initial_record: DidRecord)` | Create a new DID paid for by `sponsor`, controlled by `initial_record.controller`. Only `sponsor` signs. Fails with `SponsorIsController` if the two are the same address. |
| `update(did_id: BytesN<16>, expected_version: u32, next_record: DidRecord)` | Replace the full DID record. Fails on version mismatch or if the DID is deactivated. |
| `transfer_controller(did_id: BytesN<16>, expected_version: u32, new_controller: Address)` | Change the controller. Keys, services, and metadata are preserved. |
| `deactivate(did_id: BytesN<16>, expected_version: u32)` | Permanently deactivate the DID. Empties cryptographic material; preserves controller + metadata for audit. Irreversible. |
| `get(did_id: BytesN<16>) -> Option<DidRecord>` | Read the current record. No authorization required. |

All mutations require `controller.require_auth()`, except `register_sponsored`, which requires `sponsor.require_auth()` and no signature from the controller. All mutations except `register` use optimistic concurrency: `expected_version` MUST equal the current on-chain version, or the call is rejected with `VersionMismatch`.

### Contract-level admin

Two-step admin transfer. Per-DID mutations are NOT admin-gated - the admin role exists for future contract-wide governance only.

| Function | Purpose |
|---|---|
| `propose_admin(new_admin: Address)` | Current admin nominates a successor. Proposal lives in temporary storage and auto-expires (~10 days). |
| `accept_admin()` | Proposed admin accepts the role. Both the current admin (already past) and the proposed admin must have signed the two calls. Emits `AdminTransferred`. Fails with `NoProposedAdmin` if no proposal exists. |
| `get_admin() -> Address` | Read the current admin. No authorization required. |

---

## Authorization

| Function | Authorizing party |
|---|---|
| `__constructor` | `admin` (deployer signs) |
| `register` | `initial_record.controller` |
| `register_sponsored` | `sponsor` only - the controller does NOT sign |
| `update` | current `controller` |
| `transfer_controller` | current `controller` |
| `deactivate` | current `controller` |
| `get` | none (read-only) |
| `propose_admin` | current `admin` |
| `accept_admin` | proposed admin |
| `get_admin` | none (read-only) |

---

## Error codes

Codes are part of the ABI. Numeric values MUST NOT be renumbered.

| Code | Variant | Trigger |
|---:|---|---|
| 1 | `DidAlreadyExists` | `register` called for an existing `did_id`. |
| 2 | `DidNotFound` | Mutation on a non-existent DID. |
| 3 | `VersionMismatch` | `expected_version` ≠ current on-chain version. |
| 4 | `DidDeactivated` | Mutation on an already-deactivated DID. |
| 5 | `InvalidAuthKeyCount` | `authentication.len()` outside [1, 3]. |
| 6 | `InvalidAssertionKeyCount` | `assertion_method.len()` > 3. |
| 7 | `InvalidKeyAgreementCount` | `key_agreement.len()` > 1. |
| 8 | `InvalidServiceCount` | `services.len()` > 3. |
| 9 | `DuplicateKey` | Same `public_key_multibase` repeated within one relationship. |
| 10 | `KeyTooLong` | `public_key_multibase.len()` > 128 chars. |
| 11 | `KeyEmpty` | `public_key_multibase` is empty. |
| 12 | `ServiceTypeTooLong` | `service_type.len()` > 64 chars. |
| 13 | `ServiceIdTooLong` | `id_suffix.len()` > 32 chars. |
| 14 | `ServiceIdInvalidFormat` | `id_suffix` does not match `^[a-z0-9][a-z0-9-]*[a-z0-9]$` (or a single `[a-z0-9]`); leading/trailing hyphens rejected. |
| 15 | `ServiceEndpointInvalid` | `service_endpoint` is not `https://...` or > 255 chars. |
| 16 | `MetadataUriInvalid` | `metadata_uri` is not `https://...` or > 255 chars. |
| 17 | `NoProposedAdmin` | `accept_admin` called when no proposal exists or proposal expired. |
| 18 | `ServiceTypeEmpty` | `service_type` is empty. |
| 19 | `VersionOverflow` | DID `version` has reached `u32::MAX`; further mutations are rejected. |
| 20 | `MetadataInconsistent` | `metadata_hash` is set but `metadata_uri` is absent. |
| 21 | `DuplicateServiceId` | Two services in the same record share the same `id_suffix`. |
| 22 | `SponsorIsController` | `register_sponsored` called with `sponsor == initial_record.controller`. Use `register` instead. |

---

## Events

Each successful mutation emits a typed event:

| Event | Payload | Triggered by |
|---|---|---|
| `DidRegistered` | `did_id`, `controller`, `version` | `register` |
| `DidRegisteredSponsored` | `did_id`, `sponsor`, `controller`, `version` | `register_sponsored` |
| `DidUpdated` | `did_id`, `version` | `update` |
| `DidControllerTransferred` | `did_id`, `old_controller`, `new_controller`, `version` | `transfer_controller` |
| `DidDeactivated` | `did_id`, `version` | `deactivate` |
| `ContractInitialized` | `admin` | `__constructor` (once) |
| `AdminTransferred` | `old_admin`, `new_admin` | `accept_admin` |

Events use the `#[contractevent]` macro.

---

## Storage layout

### Per-DID records (persistent)

One persistent entry per DID, keyed by the 16-byte `did_id`:

```rust
pub enum DidDataKey {
    Record(BytesN<16>),
}
```

TTL is extended on every read AND every write (~30-day threshold, ~180-day bump). A DID that is regularly resolved or mutated stays alive without any explicit rent extension call.

### Contract admin

| Symbol key | Storage type | Lifetime |
|---|---|---|
| `Admin` | instance | bumps with contract activity (~30-day threshold, ~90-day bump) |
| `PropAdmin` | temporary | ~10 days; an unaccepted proposal expires automatically |

---

## Validation bounds

Defined in `src/model.rs`:

| Field | Limit |
|---|---|
| `authentication.len()` | 1–3 |
| `assertion_method.len()` | 0–3 |
| `key_agreement.len()` | 0–1 |
| `services.len()` | 0–3 |
| `public_key_multibase` | 1–128 chars; unique within each relationship |
| `service.id_suffix` | 1–32 chars; `^[a-z0-9][a-z0-9-]*[a-z0-9]$` (or single char); unique across services |
| `service.service_type` | 1–64 chars |
| `service.service_endpoint` | `https://`, ≤ 255 chars |
| `metadata_uri` | `https://`, ≤ 255 chars |
| `metadata_hash` | 32 bytes (SHA-256) |
| `metadata_hash` + `metadata_uri` | If `metadata_hash` is set, `metadata_uri` must also be set. |

`http://` is rejected for both `service_endpoint` and `metadata_uri`.

---

## Build & test

```bash
# build the workspace
cargo build

# run all tests for this contract
cargo test -p did-stellar-registry

# build the WASM artifact
stellar contract build

./scripts/build.sh
```

---

## References

- [`did:stellar` v0.1 specification](../../docs/did-spec/did-stellar-v0.1.md)
- [Test vectors](../../docs/did-spec/test-vectors/vectors.json)
- W3C DID Core 1.1 - https://www.w3.org/TR/did-1.1/
- W3C DID Resolution v0.3 - https://www.w3.org/TR/did-resolution/
