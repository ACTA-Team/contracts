# vc-vault

Soroban smart contract implementing a **single-tenant Verifiable Credential vault** on Stellar. Each holder gets their **own** vault instance (one contract per identity), deployed by [`vc-vault-factory`](../vc-vault-factory/README.md). A vault stores credentials, tracks their issuance status, and handles revocation and cross-vault migration.

Issuance is **open** (deny-by-exception): anyone may deposit a credential into a vault unless they are on the vault's denylist. Fees are **not** configured here — at issuance the vault asks the factory for a quote (`quote_fee`) and transfers the fee accordingly. See the [factory README](../vc-vault-factory/README.md#fee-configuration) for fee config.

> `verify_vc` returns the on-chain **status** of a credential (Valid / Revoked / Invalid). It is a revocation signal only, **not** proof of authenticity — because issuance is open, integrators MUST verify the issuer's signature and resolve the issuer DID off-chain before trusting a credential.

---

## Public ABI

### Deployment

A vault is normally deployed by the factory, which calls the constructor below. The constructor sets `vault_admin = vault_owner`.

| Function | Auth | Description |
|---|---|---|
| `__constructor(vault_owner, contract_admin, did_uri, factory_address)` | *(deployer / factory)* | Runs once at deploy. Stores the owner, admin, DID URI, and the factory address used to quote fees and validate push sources. |

### Global config

| Function | Auth | Description |
|---|---|---|
| `nominate_admin(new_admin)` | `contract_admin` | Propose a new contract admin (two-step). |
| `accept_contract_admin()` | pending nominee | Complete the admin transfer. Fails with `NoPendingAdmin` if none pending. |
| `version()` | none | Returns the crate version string. |

### Vault management

| Function | Auth | Description |
|---|---|---|
| `set_vault_admin(new_admin)` | vault admin | Transfer vault governance. Emits `VaultAdminChanged`. |
| `set_vault_did(did_uri)` | `vault_owner` | Update the vault's DID URI (≤ 256 bytes). Emits `VaultDidChanged`. |
| `vault_did()` | none | Returns the vault's DID URI, or `None`. |
| `vault_owner()` | none | Returns the vault owner address. |
| `deny_issuer(issuer_addr)` | vault admin | Add an issuer to the denylist. Denied issuers cannot `issue`. No-op if already present. |
| `allow_issuer(issuer_addr)` | vault admin | Remove an issuer from the denylist. No-op if absent. |
| `revoke_vault()` | vault admin | Permanently lock the vault against all writes. Irreversible. |
| `list_denied_issuers(offset, limit)` | none | Paginated denylist. `limit` ≤ `MAX_LIST_LIMIT` (200). |
| `denied_issuer_count()` | none | Number of denied issuers. |

### Credential operations

| Function | Auth | Description |
|---|---|---|
| `issue(vc_id, vc_data, vault_contract, issuer_addr, issuer_did) -> vc_id` | `issuer_addr` | Store a VC. `vault_contract` must equal this contract's address. Fails if the issuer is denied (`IssuerDenied`), the vault is revoked, or `vc_id` already exists. Charges the factory-quoted fee. |
| `batch_issue(issuer_addr, vault_contract, issuer_did, vcs) -> Vec<vc_id>` | `issuer_addr` | Issue 1–`MAX_BATCH_SIZE` (5) VCs atomically. `vcs` is a list of `(vc_id, vc_data)`. One fee transfer covers the batch (`unit × n`). |
| `revoke(vc_id, date)` | `vault_owner` | Revoke a `Valid` VC. `date` is an ISO-8601 timestamp recorded on-chain. |
| `push(vc_id, dest_vault)` | vault admin | Migrate a `Valid` VC to `dest_vault` (same owner only). Invokes `receive_push` on the destination. Emits `VCPushed`. |
| `receive_push(source_vault, source_owner, vc_id, vc_data, issuer_did)` | `source_vault` | Callback target of `push`. Verifies `source_vault` is a real vault (factory `is_vault`) and that `source_owner` matches this vault's owner (`PushOwnerMismatch`). |
| `verify_vc(vc_id) -> VCStatus` | none | On-chain status: `Valid` / `Revoked(date)` / `Invalid`. |
| `get_vc(vc_id) -> Option<VerifiableCredential>` | none | Returns the VC payload, or `None`. |
| `list_vc_ids(offset, limit) -> Vec<String>` | none | Paginated list of active VC IDs. `limit` ≤ 200. |
| `vc_count() -> u32` | none | Number of active VCs. |

---

## Authorization

| Role | Who signs | Capabilities |
|---|---|---|
| `contract_admin` | protocol administrator | Nominate a successor admin (two-step). No upgrade, no fee config. |
| vault admin | per-vault admin (starts as `vault_owner`) | `set_vault_admin`, `deny_issuer`, `allow_issuer`, `revoke_vault`, `push` |
| `vault_owner` | the holder | `set_vault_did`, `revoke` (a credential) |
| issuer | any address not on the denylist | `issue`, `batch_issue` |

Authorization is enforced via `require_auth()` on every privileged operation. Read-only functions require no signature.

---

## Error codes

Codes are part of the ABI and are never renumbered. Codes `2`, `3`, and `20` are **deprecated** (from the former issuer-allowlist model) and retained only for ABI stability.

| Code | Name | When |
|---:|---|---|
| 4 | `VaultRevoked` | Write attempted on a revoked vault |
| 6 | `VCNotFound` | VC not present / not `Valid` for the operation |
| 7 | `VCAlreadyRevoked` | Defined; revoke guards on non-`Valid` status |
| 8 | `VaultNotInitialized` | Vault state missing (should not occur in practice) |
| 9 | `NotInitialized` | Contract-level op before the admin is set |
| 10 | `InvalidVaultContract` | `vault_contract` param ≠ this contract's address |
| 12 | `VCAlreadyExists` | `issue` / `batch_issue` / `receive_push` would duplicate a `vc_id` |
| 13 | `NoPendingAdmin` | `accept_contract_admin` with no nomination |
| 15 | `VaultFull` | Active-VC counter would overflow `u32` |
| 16 | `LimitTooLarge` | Pagination `limit` exceeds `MAX_LIST_LIMIT` (200) |
| 17 | `BatchTooLarge` | `batch_issue` exceeds `MAX_BATCH_SIZE` (5) |
| 18 | `BatchEmpty` | `batch_issue` with an empty list |
| 19 | `InputTooLong` | A string field exceeds its per-field cap (vc_id ≤ 64, vc_data ≤ 10_000, DIDs ≤ 256, date ≤ 64) |
| 23 | `FeeOutOfBounds` | Batch fee total (`unit × n`) overflowed `i128` |
| 24 | `SourceNotAVault` | `receive_push` source is not registered in the factory |
| 25 | `IssuerDenied` | Issuer is on this vault's denylist |
| 26 | `PushOwnerMismatch` | `receive_push` source owner ≠ this vault's owner |

---

## Events

All state-changing operations emit a typed `#[contractevent]`.

| Event | Fields | Emitted by |
|---|---|---|
| `ContractInitialized` | `admin` | *(constructor)* |
| `VaultCreated` | `owner`, `did_uri` | *(constructor)* |
| `AdminNominated` | `current`, `new_admin` | `nominate_admin` |
| `AdminTransferred` | `old_admin`, `new_admin` | `accept_contract_admin` |
| `VaultAdminChanged` | `old_admin`, `new_admin` | `set_vault_admin` |
| `VaultDidChanged` | `did_uri` | `set_vault_did` |
| `IssuerDenied` | `issuer` | `deny_issuer` |
| `IssuerAllowed` | `issuer` | `allow_issuer` |
| `VaultRevoked` | — | `revoke_vault` |
| `VCIssued` | `vc_id`, `issuer` | `issue`, `batch_issue`, `receive_push` |
| `VCRevoked` | `vc_id`, `date` | `revoke` |
| `VCPushed` | `vc_id`, `dest_vault` | `push` |

---

## Data types

```rust
pub struct VerifiableCredential {
    pub id: String,
    pub data: String,          // ciphertext only — never store plaintext PII
    pub issuance_contract: Address,
    pub issuer_did: String,
}

pub enum VCStatus {
    Valid,
    Invalid,
    Revoked(String),           // ISO-8601 revocation date
}
```

---

## Storage

Single-tenant: keys are **not** namespaced by owner. Instance storage holds the contract admin/pending admin; persistent storage holds the vault metadata (`VaultOwner`, `VaultFactory`, `VaultAdmin`, `VaultDid`, `VaultRevoked`), the active-VC index (`VaultVC(vc_id)`, `VaultVCCount`, `VaultVCIndex(pos)`, `VaultVCPosition(vc_id)`), per-VC status (`VCStatus(vc_id)`), and the denylist (`VaultDeniedIssuerCount`, `VaultDeniedIssuerIndex(pos)`, `VaultDeniedIssuerPosition(addr)`). TTL is extended on every read and write (~30-day threshold, ~180-day bump).

---

## Build & deploy

Since v0.4.0 the vault is a **template**: it is not deployed standalone — the factory instantiates vaults. To publish the template:

```sh
./scripts/build.sh vc-vault                 # optimized WASM
./scripts/deploy.sh vc-vault testnet <src>  # install template, prints the WASM hash
```

Then deploy the factory with that hash (see the [factory README](../vc-vault-factory/README.md)); users create vaults via `factory.deploy(...)`.
