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

`salt` is a `BytesN<32>` chosen by the caller. Internally the factory derives the actual deploy salt as `keccak256(user_salt ‖ XDR(owner))`, which makes vault addresses deterministic per `(owner, salt)` pair and prevents frontrunning.

The vault constructor called by `deploy_v2` receives `(owner, contract_admin, did_uri, factory_address)`. The factory address is stored inside each vault so it can call `is_vault` during `receive_push`.

### Fee configuration

Fees are centralized in the factory: a vault calls `quote_fee(issuer)` at issuance time and transfers the quoted amount (in the configured token) from the issuer to the configured destination. All setters require the admin.

| Function | Auth | Description |
|---|---|---|
| `set_fee_config(token, dest, standard)` | admin | Set the fee token contract, destination, and standard amount. |
| `set_fee_enabled(enabled)` | admin | Toggle fee charging. Enabling requires token + dest + standard to be set (`FeeNotConfigured`). |
| `set_fee_standard(amount)` | admin | Update the standard fee. |
| `set_fee_custom(issuer, amount, expires_at)` | admin | Per-issuer fee override; `expires_at` (optional) must be in the future. |
| `remove_fee_custom(issuer)` | admin | Remove a per-issuer override. |
| `set_min_fee(amount)` | admin | Floor enforced on all fee amounts. |
| `quote_fee(issuer) -> FeeQuote` | none | Returns `{ enabled, amount, token, dest }`. Disabled → `{false, 0, None, None}`; otherwise the issuer's non-expired custom fee, or the standard. |

All amounts are validated to be in `[min_fee, MAX_FEE_AMOUNT]` (`MAX_FEE_AMOUNT = 10^18`) and non-negative.

### Admin

| Function | Auth | Description |
|---|---|---|
| `nominate_admin(new_admin)` | admin | Propose a successor (two-step). |
| `accept_admin()` | proposed admin | Complete the transfer. Fails with `NoPendingAdmin` if none pending. |
| `get_admin() -> Address` | none | Read the current admin. |

The factory is **immutable** — there is no `upgrade` entrypoint, and the vault template hash (`vault_init_meta`) is fixed at construction. To ship a new vault version, deploy a new factory.

---

## Events

| Event | Fields | Emitted by |
|---|---|---|
| `VaultDeployed` | `owner`, `vault_address` | `deploy` |
| `SponsoredVaultDeployed` | `deployer`, `owner`, `vault_address` | `deploy_sponsored` |
| `AdminNominated` | `current`, `nominee` | `nominate_admin` |
| `AdminTransferred` | `old_admin`, `new_admin` | `accept_admin` |
| `FeeConfigSet` | `token`, `dest`, `standard` | `set_fee_config` |
| `FeeEnabledChanged` | `enabled` | `set_fee_enabled` |
| `FeeStandardSet` | `amount` | `set_fee_standard` |
| `FeeCustomSet` | `issuer`, `amount`, `expires_at` | `set_fee_custom` |
| `FeeCustomRemoved` | `issuer` | `remove_fee_custom` |
| `MinFeeSet` | `amount` | `set_min_fee` |

---

## Error codes

| Code | Variant | Trigger |
|---:|---|---|
| 1 | `NoPendingAdmin` | `accept_admin` with no pending nomination |
| 2 | `InvalidFeeAmount` | Fee amount is negative |
| 3 | `FeeOutOfBounds` | Fee amount exceeds `MAX_FEE_AMOUNT` (10^18) |
| 4 | `FeeBelowMin` | Fee amount below the configured `min_fee` |
| 5 | `FeeNotConfigured` | `set_fee_enabled(true)` before token + dest + standard are set |
| 6 | `ExpiryInPast` | Custom fee `expires_at` is not in the future |
| 7 | `NotInitialized` | `VaultMeta` missing (constructor never ran) |

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
deploy_salt   = keccak256( user_salt (32 bytes) || XDR(owner) )
vault_address = hash( factory_address || deploy_salt )
```

`XDR(owner)` is the canonical XDR serialization of the owner `Address` (i.e. `Address.toXDR(env)` on-chain / the equivalent ScAddress XDR encoding off-chain) — **not** its StrKey display string. A client precomputing a vault address must hash the raw XDR bytes of the owner address, not the `"G..."`/`"C..."` text.

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
