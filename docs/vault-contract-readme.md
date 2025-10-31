# ACTA Vault Contract — Overview and Usage

This document describes each public function of the `acta_vault_contract`, the authorization model, and how the multi‑tenant vault works. It also covers how to deploy and reference the issuance contract.

**Testnet Vault Contract ID**

Note: Example ID — replace with your own deployment.

- `CDXXIA7O4PMY2CVKFJZOXSV7GYEINP3HE4JSQ4YPBFDQZ4RG5HVOTATN`

## Key Concepts

- `owner`: address that identifies the vault. All vault operations are scoped per `owner`.
- `vault admin`: address that controls the vault for a given `owner`. By default, it is the `owner` after initialization; it can be changed via `set_admin`.
- `ContractAdmin`: global address with permission to upgrade the contract code via `upgrade`. If no contract admin is set, the first `owner` that calls `initialize` becomes the `ContractAdmin`.
- `issuer`: address authorized to store VCs in the vault of `owner`.
- `did_uri`: DID URI stored for `owner` (e.g., `did:pkh:stellar:testnet:…`).
- `vc_id` and `vc_data`: unique identifier and payload of a verifiable credential.
- `issuance_contract` and `issuer_did`: VC metadata; the issuance contract is the contract ID of the issuer on chain.

## Authorization and State Rules

- Vault administration operations (`authorize_*`, `revoke_*`, `set_admin`, `revoke_vault`, `migrate`) must be signed by the vault admin recorded for that `owner`.
- Storing credentials (`store_vc`) requires the vault not to be revoked; the `issuer` must be authorized in the `owner` vault, and the `issuer` must sign the transaction.
- If a vault is revoked, operations like authorizing or revoking issuers and storing new VCs fail.

## Common Errors (codes)

- `#1 AlreadyInitialized`: trying to initialize the same `owner` twice.
- `#2 IssuerNotAuthorized`: storing a VC with an issuer not authorized or revoking a non‑existent issuer.
- `#3 IssuerAlreadyAuthorized`: authorizing an issuer that is already authorized.
- `#4 VaultRevoked`: operating on a revoked vault (authorize/revoke issuers, etc.).
- `#5 VCSAlreadyMigrated`: running `migrate` when there are no legacy VCs to migrate.

---

## initialize

Initializes the vault for an `owner`.

- Sets the vault admin to `owner` by default.
- If `ContractAdmin` is not set, it is set to `owner` (the first initializer of any vault).
- Stores the `did_uri` for `owner`.
- Marks the vault as not revoked and initializes the issuers list to empty.

Requirements:

- Fails with `#1` if the `owner` vault was already initialized.

Typical use:

- Create the vault for an `owner` and register its DID.

---

## authorize_issuers

Authorizes a list of issuers for the `owner` vault.

Effect:

- Replaces the vault’s authorized issuers list with the provided list.

Requirements:

- Must be signed by the vault admin.
- Vault must not be revoked; otherwise `#4`.

Typical use:

- Bulk load a complete configuration of authorized issuers at once.

---

## authorize_issuer

Authorizes a single `issuer` for the `owner` vault.

Effect:

- Adds the `issuer` to the vault’s authorized issuers if not already present.

Requirements:

- Must be signed by the vault admin.
- Vault must not be revoked; otherwise `#4`.
- Fails with `#3` if the issuer is already authorized.

Typical use:

- Add issuers incrementally without touching the rest of the list.

---

## revoke_issuer

Revokes an `issuer` previously authorized in the `owner` vault.

Effect:

- Removes the `issuer` from the vault’s authorized list.

Requirements:

- Must be signed by the vault admin.
- Vault must not be revoked; otherwise `#4`.
- Fails with `#2` if the issuer is not found.

Typical use:

- Remove permissions from an issuer that should no longer issue VCs for the `owner`.

---

## store_vc

Stores a verifiable credential (VC) in the `owner` vault.

Effect:

- Persists the VC under key `(owner, vc_id)` with its metadata (`issuer_did`, `issuance_contract`).

Requirements:

- Vault must not be revoked (`#4`).
- `issuer` must be authorized in the `owner` vault; otherwise `#2`.
- The `issuer` must sign the operation.

Typical use:

- Register a VC issued by an authorized issuer inside the `owner` vault.

---

## list_vc_ids

Lists all verifiable credential IDs stored in the vault for an `owner`.

Effect:

- Returns an array of VC identifiers (URIs or opaque strings) associated with `owner`.
- Read‑only; does not mutate state.

Requirements:

- Must be signed by the `owner` (authorization enforced via `owner.require_auth()`).
- Vault must be initialized and not revoked.

Typical use:

- Discover available VC IDs for an `owner` before fetching a specific credential with `get_vc`.

---

## get_vc

Fetches a single verifiable credential by `vc_id` for the given `owner`.

Effect:

- Returns the stored VC payload and metadata for the key `(owner, vc_id)`.
- Read‑only; does not mutate state.

Return shape:

- `id`: VC identifier.
- `data`: VC payload (string or structured object depending on your app).
- `issuance_contract`: on‑chain contract ID that issued the VC.
- `issuer_did`: DID of the issuer.

Requirements:

- Must be signed by the `owner` (authorization enforced via `owner.require_auth()`).
- Fails if the vault is revoked.
- Fails if no record exists for `(owner, vc_id)`.

Typical use:

- Read a specific VC for validation or off‑chain processing after discovering its `vc_id` via `list_vc_ids`.

---

## revoke_vault

Revokes the `owner` vault.

Effect:

- Marks the vault as revoked; no more issuers can be authorized nor new VCs stored.

Requirements:

- Must be signed by the vault admin.
- Fails with `#4` if the vault is already revoked.

Typical use:

- Lock down the vault for security or compliance reasons.

---

## migrate

Migrates the `owner` vault VCs from the legacy single vector to the current `vc_id`‑keyed storage.

Effect:

- Re‑stores all legacy VCs into the new schema and removes the legacy vector.

Requirements:

- Must be signed by the vault admin.
- Fails with `#5` if there are no legacy VCs to migrate.

Typical use:

- Run once after an upgrade that changes storage schema.

---

## set_admin

Changes the vault admin for `owner`.

Effect:

- Updates the vault admin to `new_admin` for the given `owner`.

Requirements:

- Must be signed by the current vault admin.

Typical use:

- Transfer operational control of the vault to another address (e.g., a custodian or multisig).

---

## upgrade

Upgrades the contract WASM code.

Effect:

- Replaces the contract’s WASM with the one identified by `new_wasm_hash`.

Requirements:

- Must be signed by the `ContractAdmin` (not the per‑vault admin).

Typical use:

- Deploy a new version of the contract with fixes or improvements.

---

## version

Returns the Cargo package version compiled into the contract.

Typical use:

- Check which contract version is running on the network.

---

## Data Visibility and Privacy

- Contract state on Soroban is public. Authorization rules control writes/executions, not raw reads.
- Do not store PII in plaintext. Prefer off‑chain storage and on‑chain anchoring via hashes, or store ciphertext if necessary.
- Recommended pattern: store `sha256(vc_data)` and minimal metadata; keep full VC off‑chain. If you store `vc_data`, encrypt it off‑chain and store only the ciphertext.

---

## Issuance Contract

- `issuance_contract` is the on‑chain contract ID of the issuer that originated the VC. It is stored as metadata for traceability and future cross‑verification.
- Today, the vault does not call or validate the issuance contract at `store_vc`; it just records the reference.

**Current Testnet Issuance Contract ID**

- `CANYEUDJCAPQ5ACXXJQXR4VA6727LFGFP2FFE35MF3YEQTXCMIA7BNWA`

### Deploying the Issuance Contract (Testnet)

1. Build and deploy your issuance contract (e.g., `acta_issuance_contract`).
2. Note its contract ID (`C…`) and keep it in configuration (e.g., `.env`), so your app can pass it to `store_vc`.
3. Multiple users can reference the same issuance contract; deployment is one‑time per issuer/organization, not per credential.

### CLI Example: Initialize, Store, List and Get VC (Testnet)

- Initialize vault for `owner`:
  `soroban contract invoke --id CDXXIA7O4PMY2CVKFJZOXSV7GYEINP3HE4JSQ4YPBFDQZ4RG5HVOTATN --network testnet -- initialize --owner G...OWNER --did_uri did:pkh:stellar:testnet:G...OWNER`
- Authorize an issuer:
  `soroban contract invoke --id CDXXIA7O4PMY2CVKFJZOXSV7GYEINP3HE4JSQ4YPBFDQZ4RG5HVOTATN --network testnet -- authorize_issuer --owner G...OWNER --issuer G...ISSUER`
- Store a VC:
  `soroban contract invoke --id CDXXIA7O4PMY2CVKFJZOXSV7GYEINP3HE4JSQ4YPBFDQZ4RG5HVOTATN --network testnet -- store_vc --owner G...OWNER --vc_id vc-123 --vc_data '{"name":"Alice"}' --issuer G...ISSUER --issuer_did did:pkh:stellar:testnet:G...ISSUER --issuance_contract CANYEUDJCAPQ5ACXXJQXR4VA6727LFGFP2FFE35MF3YEQTXCMIA7BNWA`

- List VC IDs for an owner:
  `soroban contract invoke --id CDXXIA7O4PMY2CVKFJZOXSV7GYEINP3HE4JSQ4YPBFDQZ4RG5HVOTATN --network testnet -- list_vc_ids --owner G...OWNER`

- Get a VC by id:
  `soroban contract invoke --id CDXXIA7O4PMY2CVKFJZOXSV7GYEINP3HE4JSQ4YPBFDQZ4RG5HVOTATN --network testnet -- get_vc --owner G...OWNER --vc_id vc-123`

Note: For read operations (`list_vc_ids`, `get_vc`), ensure you sign as the `owner` (e.g., set a default identity or pass `--source-account` with the owner’s key) because `owner.require_auth()` is enforced.

---

## Best Practices

- Align signer with role: vault admin signs administration; issuer signs `store_vc`.
- Manage issuers: use `authorize_issuers` for bulk setup and `authorize_issuer`/`revoke_issuer` for incremental changes.
- Migrate once when storage schema changes.
- Distinguish roles: `set_admin` affects a specific vault; `upgrade` requires the global `ContractAdmin`.
