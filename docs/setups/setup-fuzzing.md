# vc-vault — Installation, Build & Deployment

## Prerequisites

| Tool | Version | Install |
|---|---|---|
| Rust | stable + nightly | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Stellar CLI | ≥ 21.0.0 | `cargo install --locked stellar-cli` |
| cargo-fuzz | latest | `cargo install cargo-fuzz` |
| wasm32v1-none target | — | `rustup target add wasm32v1-none` |
| nightly toolchain | — | `rustup toolchain install nightly` |

Verify:

```sh
rustc --version
stellar --version
cargo fuzz --version
```

---

## Repository layout

```
contracts/
  vc-vault/
    src/           Contract source
    fuzz/          Fuzz targets (cargo-fuzz workspace)
    Cargo.toml
docs/
scripts/
  build.sh         Build + optimize WASM
  release.sh       Deploy to testnet
Cargo.toml         Workspace root
```

---

## Building

### Debug build (fast, for development)

```sh
cargo build -p vc-vault-contract
```

### Release WASM (for deployment)

```sh
sh scripts/build.sh
```

Outputs:
- `target/wasm32v1-none/release/vc_vault_contract.wasm` — unoptimized
- `target/wasm32v1-none/release/vc_vault_contract.optimized.wasm` — optimized (deploy this one)

The build script uses `cargo rustc -- --crate-type cdylib` to force cdylib output for the WASM build without declaring it in `Cargo.toml`. This is required because declaring `cdylib` in `Cargo.toml` would cause cargo to build a native `.dylib` during fuzzing, which fails to link sancov symbols on macOS. The script runs `set -eu` so it will fail fast on any error. Never deploy a stale artifact.

---

## Running tests

```sh
cargo test -p vc-vault-contract
```

Expected output: **54 tests, 0 failures, 0 warnings**.

The test suite includes:
- 49 functional tests covering the full VC lifecycle, issuer management, sponsored vaults, fee config, migration, and push-related regression cases.
- 5 targeted authorization tests (`setup_no_mock`) that verify `require_auth()` guards are enforced and would catch regressions if a guard is accidentally removed.

---

## Running the fuzz suite

Fuzz targets require nightly Rust and live under `contracts/vc-vault/fuzz/`.

```sh
cd contracts/vc-vault

# Run the lifecycle fuzzer (recommended starting point)
cargo +nightly fuzz run fuzz_lifecycle --sanitizer none

# Run a specific focused fuzzer
cargo +nightly fuzz run fuzz_issue --sanitizer none
cargo +nightly fuzz run fuzz_revoke --sanitizer none
cargo +nightly fuzz run fuzz_verify_vc --sanitizer none
cargo +nightly fuzz run fuzz_push --sanitizer none
cargo +nightly fuzz run fuzz_issuer_ops --sanitizer none
```

> **Why `--sanitizer none`?** Soroban contracts use `#![no_std]`. On macOS aarch64, AddressSanitizer (ASAN) fails because `no_std` lacks the expected sanitizer init infrastructure. Since contracts run in a WASM sandbox with no raw memory access, memory safety bugs are not the fuzzing target — logic bugs are. Coverage-guided fuzzing without ASAN is fully effective for invariant checking.

To stop a fuzzer: `Ctrl+C`. Crash inputs are saved to `fuzz/artifacts/<target>/`.

To replay a crash:

```sh
cargo +nightly fuzz run fuzz_lifecycle --sanitizer none fuzz/artifacts/fuzz_lifecycle/<crash-file>
```

---

## Deploying to testnet

```sh
sh scripts/release.sh
```

The script:
1. Adds the `testnet` network config if not already present (idempotent).
2. Generates the `vc_vault_admin` keypair if not already present (idempotent).
3. Runs `scripts/build.sh` to produce a fresh optimized WASM.
4. Deploys with `stellar contract deploy` and prints the contract ID.

The script uses `set -eu` — any failure stops execution immediately.

---

## Common errors

| Error | Cause | Fix |
|---|---|---|
| `error: the option Z is only accepted on the nightly compiler` | Running `cargo fuzz` without nightly | Add `+nightly` or `rustup override set nightly` in `contracts/vc-vault/` |
| `Undefined symbols: ___sanitizer_cov_*` | ASAN + `no_std` on macOS aarch64 | Add `--sanitizer none` |
| `error: no such command: fuzz` | cargo-fuzz not installed | `cargo install cargo-fuzz` |
| `wasm32v1-none target not found` | Missing WASM target | `rustup target add wasm32v1-none` |
