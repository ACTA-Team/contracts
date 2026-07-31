#!/usr/bin/env bash
#
# Security scanning script for ACTA Soroban contracts.
# Runs the Stellar ecosystem security tools listed in the SCF audit readiness checklist.
#
# Usage:
#   bash scripts/security-scan.sh            # Run all scanners
#   bash scripts/security-scan.sh --scout     # Scout only (Soroban static analysis)
#   bash scripts/security-scan.sh --almanax   # Almanax only
#
# Prerequisites:
#   - Rust toolchain (rustup, cargo)
#   - soroban-cli (for contract compilation)
#   - scout-audit (cargo install scout-audit)
#   - Almanax API key in ALMANAX_API_KEY env var (optional)

set -euo pipefail

REPORT_DIR="${REPORT_DIR:-docs/audits/scans}"
TIMESTAMP="$(date -u +%Y-%m-%dT%H%M%SZ)"
REPORT_FILE="${REPORT_DIR}/scan-${TIMESTAMP}.md"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info()  { echo -e "${GREEN}[INFO]${NC}  $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

# ---------------------------------------------------------------------------
# Tool availability checks
# ---------------------------------------------------------------------------

check_cargo() {
  if ! command -v cargo &>/dev/null; then
    log_error "cargo is not installed. Install Rust: https://rustup.rs"
    return 1
  fi
}

check_scout() {
  if ! cargo install --list 2>/dev/null | grep -q 'scout-audit'; then
    log_warn "scout-audit is not installed."
    log_info "Install with: cargo install scout-audit"
    log_info "Repository: https://github.com/stellar/rust-scout-audit"
    return 1
  fi
  return 0
}

check_soroban() {
  if ! command -v soroban &>/dev/null; then
    log_warn "soroban CLI is not installed."
    log_info "Install with: cargo install soroban-cli"
    return 1
  fi
  return 0
}

# ---------------------------------------------------------------------------
# Scout — Soroban-specific static analyzer
# ---------------------------------------------------------------------------

run_scout() {
  log_info "=== Scout: Soroban Static Analyzer ==="

  mkdir -p "${REPORT_DIR}"

  {
    echo "# Scout Audit — ${TIMESTAMP}"
    echo ""
    echo "| Contract | Findings | High | Medium | Low |"
    echo "|---|---|---|---|---|"
  } > "${REPORT_FILE}"

  local total_findings=0
  local total_high=0

  for contract_dir in contracts/*/; do
    local contract_name
    contract_name="$(basename "$contract_dir")"

    if [ ! -f "${contract_dir}/Cargo.toml" ]; then
      continue
    fi

    log_info "Scanning ${contract_name}..."

    local findings=0
    local high=0
    local medium=0
    local low=0

    # Run scout-audit on the contract
    local scout_output
    if check_scout; then
      scout_output="$(cargo scout-audit --manifest-path "${contract_dir}/Cargo.toml" 2>&1)" || true
    else
      scout_output="[scout-audit not installed — skipped]"
    fi

    # Parse scout output for severity counts (scout uses standard exit codes + JSON output)
    if echo "$scout_output" | grep -q '"severity":"High"'; then
      high=$(echo "$scout_output" | grep -c '"severity":"High"' || echo 0)
    fi
    if echo "$scout_output" | grep -q '"severity":"Medium"'; then
      medium=$(echo "$scout_output" | grep -c '"severity":"Medium"' || echo 0)
    fi
    if echo "$scout_output" | grep -q '"severity":"Low"'; then
      low=$(echo "$scout_output" | grep -c '"severity":"Low"' || echo 0)
    fi
    findings=$((high + medium + low))

    echo "| ${contract_name} | ${findings} | ${high} | ${medium} | ${low} |" >> "${REPORT_FILE}"

    # Append detailed output
    {
      echo ""
      echo "## ${contract_name}"
      echo ""
      echo '```'
      echo "${scout_output}"
      echo '```'
    } >> "${REPORT_FILE}"

    total_findings=$((total_findings + findings))
    total_high=$((total_high + high))
  done

  {
    echo ""
    echo "---"
    echo ""
    echo "**Total findings:** ${total_findings} (${total_high} High)"
    echo ""
    echo "Report generated: ${TIMESTAMP}"
  } >> "${REPORT_FILE}"

  if [ "${total_high}" -gt 0 ]; then
    log_warn "Scout found ${total_high} high-severity issue(s). Review ${REPORT_FILE}"
  else
    log_info "Scout: no high-severity issues found. Report: ${REPORT_FILE}"
  fi
}

# ---------------------------------------------------------------------------
# Almanax — on-chain security monitoring
# ---------------------------------------------------------------------------

run_almanax() {
  log_info "=== Almanax: On-Chain Security Monitor ==="

  if [ -z "${ALMANAX_API_KEY:-}" ]; then
    log_warn "ALMANAX_API_KEY not set — skipping Almanax scan."
    log_info "Set ALMANAX_API_KEY in your environment or CI secrets."
    return 0
  fi

  mkdir -p "${REPORT_DIR}"
  local almanax_report="${REPORT_DIR}/almanax-${TIMESTAMP}.json"

  log_info "Running Almanax scan on deployed contracts..."

  # Almanax scans deployed contract addresses
  # Contract addresses are configured in deploy scripts
  # This is a placeholder — replace with actual Almanax CLI/API calls
  log_info "Almanax report placeholder: ${almanax_report}"
  log_warn "Almanax scan requires deployed contract addresses. Update scripts/deploy-mainnet.sh output."
}

# ---------------------------------------------------------------------------
# Full scan
# ---------------------------------------------------------------------------

run_all() {
  log_info "=== ACTA Security Scan — ${TIMESTAMP} ==="
  echo ""

  # 1. Scout (static analysis)
  run_scout || true

  echo ""

  # 2. Almanax (on-chain monitoring)
  run_almanax || true

  echo ""
  log_info "=== Scan complete ==="
  log_info "Reports written to ${REPORT_DIR}/"
}

# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

case "${1:-all}" in
  --scout)   check_cargo && check_soroban && run_scout ;;
  --almanax) run_almanax ;;
  all|*)     check_cargo && check_soroban && run_all ;;
esac
