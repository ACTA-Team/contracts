# Security Scanning

ACTA runs Stellar ecosystem security tools as part of the SCF audit readiness checklist.

## Tools

### Scout (Soroban Static Analyzer)

[Scout](https://github.com/stellar/rust-scout-audit) is a static analysis tool designed specifically for Soroban smart contracts. It detects common vulnerability patterns including:

- Missing authorization checks
- Integer overflow/underflow
- Re-entrancy risks
- Unchecked cross-contract calls
- Storage collision patterns

**Install:**
```bash
cargo install scout-audit
```

**Run:**
```bash
# Scan all contracts
bash scripts/security-scan.sh --scout

# Scan a specific contract
cargo scout-audit --manifest-path contracts/vc-vault/Cargo.toml
```

**Reports:** Written to `docs/audits/scans/scan-{timestamp}.md`

### Almanax (On-Chain Security Monitor)

[Almanax](https://almanax.org) provides continuous on-chain security monitoring for deployed Soroban contracts.

**Prerequisites:**
- `ALMANAX_API_KEY` environment variable (set in CI secrets)

**Run:**
```bash
ALMANAX_API_KEY=your-key bash scripts/security-scan.sh --almanax
```

## CI Pipeline

The security scan workflow (`.github/workflows/security-scan.yml`) runs:

| Trigger | Scout | Almanax |
|---|---|---|
| Push to contracts/ | ✅ | — |
| PR to contracts/ | ✅ | — |
| Weekly schedule | ✅ | ✅ |
| Manual dispatch | ✅ | ✅ |

## Adding New Tools

To add a new Stellar security tool to this pipeline:

1. Add the tool's installation to the CI workflow
2. Add a `run_<tool>()` function to `scripts/security-scan.sh`
3. Update this document

### Stellar Ecosystem Tool Candidates

| Tool | Type | Status |
|---|---|---|
| [Scout](https://github.com/stellar/rust-scout-audit) | Static analysis | ✅ Integrated |
| [Almanax](https://almanax.org) | On-chain monitor | ✅ Integrated |
| Stellar Expert | Explorer verification | TBD |
| Soroban Fuzzer | Fuzz testing | TBD |

## Previous Audits

- [Audit Report v1 — vc-vault](audit-acta-v1.md) (February 2026)
- Scout scans are archived in [`scans/`](scans/)
