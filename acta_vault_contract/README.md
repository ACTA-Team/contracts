# ACTA Vault Contract

Secure, multi‑tenant repository for storing Verifiable Credentials (VCs) and managing authorized issuers.

**Testnet Vault Contract ID**
Note: Example ID — replace with your own deployment.

- `CDK642PLEPCQH7WUBLHYYSKRJZOUIRIPY7GXQRHOETGR2JJ76UK6SWLZ`

## Multi‑Tenant Model

- Per‑owner isolation: every `owner` has its own vault with independent admin, issuer list, and revocation state.
- Vault admin: defaults to the `owner` on initialization; can be changed with `set_admin`.
- Contract admin: the first `owner` to call `initialize` becomes the global `ContractAdmin` (used only for `upgrade`).

## Public Functions

- `initialize(owner, did_uri)`: creates the vault for `owner` and records its DID URI.
- `authorize_issuers(owner, issuers)` / `authorize_issuer(owner, issuer)`: manage authorized issuers per vault.
- `store_vc(owner, vc_id, vc_data, issuer, issuer_did, issuance_contract)`: store VC payload for `owner` issued by an authorized `issuer`.
- `revoke_vault(owner)`: mark the `owner` vault as revoked.
- Per‑vault admin: `migrate(owner)`, `set_admin(owner, new_admin)`.
- Contract admin: `upgrade(new_wasm_hash)`, `version()`.

## Types

- `VerifiableCredential { id: String, data: String, issuance_contract: Address, issuer_did: String }`

## Authorization

- Issuer must be authorized in the `owner` vault and must sign `store_vc`.
- Vault admin must sign per‑vault admin operations.
- `upgrade` requires `ContractAdmin` signature.

## Data Visibility & Privacy

- Contract state is public on chain. Do not store PII in plaintext.
- Recommended: anchor off‑chain data via hash, or store ciphertext only.

## Issuance Contract

- `issuance_contract` is the on‑chain contract ID (`C…`) of the issuer that originated the VC.
- It is stored as metadata for traceability; the vault does not call/validate it during `store_vc` today.
- Deploy once per issuer/organization, then reference its contract ID in every VC.

## CLI Examples (Testnet)

- Initialize:
  `soroban contract invoke --id CDK642PLEPCQH7WUBLHYYSKRJZOUIRIPY7GXQRHOETGR2JJ76UK6SWLZ --network testnet -- initialize --owner G...OWNER --did_uri did:pkh:stellar:testnet:G...OWNER`
- Authorize issuer:
  `soroban contract invoke --id CDK642PLEPCQH7WUBLHYYSKRJZOUIRIPY7GXQRHOETGR2JJ76UK6SWLZ --network testnet -- authorize_issuer --owner G...OWNER --issuer G...ISSUER`
- Store VC:
  `soroban contract invoke --id CDK642PLEPCQH7WUBLHYYSKRJZOUIRIPY7GXQRHOETGR2JJ76UK6SWLZ --network testnet -- store_vc --owner G...OWNER --vc_id vc-123 --vc_data '{"name":"Alice"}' --issuer G...ISSUER --issuer_did did:pkh:stellar:testnet:G...ISSUER --issuance_contract C...ISSUANCE_ID`

## License

Apache 2.0.
