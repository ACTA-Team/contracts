# vc-issuer-registry

A Stellar smart contract that provides an on-chain allowlist and metadata registry for Verifiable Credential issuers.

## Overview

`vc-issuer-registry` is a standalone contract that separates issuer governance from VC storage. It answers the question: _"Is this address allowed to issue credentials?"_ — without coupling that logic to `vc-vault`.

## Storage layout

| Key               | Storage type | Description                               |
| ----------------- | ------------ | ----------------------------------------- |
| `Admin`           | Instance     | Contract admin address                    |
| `Issuer(Address)` | Persistent   | `IssuerRecord` for each registered issuer |

### IssuerRecord

```rust
pub struct IssuerRecord {
    pub allowed: bool,
    pub name: Option<Symbol>,
    pub did: Option<Bytes>,
    pub url: Option<Bytes>,
}
```

## Entry points

| Function                                        | Auth  | Description                                |
| ----------------------------------------------- | ----- | ------------------------------------------ |
| `initialize(admin)`                             | admin | One-time init; stores admin                |
| `add_issuer(issuer, name, did, url)`            | admin | Register a new issuer (allowed = true)     |
| `set_issuer_metadata(issuer, name, did, url)`   | admin | Update metadata; preserves `allowed` flag  |
| `set_issuer_allowed(issuer, allowed)`           | admin | Toggle allowlist flag                      |
| `remove_issuer(issuer)`                         | admin | **Delete** issuer record from registry     |
| `get_issuer(issuer)`                            | —     | Return full `IssuerRecord` (panics if missing) |
| `is_issuer_allowed(issuer)`                     | —     | Return `true` if registered **and** allowed |
| `admin()`                                       | —     | Return current admin address               |
| `version()`                                     | —     | Return crate version string                |

### Removal semantics

`remove_issuer` **deletes** the on-chain record entirely (hard delete). After removal:

- `is_issuer_allowed` returns `false`.
- `get_issuer` panics with `IssuerNotFound`.
- The same address can be re-added later with `add_issuer`.

This behaviour keeps storage costs predictable and avoids zombie records.

### Metadata validation

`did` and `url` fields are bounded to a maximum of **256 bytes** each.
`add_issuer` and `set_issuer_metadata` will panic with `InvalidMetadata` if
either field exceeds this limit. `name` uses the `Symbol` type which is already
bounded by the Soroban host.

### set_issuer_metadata safety

`set_issuer_metadata` updates **only** the metadata fields (`name`, `did`, `url`).
It does **not** modify the `allowed` flag. In particular, calling it on a
disabled issuer will **not** re-enable it.

## Error codes

| Code | Variant               | Meaning                                |
| ---- | --------------------- | -------------------------------------- |
| 1    | `AlreadyInitialized`  | `initialize` called more than once     |
| 2    | `IssuerNotFound`      | Issuer not in registry                 |
| 3    | `IssuerAlreadyExists` | Issuer already registered              |
| 4    | `NotInitialized`      | Contract not yet initialized           |
| 5    | `InvalidMetadata`     | Metadata field exceeds max byte length |

## Events

| Event              | Emitted by             | Fields                           |
| ------------------ | ---------------------- | -------------------------------- |
| `Initialized`      | `initialize`           | `admin: Address`                 |
| `IssuerAdded`      | `add_issuer`           | `issuer: Address`                |
| `MetadataUpdated`  | `set_issuer_metadata`  | `issuer: Address`                |
| `IssuerAllowedSet` | `set_issuer_allowed`   | `issuer: Address, allowed: bool` |
| `IssuerRemoved`    | `remove_issuer`        | `issuer: Address`                |

## Build & test

```bash
# from repo root
cargo build -p vc-issuer-registry-contract
cargo test -p vc-issuer-registry-contract

# WASM
stellar contract build
```
