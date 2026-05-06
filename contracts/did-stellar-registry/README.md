# did-stellar-registry

On-chain Soroban registry for the [`did:stellar`](../../docs/did-spec/did-stellar-v0.1.md) DID method (v0.1).

This contract is the canonical source of truth for the state of every `did:stellar` identifier on a given network. The DID resolver reads `DidRecord` directly from this contract via Stellar RPC; no indexer is required.

---

## Public ABI

### Per-DID operations

| Function | Purpose |
|---|---|
| `register(did_id: BytesN<16>, initial_record: DidRecord)` | Create a new DID. Fails if `did_id` is already taken. |
| `update(did_id: BytesN<16>, expected_version: u32, next_record: DidRecord)` | Replace the full DID record. Fails on version mismatch or if the DID is deactivated. |
| `transfer_controller(did_id: BytesN<16>, expected_version: u32, new_controller: Address)` | Change the controller. Keys, services, and metadata are preserved. |
| `deactivate(did_id: BytesN<16>, expected_version: u32)` | Permanently deactivate the DID. Empties cryptographic material; preserves controller + metadata for audit. Irreversible. |
| `get(did_id: BytesN<16>) -> Option<DidRecord>` | Read the current record. No authorization required. |

All mutations require `controller.require_auth()`. All mutations except `register` use optimistic concurrency: `expected_version` MUST equal the current on-chain version, or the call is rejected with `VersionMismatch`.

The auto-generated client struct is `DidStellarRegistryClient`.

---

## Authorization

| Function | Authorizing party |
|---|---|
| `register` | `initial_record.controller` |
| `update` | current `controller` |
| `transfer_controller` | current `controller` |
| `deactivate` | current `controller` |
| `get` | none (read-only) |

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
| 12 | `ServiceTypeTooLong` | `service_type` empty or > 64 chars. |
| 13 | `ServiceIdTooLong` | `id_suffix.len()` > 32 chars. |
| 14 | `ServiceIdInvalidFormat` | `id_suffix` does not match `^[a-z0-9-]+$`. |
| 15 | `ServiceEndpointInvalid` | `service_endpoint` is not `https://...` or > 255 chars. |
| 16 | `MetadataUriInvalid` | `metadata_uri` is not `https://...` or > 255 chars. |

---

## Events

Each successful mutation emits a typed event:

| Event | Payload | Triggered by |
|---|---|---|
| `DidRegistered` | `did_id`, `controller`, `version` | `register` |
| `DidUpdated` | `did_id`, `version` | `update` |
| `DidControllerTransferred` | `did_id`, `old_controller`, `new_controller`, `version` | `transfer_controller` |
| `DidDeactivated` | `did_id`, `version` | `deactivate` |

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
| `service.id_suffix` | 1–32 chars; `^[a-z0-9-]+$` |
| `service.service_type` | 1–64 chars |
| `service.service_endpoint` | `https://`, ≤ 255 chars |
| `metadata_uri` | `https://`, ≤ 255 chars |
| `metadata_hash` | 32 bytes (SHA-256) |

`http://` is rejected for both `service_endpoint` and `metadata_uri`.

---

## Build & test

```bash
# build the workspace
cargo build

# run all tests for this contract
cargo test -p did-stellar-registry

# build the WASM artifact
./scripts/build.sh
```

---

## References

- [`did:stellar` v0.1 specification](../../docs/did-spec/did-stellar-v0.1.md)
- [Test vectors](../../docs/did-spec/test-vectors/vectors.json)
- W3C DID Core 1.1 — https://www.w3.org/TR/did-1.1/
- W3C DID Resolution v0.3 — https://www.w3.org/TR/did-resolution/
