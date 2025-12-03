# Smart Contracts

Issuance, storage, and verification of Verifiable Credentials (VC) on Soroban.

This monorepo contains the ACTA contracts located in `contracts/`:

- **`issuance`** (`contracts/issuance/`): Issues, verifies, and revokes Verifiable Credentials (VCs). Manages VC status (valid, revoked, invalid) and stores encrypted payloads in the holder's Vault contract.

- **`vault`** (`contracts/vault/`): Secure, multi-tenant repository for storing Verifiable Credentials (VCs) and managing authorized issuers per owner. Each owner has an isolated vault with independent admin, issuer authorization list, and revocation state.

## Build

First, build the contracts in release mode:

```bash
cargo build --release
```

Then, optimize the WASM files:

```bash
chmod +x scripts/build.sh
./scripts/build.sh
```

Or run both steps manually:

```bash
# Build contracts
cargo build --release

# Optimize WASM files
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/vault_contract.wasm
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/issuance_contract.wasm
```

The optimized WASM files will be generated at:

- `target/wasm32-unknown-unknown/release/vault_contract.optimized.wasm`
- `target/wasm32-unknown-unknown/release/issuance_contract.optimized.wasm`

## Release (Testnet)

**Important:** Make sure you have built and optimized the contracts first (see Build section above).

Then, deploy to testnet:

```bash
chmod +x scripts/release.sh
./scripts/release.sh
```

This script will:

1. Configure the testnet network (if not already configured)
2. Generate a key for signing transactions (`acta_admin`)
3. Install and deploy both contracts to testnet

**Note:** The script will output the contract IDs after deployment. Save these IDs for future reference.

## License

This software is licensed under the [Apache License 2.0](./LICENSE).
