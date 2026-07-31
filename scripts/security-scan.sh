#!/usr/bin/env bash
#
# Security scanning script for ACTA Soroban contracts.
# Runs Stellar ecosystem security tools for SCF audit readiness.
#
# Usage:
#   bash scripts/security-scan.sh            # Run all scanners
#   bash scripts/security-scan.sh --scout     # Scout only (Soroban static analysis)
#   bash scripts/security-scan.sh --almanax   # Almanax only
#
# Prerequisites:
#   - Rust toolchain (rustup, cargo)
#   - soroban-cli (cargo install soroban-cli)
#   - scout-audit (cargo install scout-audit)
#   - ALMANAX_API_KEY env var for Almanax (optional)

set -euo pipefail

REPORT_DIR="${REPORT_DIR:-docs/audits/scans}"
TIMESTAMP="$(date -u +%Y-%m-%dT%H%M%SZ)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info()  { echo -e "${GREEN}[INFO]${NC}  $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

# ---------------------------------------------------------------------------
# Scout — Soroban-specific static analyzer
# ---------------------------------------------------------------------------

run_scout() {
  log_info "=== Scout: Soroban Static Analyzer ==="

  if ! command -v cargo &>/dev/null; then
    log_error "cargo is not installed. Install Rust: https://rustup.rs"
    return 1
  fi

  if ! cargo install --list 2>/dev/null | grep -q 'scout-audit'; then
    log_warn "scout-audit is not installed."
    log_info "Install: cargo install scout-audit"
    log_info "Repo: https://github.com/stellar/rust-scout-audit"
    return 1
  fi

  if ! command -v soroban &>/dev/null; then
    log_warn "soroban CLI is not installed. Install: cargo install soroban-cli"
    return 1
  fi

  mkdir -p "${REPORT_DIR}"
  local report="${REPORT_DIR}/scout-${TIMESTAMP}.md"

  {
    echo "# Scout Audit — ${TIMESTAMP}"
    echo ""
    echo "| Contract | Findings | High | Medium | Low |"
    echo "|---|---|---|---|---|"
  } > "${report}"

  local total_findings=0 total_high=0

  for contract_dir in contracts/*/; do
    local name
    name="$(basename "$contract_dir")"
    [ -f "${contract_dir}/Cargo.toml" ] || continue

    log_info "Scanning ${name}..."

    local output
    output="$(cargo scout-audit --manifest-path "${contract_dir}/Cargo.toml" 2>&1)" || true

    local high=0 medium=0 low=0
    high=$(echo "$output" | grep -c '"severity":"High"' || echo 0)
    medium=$(echo "$output" | grep -c '"severity":"Medium"' || echo 0)
    low=$(echo "$output" | grep -c '"severity":"Low"' || echo 0)
    local findings=$((high + medium + low))

    echo "| ${name} | ${findings} | ${high} | ${medium} | ${low} |" >> "${report}"

    {
      echo ""
      echo "## ${name}"
      echo ""
      echo '```'
      echo "${output}"
      echo '```'
    } >> "${report}"

    total_findings=$((total_findings + findings))
    total_high=$((total_high + high))
  done

  {
    echo ""
    echo "---"
    echo ""
    echo "**Total:** ${total_findings} findings (${total_high} High)"
    echo "Generated: ${TIMESTAMP}"
  } >> "${report}"

  if [ "${total_high}" -gt 0 ]; then
    log_warn "Scout: ${total_high} high-severity issue(s). See ${report}"
  else
    log_info "Scout: no high-severity issues. Report: ${report}"
  fi
}

# ---------------------------------------------------------------------------
# Almanax — on-chain security monitoring
# ---------------------------------------------------------------------------

run_almanax() {
  log_info "=== Almanax: On-Chain Security Monitor ==="

  if [ -z "${ALMANAX_API_KEY:-}" ]; then
    log_warn "ALMANAX_API_KEY not set — skipping Almanax."
    log_info "Set ALMANAX_API_KEY in env or CI secrets."
    return 0
  fi

  mkdir -p "${REPORT_DIR}"
  local report="${REPORT_DIR}/almanax-${TIMESTAMP}.json"
  log_info "Almanax report: ${report}"
  log_warn "Almanax scan requires deployed contract addresses — update deploy scripts."
}

# ---------------------------------------------------------------------------
# Full scan
# ---------------------------------------------------------------------------

run_all() {
  log_info "=== ACTA Security Scan — ${TIMESTAMP} ==="
  echo ""
  run_scout || true
  echo ""
  run_almanax || true
  echo ""
  log_info "=== Done. Reports: ${REPORT_DIR}/ ==="
}

case "${1:-all}" in
  --scout)   run_scout ;;
  --almanax) run_almanax ;;
  all|*)     run_all ;;
esac
