# Smart Contracts (Soroban)

ACTA Verifiable Credentials on Soroban: **vault storage + issuance status registry**, unified into **one** contract.

## Contract

This repo contains a single Soroban contract located at:

- **`contracts/contracts/acta/`**: unified contract that includes:
  - **Vault (per owner)**: `create_vault`, issuer authorization, `list_vc_ids`, `get_vc`, `push`, `revoke_vault`, `set_vault_admin`
  - **Issuance (status registry)**: `issue`, `verify_vc`, `revoke`
  - **Admin**: `initialize`, `set_contract_admin`, fee config (`set_fee_config`, `set_fee_enabled`), `upgrade`, `version`

## Security & Privacy

- Contract state is public on-chain: **store only ciphertext** in `vc_data` (never plaintext PII).
- Admin-gated functions require signatures (`require_auth()`).
- `initialize` requires `contract_admin` signature; `create_vault` requires `owner` signature (prevents hostile/grief initialization).
- Vault write operations are blocked if the vault is revoked.

## Build

Use the build script (Soroban v21 uses `wasm32v1-none` output by default):

**Linux/macOS:**

```bash
chmod +x scripts/build.sh
./scripts/build.sh
```

**Windows (PowerShell):**

```bash
bash scripts/build.sh
```

**Manual:**

```bash
soroban contract build
soroban contract optimize --wasm target/wasm32v1-none/release/acta_contract.wasm
```

Build outputs:

- `target/wasm32v1-none/release/acta_contract.wasm`
- `target/wasm32v1-none/release/acta_contract.optimized.wasm`

## Deploy (Testnet)

Use the release script:

**Linux/macOS:**

```bash
chmod +x scripts/release.sh
./scripts/release.sh
```

**Windows (PowerShell):**

```bash
bash scripts/release.sh
```

The script configures testnet (idempotent), generates `acta_admin` (idempotent), builds/optimizes, and deploys the unified contract.

## License

This software is licensed under the [Apache License 2.0](./LICENSE).
