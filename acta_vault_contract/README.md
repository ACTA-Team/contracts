# ACTA Vault Contract

Secure repository for storing encrypted Verifiable Credential (VC) payloads and managing authorized issuers.

## Features
- `initialize(admin, did_uri)`: registers the holder’s DID URI directly (no DID contract).
- `authorize_issuers(issuers)` / `authorize_issuer(issuer)`: manage authorized issuers.
- `store_vc(vc_id, vc_data, issuer, issuer_did, issuance_contract)`: store encrypted VC payload.
- `revoke_vault()`: mark the vault as revoked.
- Administration: `migrate`, `set_admin`, `upgrade`, `version`.

## Types
- `VerifiableCredential { id: String, data: String, issuance_contract: Address, issuer_did: String }`

## Security
- `issuer.require_auth()` in `store_vc` and validation against the authorized issuers list.
- `vc_data` must be encrypted off-chain (no plaintext PII on-chain).

## Example (CLI)
```bash
soroban contract invoke \
  --id VAULT_CONTRACT_ID \
  --source ADMIN_SECRET \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase 'Test SDF Network ; September 2015' \
  -- \
  authorize_issuer \
  --issuer GC...ISSUER_ADDRESS
```

## License
Apache 2.0.