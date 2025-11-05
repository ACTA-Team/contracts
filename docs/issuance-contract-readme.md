# ACTA Issuance Contract — Overview and Usage

This guide describes each public function of the `acta_issuance_contract`, its authorization model, how it interacts with the Vault, and examples for testnet usage.

Network IDs (examples — replace with your own deployments if different)

- Testnet — Issuance: `CABQIR63YVKUBRSTEZFCXKJAR43PNF27WL2TAU67U5YMNABKZEWPHRDN`
- Testnet — Vault: `CDK642PLEPCQH7WUBLHYYSKRJZOUIRIPY7GXQRHOETGR2JJ76UK6SWLZ`
- Mainnet — Issuance: `CAIHPYSATKLN7WL4ERMX2HCEND4BXOWATH4ETYOJSY4MB7YRNY6L7TYC`
- Mainnet — Vault: `CBWXA3XCP7DHIDIFEPUVNKCXKN27KMYQBKKRMSEA25EEDIKRKRTTQZQ4`

## Key Concepts

- `admin`: address that controls the issuance contract. Must sign sensitive actions.
- `issuer_did`: issuer’s DID stored in the contract for traceability (e.g., `did:pkh:stellar:testnet:G...`).
- `vc_id` and `vc_data`: identifier and encrypted payload of the credential.
- `VCStatus`: on-chain VC status tracked by issuance: `valid`, `revoked(date)`, or `invalid`.
- `vault_contract`: contract ID of the holder’s Vault where the VC payload is stored.

## Authorization and State Rules

- Sensitive actions (`initialize`, `issue`, `revoke`, `migrate`, `upgrade`, `set_admin`) require the current `admin` signature (`require_auth`).
- `issue` internally calls `store_vc` on the holder’s Vault, using the issuance `admin` as the `issuer`. The Vault must:
  - be initialized for the `owner`;
  - have the `issuer` (issuance admin) authorized;
  - not be revoked.
- The contract stores local VC status (`VCStatus`) to power `verify` and prevent duplicate revocations.

## Common Errors (codes)

- `#1 AlreadyInitialized`: attempt to initialize a contract that already has an `admin`.
- `#2 VCNotFound`: attempt to revoke a non-existent/invalid VC.
- `#3 VCAlreadyRevoked`: attempt to revoke a VC that is already revoked.
- `#4 VCSAlreadyMigrated`: run `migrate` when there are no legacy VCs to migrate.

---

## initialize

Initialize the issuance contract with the `admin` and `issuer_did`.

- Effect: sets `admin` and stores `issuer_did`.
- Requirements: fails with `#1` if already initialized.
- Typical use: prepare the contract to issue VCs.

Example (testnet):
`soroban contract invoke --id CABQIR63YVKUBRSTEZFCXKJAR43PNF27WL2TAU67U5YMNABKZEWPHRDN --network testnet --source acta_sc_source -- initialize --admin G...ADMIN --issuer_did did:pkh:stellar:testnet:G...ADMIN`

---

## issue

Issue a VC and store it in the holder’s Vault.

- Effect:
  - Calls `store_vc(owner, vc_id, vc_data, issuer, issuer_did, issuance_contract)` on the Vault.
  - Records local status `VCStatus::Valid` for `vc_id` in issuance.
  - Returns the `vc_id`.
- Requirements:
  - Must be signed by the issuance `admin`.
  - The holder’s Vault must have the `admin` authorized as `issuer`.
- Typical use: back-end issuance flows for one or multiple holders.

Example (testnet):
`soroban contract invoke --id CABQIR63YVKUBRSTEZFCXKJAR43PNF27WL2TAU67U5YMNABKZEWPHRDN --network testnet --source acta_sc_source -- issue --owner G...OWNER --vc_id "vc-123" --vc_data "<encrypted_payload>" --vault_contract CDK642PLEPCQH7WUBLHYYSKRJZOUIRIPY7GXQRHOETGR2JJ76UK6SWLZ`

---

## verify

Verify the status of a VC.

- Effect: returns a `Map { "status": "valid"|"invalid"|"revoked", "since": date? }`.
- Requirements: none (read-only).
- Typical use: quick status checks (e.g., during access verification).

Example (testnet):
`soroban contract invoke --id CABQIR63YVKUBRSTEZFCXKJAR43PNF27WL2TAU67U5YMNABKZEWPHRDN --network testnet -- verify --vc_id vc-123`

---

## revoke

Revoke a VC by `vc_id`, recording the revocation `date`.

- Effect: sets status to `VCStatus::Revoked(date)`.
- Requirements:
  - Must be signed by the `admin`.
  - Fails with `#2` if the VC does not exist/invalid.
  - Fails with `#3` if the VC is already revoked.
- Typical use: remove a VC for compliance, fraud, or expiration.

Example (testnet):
`soroban contract invoke --id CABQIR63YVKUBRSTEZFCXKJAR43PNF27WL2TAU67U5YMNABKZEWPHRDN --network testnet --source acta_sc_source -- revoke --vc_id vc-123 --date 2024-10-15T12:00:00Z`

---

## migrate

Migrate legacy storage (`VCs`, `Revocations`) to the current `VCStatus` scheme by key.

- Effect: re-stores legacy VCs and removes old keys.
- Requirements: must be signed by `admin`; fails with `#4` if no legacy VCs.
- Typical use: run once after an upgrade that changed the storage scheme.

Example (testnet):
`soroban contract invoke --id CABQIR63YVKUBRSTEZFCXKJAR43PNF27WL2TAU67U5YMNABKZEWPHRDN --network testnet --source acta_sc_source -- migrate`

---

## set_admin

Change the `admin` of the issuance contract.

- Effect: assigns `new_admin` as administrator.
- Requirements: must be signed by the current `admin`.
- Typical use: transfer control to another entity (custodian, multisig, etc.).

Example (testnet):
`soroban contract invoke --id CABQIR63YVKUBRSTEZFCXKJAR43PNF27WL2TAU67U5YMNABKZEWPHRDN --network testnet --source acta_sc_source -- set_admin --new_admin G...NEWADMIN`

---

## upgrade

Upgrade the contract WASM code.

- Effect: replaces the WASM with the `new_wasm_hash` (32 bytes).
- Requirements: must be signed by the `admin`.
- Typical use: deploy new versions with improvements or fixes.

Example (testnet):
`soroban contract invoke --id CABQIR63YVKUBRSTEZFCXKJAR43PNF27WL2TAU67U5YMNABKZEWPHRDN --network testnet --source acta_sc_source -- upgrade --new_wasm_hash a7a34e34b16d6ad3d4876f58737dfcbbaa8b5bb21abe17a3da022d181e4da3917`

---

## version

Returns the package version (`CARGO_PKG_VERSION`) compiled into the contract.

- Typical use: audit which version is running on-chain.

Example (testnet):
`soroban contract invoke --id CABQIR63YVKUBRSTEZFCXKJAR43PNF27WL2TAU67U5YMNABKZEWPHRDN --network testnet -- version`

---

## Data Visibility & Privacy

- Soroban state is public: authorization rules control execution/write access, not raw reads.
- Do not store plaintext PII: encrypt `vc_data` off-chain; consider storing only `sha256(vc_data)` and minimal metadata.
- `issuer_did` and `issuance_contract` (in Vault) are traceability metadata.

## Interaction with Vault

- Recommended flow:
  - Initialize the Vault for the `owner` and record its `did_uri`.
  - Authorize the issuance `admin` as `issuer` in that Vault.
  - Call `issue` on issuance: this stores the VC in the Vault and marks status `valid` in issuance.
- For revocation, use `revoke` on issuance; `verify` checks local status.

## Best Practices

- Align signer with role: `admin` signs administrative actions; `issuer` (issuance admin) is authorized per Vault.
- Manage IDs in configuration (e.g., `.env`):
  - `ISSUANCE_CONTRACT_ID=CABQIR63YVKUBRSTEZFCXKJAR43PNF27WL2TAU67U5YMNABKZEWPHRDN`
  - `VAULT_CONTRACT_ID=CDK642PLEPCQH7WUBLHYYSKRJZOUIRIPY7GXQRHOETGR2JJ76UK6SWLZ`
