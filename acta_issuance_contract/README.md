# ACTA VC Issuance Contract

Issues, verifies, and revokes Verifiable Credentials (VCs) on Soroban.

## Features
- VC issuance: stores encrypted payload in the holder’s Vault.
- Status verification: `valid`, `revoked` (with date), `invalid`.
- VC revocation by `vc_id`.
- Administration: `initialize`, `set_admin`, `upgrade`, `version`.

## Security & Privacy
- `admin.require_auth()` for sensitive actions.
- `vc_data` must be encrypted off-chain (no plaintext PII on-chain).
- On-chain only stores status and access control.

## Functions
- `initialize(admin: Address, issuer_did: String)`
- `issue(vc_id: String, vc_data: String, vault_contract: Address) -> String`
- `verify(vc_id: String) -> Map<String, String>`
- `revoke(vc_id: String, date: String)`
- `set_admin(new_admin: Address)`
- `upgrade(wasm_hash: BytesN<32>)`
- `version() -> String`

## Example (CLI)
```bash
soroban contract invoke \
  --id ISSUANCE_CONTRACT_ID \
  --source ADMIN_SECRET \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase 'Test SDF Network ; September 2015' \
  -- \
  issue \
  --vc_id "sample-vc-id" \
  --vc_data "<encrypted_payload>" \
  --vault_contract VAULT_CONTRACT_ID
```

## License
Apache 2.0.