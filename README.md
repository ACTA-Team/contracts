# contracts-acta

Soroban smart contracts for the ACTA identity and Verifiable Credential infrastructure on Stellar.

**Latest release:** [vc-vault v0.4.0](https://github.com/ACTA-Team/contracts-acta/releases/tag/vc-vault-v0.4.0)

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

### `vc-vault`

Per-holder vault for Verifiable Credentials on Stellar. Manages VC storage, issuance status, revocation, issuer authorization, and fee collection in USDC. Contract admin is set at deploy time via `__constructor`.

| Category | Functions |
|---|---|
| Admin | `nominate_admin`, `accept_contract_admin`, `upgrade`, `version`, `fee_*` |
| Vault | `create_vault`, `create_sponsored_vault`, `set_vault_admin`, `authorize_issuer`, `authorize_issuers`, `revoke_issuer`, `revoke_vault`, `list_authorized_issuers`, `list_denied_issuers`, `authorized_issuer_count`, `denied_issuer_count` |
| Credentials | `issue`, `batch_issue`, `issue_linked`, `revoke`, `verify_vc`, `get_vc`, `list_vc_ids`, `vc_count`, `push` |

See [`contracts/vc-vault/README.md`](contracts/vc-vault/README.md) for the full ABI, authorization model, and error codes.

---

## Testnet Deployments

| Contract | Contract ID |
|---|---|
| `did-stellar-registry` | `CB7ATU7SF5QUKJMSULJDJVWJZVDXC23HTZX6NFUDTSFPVT6MA575NNZJ` |
| `vc-vault` | `CATL4IDH7XXPDC2UHSEX2GP45PPBVDFSKUDTKCSQICDOJVDLYNKISXFH` |

Network: Stellar Testnet (`Test SDF Network ; September 2015`)  
Full deployment record: [`docs/deployments/testnet.md`](docs/deployments/testnet.md)

---

## Build

```bash
# Build a specific contract
./scripts/build.sh vc-vault
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
./scripts/deploy.sh did-stellar-registry testnet acta_deployer
./scripts/deploy.sh vc-vault testnet acta_deployer
```

Record the resulting contract ID in [`docs/deployments/<network>.md`](docs/deployments/).

---

## Specification

The `did:stellar` v0.1 method specification lives at [`docs/did-spec/did-stellar-v0.1.md`](docs/did-spec/did-stellar-v0.1.md). It covers:

- DID syntax and identifier generation
- On-chain data model (`DidRecord`)
- Contract operations and mutation semantics
- DID Document construction rules
- Proof of control protocol
- Normative test vectors

---

## Tests

```bash
cargo test -p vc-vault-contract          # 127 tests
cargo test -p did-stellar-registry       # 56 tests
```

---

## License

Licensed under the [Apache License 2.0](./LICENSE).
