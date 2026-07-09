# contracts-acta

Soroban smart contracts for the ACTA identity and Verifiable Credential infrastructure on Stellar.

**Live on mainnet:** [mainnet-v1.0.0](https://github.com/ACTA-Team/contracts-acta/releases/tag/mainnet-v1.0.0) · **Latest contract release:** [vc-vault v0.4.0](https://github.com/ACTA-Team/contracts-acta/releases/tag/vc-vault-v0.4.0)

---

## Contracts

### `did-stellar-registry`

On-chain registry for the [`did:stellar`](docs/did-spec/did-stellar-v0.1.md) decentralized identifier method (v0.1). Stores the authoritative `DidRecord` for each DID on Stellar, readable directly from RPC without an indexer.

| Function | Description |
|---|---|
| `register` | Create a new DID |
| `update` | Replace the full DID record (optimistic concurrency) |
| `transfer_controller` | Transfer control to a different Stellar account |
| `deactivate` | Permanently deactivate a DID |
| `get` | Read-only: return the current `DidRecord` |

See [`contracts/did-stellar-registry/README.md`](contracts/did-stellar-registry/README.md) for the full ABI, authorization model, and error codes.

---

### `vc-vault-factory`

Deploys and tracks **single-tenant `vc-vault` instances**, one vault contract per holder, rather than a shared multi-tenant contract. Derives deterministic vault addresses from `(owner, salt)`, centralizes fee configuration, and maintains the `is_vault` registry that vaults use to validate cross-vault transfers.

| Category | Functions |
|---|---|
| Deploy | `deploy`, `deploy_sponsored`, `is_vault` |
| Fees | `set_fee_config`, `set_fee_enabled`, `set_fee_standard`, `set_fee_custom`, `remove_fee_custom`, `set_min_fee`, `quote_fee` |
| Admin | `nominate_admin`, `accept_admin`, `get_admin` |

See [`contracts/vc-vault-factory/README.md`](contracts/vc-vault-factory/README.md) for the full ABI.

---

### `vc-vault`

**Single-tenant** Verifiable Credential vault, each holder owns their own instance, deployed by the factory. Manages VC storage, issuance status, revocation, and cross-vault migration. Issuance is **open** (anyone may issue unless denylisted); fees are quoted by the factory at issuance time.

| Category | Functions |
|---|---|
| Admin | `nominate_admin`, `accept_contract_admin`, `version` |
| Vault | `set_vault_admin`, `set_vault_did`, `vault_did`, `vault_owner`, `deny_issuer`, `allow_issuer`, `revoke_vault`, `list_denied_issuers`, `denied_issuer_count` |
| Credentials | `issue`, `batch_issue`, `revoke`, `push`, `receive_push`, `verify_vc`, `get_vc`, `list_vc_ids`, `vc_count` |

---

## Mainnet Deployments

| Contract | Version | Contract ID / WASM hash |
|---|---|---|
| `did-stellar-registry` | 0.2.0 | `CD6LSWW5ZSXOO5WAIHKQLQ262TW7BPI37PNEVMMA273BAPC65NN2AYXQ` |
| `vc-vault-factory` | 0.1.0 | `CCWNZ6UMUXCDOVP2TWOPVLI4KP4VY4YF7VKPN6XLYVHNFAT24NDB33CX` |
| `vc-vault` | 0.4.0 | template WASM hash `2bd0323a98acb8469606808368da6c79824f2dd8391494b94ddbeb3d22c1a957` (instances deployed via the factory) |

Network: Stellar Mainnet (`Public Global Stellar Network ; September 2015`)  
Factory fee: **1 USDC per credential** issued (paid by the issuer).  
Full deployment record: [`docs/deployments/mainnet.md`](docs/deployments/mainnet.md)

---

## Testnet Deployments

| Contract | Version | Contract ID / WASM hash |
|---|---|---|
| `did-stellar-registry` | 0.2.0 | `CBUNQ3GX3ZQ4MF64H7JCYZMXLGOS47VPIQQS7NCR6V3KX6YP7O72L5QF` |
| `vc-vault-factory` | 0.1.0 | `CDRFQRIP4FA3WMPWCSAM3XEY6EM6EGKRYZRSCSVZ5NHCF6AGEVR2XEPQ` |
| `vc-vault` | 0.4.0 | template WASM hash `2bd0323a98acb8469606808368da6c79824f2dd8391494b94ddbeb3d22c1a957` (instances deployed via the factory) |

Network: Stellar Testnet (`Test SDF Network ; September 2015`)  
Full deployment record: [`docs/deployments/testnet.md`](docs/deployments/testnet.md)

---

## Build

```bash
# Build a specific contract (or `all`)
./scripts/build.sh vc-vault
./scripts/build.sh vc-vault-factory
./scripts/build.sh did-stellar-registry
```

Output files:
- `target/wasm32v1-none/release/<contract>.wasm`
- `target/wasm32v1-none/release/<contract>.optimized.wasm`

---

## Deploy

```bash
# Prerequisites: stellar-cli installed, network configured, key generated
./scripts/deploy.sh <package> <network> <source-account>

# Examples
./scripts/deploy.sh did-stellar-registry testnet acta_deployer   # deploy
./scripts/deploy.sh vc-vault             testnet acta_deployer   # install template, prints WASM hash
./scripts/deploy.sh vc-vault-factory     testnet acta_deployer   # installs vault + deploys factory
```

`vc-vault` is not deployed standalone — the factory instantiates per-holder vaults via `factory.deploy(...)`. Record the resulting IDs in [`docs/deployments/<network>.md`](docs/deployments/).

---

## Specification

The `did:stellar` v0.1 method specification lives at [`docs/did-spec/did-stellar-v0.1.md`](docs/did-spec/did-stellar-v0.1.md). It covers:

- Goals and design decisions
- DID syntax and identifier generation
- On-chain registry data model (`DidRecord`) and contract operations
- DID Document construction rules
- Proof of control protocol
- Security, privacy, and cost considerations
- Acceptance criteria and normative test vectors
- Integrator notes and a worked end-to-end example

---

## Tests

```bash
cargo test                               # whole workspace (146 tests)
cargo test -p vc-vault-contract          # 61 tests
cargo test -p vc-vault-factory-contract  # 27 tests
cargo test -p did-stellar-registry       # 58 tests
```

---

## License

Licensed under the [Apache License 2.0](./LICENSE).
