# vc-vault-factory

Soroban smart contract that deploys and tracks **single-tenant `vc-vault` instances** on Stellar. Each holder gets their own vault contract — one deployment per identity — rather than sharing a single multi-tenant contract. The factory derives deterministic vault addresses from `(owner, salt)` and maintains a registry used by vaults to validate cross-vault VC transfers.

---

## Public ABI

### Deployment

| Function | Auth | Description |
|---|---|---|
| `__constructor(vault_init_meta)` | *(deployer)* | Runs once at deploy. Stores the vault WASM hash and the contract admin that will be set on every deployed vault. |

`VaultInitMeta` fields:

| Field | Type | Description |
|---|---|---|
| `vault_hash` | `BytesN<32>` | WASM hash of the `vc-vault` contract to deploy |
| `contract_admin` | `Address` | Admin address passed to every new vault's constructor |

### Factory functions

| Function | Auth | Description |
|---|---|---|
| `deploy(owner, did_uri, salt)` | `owner` | Deploy a new `vc-vault` for `owner`. Returns the new vault address. |
| `deploy_sponsored(deployer, owner, did_uri, salt)` | `deployer` | Deploy a vault on behalf of `owner`. The deployer signs and pays; the vault belongs to `owner` from creation. Anyone can be a deployer — no whitelist. |
| `is_vault(vault_address)` | none | Return `true` if `vault_address` was deployed by this factory. Used by `receive_push` inside vaults to validate transfer sources. |

`salt` is a `BytesN<32>` chosen by the caller. Internally the factory derives the actual deploy salt as `keccak256(user_salt ‖ owner_bytes)`, which makes vault addresses deterministic per `(owner, salt)` pair and prevents frontrunning.

The vault constructor called by `deploy_v2` receives `(owner, contract_admin, did_uri, factory_address)`. The factory address is stored inside each vault so it can call `is_vault` during `receive_push`.

---

## Events

| Event | Fields | Emitted by |
|---|---|---|
| `VaultDeployed` | `owner`, `vault_address` | `deploy` |
| `SponsoredVaultDeployed` | `deployer`, `owner`, `vault_address` | `deploy_sponsored` |

---

## Storage layout

| Key | Type | TTL |
|---|---|---|
| `VaultMeta` (instance) | `VaultInitMeta` | threshold 30 days, bump 31 days |
| `Contracts(vault_address)` (persistent) | `bool` | threshold 100 days, bump 120 days |

Instance TTL is extended on every call to `deploy`, `deploy_sponsored`, and `is_vault`.
Persistent entries for deployed vault addresses are extended on `set_deployed` and on each `is_vault` lookup.

---

## Address derivation

Vault addresses are deterministic and unique per `(owner, salt)` pair:

```
deploy_salt = keccak256(user_salt || owner_address_bytes)
vault_address = hash(factory_address || deploy_salt)
```

Two different owners using the same user salt get different vault addresses. The same owner using different salts also gets different addresses. This means a vault address can be pre-computed client-side before submitting a transaction.

---

## Building

```sh
cargo build -p vc-vault-factory-contract --target wasm32v1-none --release
stellar contract build --optimize --manifest-path contracts/vc-vault-factory/Cargo.toml
```

## Testing

```sh
cargo test -p vc-vault-factory-contract
```
