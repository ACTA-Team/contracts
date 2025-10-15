# ACTA VC Issuance Contract

Issues, verifies, and revokes Verifiable Credentials (VCs) on Soroban.

## Testnet Contract ID

Note: Example ID — replace with your own deployment.

- `CBQ5HQI4CG6VI74D46ZJ6YVTXZJ6UQZM7GLC2WUDS75ACXWEBI2AE2OG`

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
- `issue(owner: Address, vc_id: String, vc_data: String, vault_contract: Address) -> String`
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
  --owner G...OWNER \
  --vc_id "sample-vc-id" \
  --vc_data "<encrypted_payload>" \
  --vault_contract VAULT_CONTRACT_ID
```

## Deploy & Initialize (Testnet)

```bash
# From ACTA-Contracts/acta_issuance_contract
soroban contract build

# Deploy (uses default identity: acta_sc_source)
soroban contract deploy \
  --wasm "C:\\Projects\\ACTA\\API\\ACTA-Contracts\\target\\wasm32v1-none\\release\\acta_issuance_contract.wasm" \
  --network testnet \
  --source acta_sc_source
# => Contract ID: CBQ5HQI4CG6VI74D46ZJ6YVTXZJ6UQZM7GLC2WUDS75ACXWEBI2AE2OG

# Initialize with admin and issuer DID
soroban contract invoke \
  --id CBQ5HQI4CG6VI74D46ZJ6YVTXZJ6UQZM7GLC2WUDS75ACXWEBI2AE2OG \
  --network testnet \
  --source acta_sc_source \
  -- \
  initialize \
  --admin GDIWRJDHMK3JTMXSMCGFEM2QMCHSQ2BTMY2DFH3MS7VZGHXLI46OYE25 \
  --issuer_did "did:pkh:stellar:testnet:GDIWRJDHMK3JTMXSMCGFEM2QMCHSQ2BTMY2DFH3MS7VZGHXLI46OYE25"
```

## Issue a VC (Testnet)

```bash
soroban contract invoke \
  --id CBQ5HQI4CG6VI74D46ZJ6YVTXZJ6UQZM7GLC2WUDS75ACXWEBI2AE2OG \
  --network testnet \
  --source acta_sc_source \
  -- \
  issue \
  --owner G...OWNER \
  --vc_id "vc-123" \
  --vc_data "<encrypted_payload>" \
  --vault_contract CD7AN2XKCQLFNENL6YUUNZ6FBAL63N5J5X7AEGLRYSG6YBS6V35OSJCH
```

## License

Apache 2.0.
