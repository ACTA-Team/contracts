# Security Scanning

ACTA runs Stellar ecosystem security tools as part of the SCF audit readiness checklist.

## Tools

### Scout (Soroban Static Analyzer)

[Scout](https://github.com/stellar/rust-scout-audit) detects common vulnerability patterns in Soroban contracts:

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
bash scripts/security-scan.sh --scout
```

Reports are written to `docs/audits/scans/scout-{timestamp}.md`.

### Almanax (On-Chain Monitor)

[Almanax](https://almanax.org) provides continuous on-chain security monitoring for deployed contracts.

Requires `ALMANAX_API_KEY` environment variable (set in CI secrets).

```bash
ALMANAX_API_KEY=your-key bash scripts/security-scan.sh --almanax
```

## CI Pipeline

| Trigger | Scout | Almanax |
|---|---|---|
| Push to contracts/ | ✅ | — |
| PR to contracts/ | ✅ | — |
| Weekly schedule | ✅ | ✅ |
| Manual dispatch | ✅ | ✅ |

## Adding New Tools

1. Add installation to `.github/workflows/security-scan.yml`
2. Add a `run_<tool>()` function to `scripts/security-scan.sh`
3. Update this document

## Previous Audits

- [Audit Report v1 — vc-vault](audit-acta-v1.md) (February 2026)
