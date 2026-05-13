# vc-vault

Soroban smart contract that implements a per-owner **Verifiable Credential vault** on Stellar. Each holder owns an isolated vault where authorized issuers can store, verify, and revoke credentials. The contract also acts as the issuance status registry, so a single deployment handles both storage and lifecycle management.

---

## Public ABI

### Global config

| Function | Auth | Description |
|---|---|---|
| `initialize(contract_admin)` | `contract_admin` | One-time setup. Sets the contract admin and initializes fee state. Fails with `AlreadyInitialized` if called again. |
| `nominate_admin(new_admin)` | `contract_admin` | Propose a new contract admin. Must be accepted by the nominee. |
| `accept_contract_admin()` | pending nominee | Complete the two-step admin transfer. Fails with `NoPendingAdmin` if no nomination exists. |
| `upgrade(new_wasm_hash)` | `contract_admin` | Replace the contract WASM. Irreversible per-invocation. |
| `version()` | none | Returns the crate version string (from `CARGO_PKG_VERSION`). |

### Fee configuration

Fees are charged in a token of the admin's choosing (e.g. USDC). Four tiers are available; the issuer selects the applicable tier and passes it as `fee_override` when calling `issue`.

| Function | Auth | Description |
|---|---|---|
| `set_fee_config(token, dest, amount)` | `contract_admin` | Configure the fee token contract, destination address, and default amount. |
| `set_fee_enabled(enabled)` | `contract_admin` | Toggle fee charging globally. |
| `set_fee_admin(amount)` | `contract_admin` | Set the admin-tier fee (default: 0). |
| `set_fee_standard(amount)` | `contract_admin` | Set the standard-tier fee (default: 1,000,000 stroops). |
| `set_fee_early(amount)` | `contract_admin` | Set the early-adopter-tier fee (default: 400,000 stroops). |
| `set_fee_custom(issuer, amount)` | `contract_admin` | Set a custom fee for a specific issuer address. |
| `get_fee_admin()` | none | Read the admin-tier fee amount. |
| `get_fee_standard()` | none | Read the standard-tier fee amount. |
| `get_fee_early()` | none | Read the early-adopter-tier fee amount. |
| `get_fee_custom(issuer)` | none | Read the custom fee for an issuer, falling back to the default amount if none is set. |
| `fee_config()` | none | Returns a `FeeConfig` struct with all fee state (enabled, configured, token, dest, amount). |

### Vault management

Each vault is scoped to an owner `Address`. The vault admin starts as the owner and can be transferred.

| Function | Auth | Description |
|---|---|---|
| `create_vault(owner, did_uri)` | `owner` | Initialize a vault for `owner`. Fails if the contract is not initialized or if the vault already exists. |
| `create_sponsored_vault(sponsor, owner, did_uri)` | `sponsor` | Create a vault on behalf of `owner`. Sponsor must be the contract admin or an authorized sponsor, unless `open_to_all` is enabled. |
| `set_vault_admin(owner, new_admin)` | vault admin | Transfer vault governance to `new_admin`. Emits `VaultAdminChanged`. |
| `authorize_issuer(owner, issuer)` | vault admin | Add a single issuer to the vault's allowlist. Fails if already authorized. |
| `authorize_issuers(owner, issuers)` | vault admin | Replace the full issuer list. Duplicates in the input are silently dropped. |
| `revoke_issuer(owner, issuer)` | vault admin | Remove an issuer from the allowlist and add them to the deny list. Denied issuers cannot be auto-authorized on future `issue` calls. |
| `revoke_vault(owner)` | vault admin | Permanently lock the vault against writes. Irreversible. |
| `set_sponsored_vault_open_to_all(open)` | `contract_admin` | When `true`, any address can create sponsored vaults without being explicitly authorized. |
| `get_sponsored_vault_open_to_all()` | none | Query the current open-to-all setting. |
| `add_sponsored_vault_sponsor(sponsor)` | `contract_admin` | Add an address to the authorized sponsors list. |
| `remove_sponsored_vault_sponsor(sponsor)` | `contract_admin` | Remove an address from the authorized sponsors list. |

### VC operations

| Function | Auth | Description |
|---|---|---|
| `issue(owner, vc_id, vc_data, vault_contract, issuer, issuer_did, fee_override)` | `issuer` | Store a VC in `owner`'s vault. Auto-authorizes `issuer` if not already in the allowlist and not denied. Charges `fee_override` tokens if fees are enabled and `fee_override > 0`. Returns `vc_id`. |
| `revoke(owner, vc_id, date)` | `owner` | Permanently revoke a VC. `date` is an ISO-8601 timestamp recorded on-chain. |
| `list_vc_ids(owner)` | none | Return all VC IDs currently in `owner`'s vault. |
| `get_vc(owner, vc_id)` | none | Return the `VerifiableCredential` payload, or `None` if not found. |
| `verify_vc(owner, vc_id)` | none | Return `VCStatus::Valid`, `VCStatus::Revoked(date)`, or `VCStatus::Invalid`. If the VC was issued by an external issuance contract, delegates the status check via cross-contract call. |
| `push(from_owner, to_owner, vc_id, issuer)` | `from_owner` | Move a `Valid` VC from one vault to another. `issuer` must be authorized in the source vault. Revoked VCs cannot be pushed. Emits `VCPushed`. |

### Linked VCs

Linked VCs establish a parent–child relationship between credentials across vaults.

| Function | Auth | Description |
|---|---|---|
| `issue_linked(issuer, owner, vc_id, data, issuance_contract, issuer_did, parent_owner, parent_vc_id)` | `issuer` | Issue a VC that references a parent VC. Validates that the parent is `Valid` at issuance time. |
| `get_vc_parent(owner, vc_id)` | none | Return `Some((parent_owner, parent_vc_id))` for linked VCs, or `None` for regular VCs. |

### Migration

| Function | Auth | Description |
|---|---|---|
| `migrate(owner)` | vault admin | Migrate VCs from the legacy storage format to the current format. Fails with `VCSAlreadyMigrated` if migration has already been run. |
| `migrate_vc_index(owner)` | none | Migrate the v0.1 `Vec<String>` VC index to the O(1) index. Safe for vaults with ≤ 10 legacy entries. |
| `migrate_vc_index_chunk(owner, chunk_size)` | none | Chunked variant for large legacy vaults. Migrates up to `chunk_size` entries (max 10) per call. Returns remaining count; 0 means done. Call repeatedly until 0. |
| `migrate_issuer_index(owner)` | none | Migrate legacy `Vec<Address>` issuer lists to the O(1) index. |

Vaults have no per-vault VC cap; the limit is the u32 position counter (~4.3 billion) and storage rent paid by transactors.

---

## Authorization

| Role | Who signs | What they can do |
|---|---|---|
| `contract_admin` | Contract-level administrator | Initialize, upgrade, configure fees, manage sponsors, nominate successor |
| vault admin | Per-vault administrator (starts as `owner`) | Manage issuer allowlist, revoke vault, transfer vault admin |
| `owner` | Vault owner | Create vault, revoke a VC they hold |
| `issuer` | Authorized credential issuer | Issue and push VCs |
| `sponsor` | Authorized sponsor | Create vaults on behalf of users |

Authorization is enforced via `require_auth()` on every privileged operation. Read-only functions (`list_vc_ids`, `get_vc`, `verify_vc`, `fee_config`, etc.) require no signature.

---

## Error Codes

| Code | Name | When |
|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` or `create_vault` called on an already-initialized resource |
| 2 | `IssuerNotAuthorized` | Issuer not in vault's allowlist (and not eligible for auto-auth) |
| 3 | `IssuerAlreadyAuthorized` | `authorize_issuer` called for an issuer already in the list |
| 4 | `VaultRevoked` | Write attempted on a revoked vault |
| 5 | `VCSAlreadyMigrated` | `migrate` called after migration has already been completed |
| 6 | `VCNotFound` | VC not present in the vault or status registry |
| 7 | `VCAlreadyRevoked` | `push` attempted on a revoked VC |
| 8 | `VaultNotInitialized` | Operation on a vault that has not been created |
| 9 | `NotInitialized` | Contract operation before `initialize` has been called |
| 10 | `InvalidVaultContract` | `vault_contract` param does not match this contract's address |
| 11 | `NotAuthorizedSponsor` | `create_sponsored_vault` by an address that is not authorized |
| 12 | `VCAlreadyExists` | `issue` or `push` would create a duplicate VC ID in the target vault |
| 13 | `NoPendingAdmin` | `accept_contract_admin` called with no active nomination |
| 14 | `ParentVCInvalid` | `issue_linked` called with a parent VC that does not exist or is revoked |
| 15 | `VaultFull` | u32 VC position counter would overflow (~4.3 billion VCs) |
| 16 | `LimitTooLarge` | `list_vc_ids` / `list_authorized_issuers` `limit` exceeds `MAX_LIST_LIMIT` (200) |
| 17 | `BatchTooLarge` | `batch_issue` request exceeds `MAX_BATCH_SIZE` (5) |
| 18 | `BatchEmpty` | `batch_issue` called with an empty VC list |
| 19 | `InputTooLong` | A string field exceeds its per-field length cap |
| 20 | `IssuerListTooLong` | `authorize_issuers` called with a list exceeding `MAX_ISSUERS_LIST` (100) |
| 21 | `IssuersAlreadyMigrated` | `migrate_issuer_index` called after migration is complete |
| 22 | `InvalidFeeAmount` | Fee amount is negative |
| 23 | `FeeOutOfBounds` | Fee amount exceeds `MAX_FEE_AMOUNT` (10^18 stroops) |

---

## Events

All state-changing operations emit a typed `#[contractevent]` for on-chain observability.

| Event | Fields | Emitted by |
|---|---|---|
| `VaultCreated` | `owner`, `did_uri` | `create_vault` |
| `SponsoredVaultCreated` | `sponsor`, `owner`, `did_uri` | `create_sponsored_vault` |
| `VaultRevoked` | `owner` | `revoke_vault` |
| `VaultAdminChanged` | `owner`, `old_admin`, `new_admin` | `set_vault_admin` |
| `IssuerAuthorized` | `owner`, `issuer` | `authorize_issuer`, `authorize_issuers` |
| `IssuerRevoked` | `owner`, `issuer` | `revoke_issuer` |
| `VCIssued` | `owner`, `vc_id`, `issuer` | `issue` |
| `VCRevoked` | `owner`, `vc_id`, `date` | `revoke` |
| `VCPushed` | `from_owner`, `to_owner`, `vc_id` | `push` |
| `LinkedVCIssued` | `issuer`, `owner`, `vc_id`, `parent_owner`, `parent_vc_id` | `issue_linked` |

---

## Storage layout

| Key | Type | Description |
|---|---|---|
| `ContractAdmin` (instance) | `Address` | Current contract admin |
| `PendingAdmin` (instance) | `Address` | Pending admin nomination (cleared on accept) |
| `FeeEnabled` (instance) | `bool` | Whether fee charging is active |
| `FeeTokenContract` (instance) | `Address` | Token contract used for fees |
| `FeeDest` (instance) | `Address` | Fee recipient address |
| `FeeAmount` (instance) | `i128` | Default fee amount |
| `FeeAdmin` (instance) | `i128` | Admin-tier fee |
| `FeeStandard` (instance) | `i128` | Standard-tier fee |
| `FeeEarly` (instance) | `i128` | Early-adopter-tier fee |
| `FeeCustom(issuer)` (instance) | `i128` | Custom fee per issuer |
| `SponsoredVaultOpenToAll` (instance) | `bool` | Whether sponsored vault creation is unrestricted |
| `VaultAdmin(owner)` (persistent) | `Address` | Current admin of this vault |
| `VaultDid(owner)` (persistent) | `String` | DID URI associated with the vault |
| `VaultRevoked(owner)` (persistent) | `bool` | Whether this vault is permanently revoked |
| `VaultIssuers(owner)` (persistent) | `Vec<Address>` | Authorized issuer list |
| `VaultDeniedIssuers(owner)` (persistent) | `Vec<Address>` | Denied issuer list (cannot be re-authorized automatically) |
| `VaultVC(owner, vc_id)` (persistent) | `VerifiableCredential` | VC payload |
| `VaultVCIds(owner)` (persistent) | `Vec<String>` | Index of VC IDs in this vault |
| `VCStatus(owner, vc_id)` (persistent) | `VCStatus` | `Valid`, `Revoked(date)`, or `Invalid` (default) |
| `VCParent(owner, vc_id)` (persistent) | `(Address, String)` | Parent vault + VC ID for linked credentials |
| `SponsoredVaultSponsor(sponsor)` (persistent) | `bool` | Presence flag for authorized sponsors |

TTL is extended on every read and write: ~30-day threshold, ~180-day bump for persistent entries; same for instance storage.

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

## Building

```sh
# Build optimized WASM
./scripts/build.sh
# Output: target/wasm32v1-none/release/vc_vault_contract.optimized.wasm
```

## Deploying

```sh
# Deploy to testnet (requires Stellar CLI)
./scripts/deploy.sh vc-vault-contract testnet <source-account>
```

Record the resulting contract ID in `docs/deployments/testnet.md`.
