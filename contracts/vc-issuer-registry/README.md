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

| Function                                | Auth  | Description                             |
| --------------------------------------- | ----- | --------------------------------------- |
| `initialize(admin)`                     | admin | One-time init; stores admin             |
| `add_issuer(issuer, name, did, url)`    | admin | Register a new issuer                   |
| `update_issuer(issuer, name, did, url)` | admin | Update issuer metadata                  |
| `set_issuer_allowed(issuer, allowed)`   | admin | Toggle allowlist flag                   |
| `remove_issuer(issuer)`                 | admin | Remove issuer from registry             |
| `get_issuer(issuer)`                    | —     | Return full `IssuerRecord`              |
| `is_allowed(issuer)`                    | —     | Return `true` if registered and allowed |
| `admin()`                               | —     | Return current admin address            |
| `version()`                             | —     | Return crate version string             |

## Error codes

| Code | Variant               | Meaning                            |
| ---- | --------------------- | ---------------------------------- |
| 1    | `Unauthorized`        | Caller is not the admin            |
| 2    | `AlreadyInitialized`  | `initialize` called more than once |
| 3    | `IssuerNotFound`      | Issuer not in registry             |
| 4    | `IssuerAlreadyExists` | Issuer already registered          |
| 5    | `InvalidMetadata`     | Metadata validation failed         |
| 6    | `NotInitialized`      | Contract not yet initialized       |

## Events

| Event              | Emitted by           | Fields                           |
| ------------------ | -------------------- | -------------------------------- |
| `Initialized`      | `initialize`         | `admin: Address`                 |
| `IssuerAdded`      | `add_issuer`         | `issuer: Address`                |
| `IssuerUpdated`    | `update_issuer`      | `issuer: Address`                |
| `IssuerAllowedSet` | `set_issuer_allowed` | `issuer: Address, allowed: bool` |
| `IssuerRemoved`    | `remove_issuer`      | `issuer: Address`                |

## Build & test

```bash
# from repo root
cargo build -p vc-issuer-registry-contract
cargo test -p vc-issuer-registry-contract

# WASM
stellar contract build
```
