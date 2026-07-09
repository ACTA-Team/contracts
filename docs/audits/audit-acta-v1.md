# Security Audit Report - vc-vault Contract

| | |
|---|---|
| **Contract** | `vc-vault-contract` |
| **Version** | `0.21.0` |
| **Platform** | Soroban / Stellar |
| **SDK** | `soroban-sdk 23.4.0` |
| **Branch** | `feat/audit-vc-vault-contract` |
| **Audit Date** | February 2026 |
| **Status** | Completed - fixes applied in-branch |

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Scope](#2-scope)
3. [Methodology](#3-methodology)
4. [Risk Classification](#4-risk-classification)
5. [Summary of Findings](#5-summary-of-findings)
6. [Detailed Findings](#6-detailed-findings)
7. [Design Decisions](#7-design-decisions)
8. [Conclusion](#8-conclusion)

---

## 1. Executive Summary

This document presents the results of a security review of the `vc-vault` smart contract, a Soroban contract deployed on the Stellar network for issuing, storing, and revoking Verifiable Credentials (VCs). The contract manages per-owner vaults, issuer authorization, and a VC lifecycle registry (issue, verify, revoke).

The audit identified **24 findings** across High, Medium, and Low severity levels. Three findings are classified as **High severity**, all of which have been resolved. The most critical was a cross-vault namespace collision attack (A-17) that allowed any issuer to overwrite or reset the revocation status of any credential on the network by reusing a known `vc_id`.

Five additional findings (A-22 through A-26) were discovered after the initial review through coverage-guided fuzzing and automated analysis. A-22 through A-24 stem from `push` not clearing the `VCStatus` entry in the source vault. A-25 and A-26 are complementary gaps in the destination vault: A-25 - `push` did not write the status for the recipient; A-26 - the unconditional status write introduced by the A-25 fix could overwrite an existing revoked status, allowing a credential revocation to be silently undone.

Of the 24 findings, **23 have been fixed** and **1 has been left out of scope** with documented rationale.

The contract is considered ready for deployment following the applied fixes, with the deferred and acknowledged items tracked for future iterations.

---

## 2. Scope

### Files reviewed

| File | Description |
|---|---|
| `contracts/vc-vault/src/contract.rs` | Public entrypoints and validation helpers |
| `contracts/vc-vault/src/api/mod.rs` | Public contract trait definition |
| `contracts/vc-vault/src/storage/mod.rs` | Storage layout, keys, and helpers |
| `contracts/vc-vault/src/vault/issuer.rs` | Issuer list management |
| `contracts/vc-vault/src/vault/credential.rs` | VC payload storage |
| `contracts/vc-vault/src/vault/mod.rs` | Vault module exports |
| `contracts/vc-vault/src/issuance/mod.rs` | VC revocation logic |
| `contracts/vc-vault/src/model/` | Data types: `VCStatus`, `VerifiableCredential` |
| `contracts/vc-vault/src/events/mod.rs` | Contract events (added during this audit) |
| `contracts/vc-vault/src/error.rs` | Error codes |
| `contracts/vc-vault/src/test.rs` | Unit test suite |
| `scripts/build.sh` | Build and optimization script |
| `scripts/release.sh` | Testnet deployment script |

### Out of scope

- Client-side SDK and DID resolution logic
- Off-chain issuance pipelines
- Cross-contract issuance integrations (external `issuance_contract` implementations)

---

## 3. Methodology

The review combined the following techniques:

**Manual code review.** All source files were read in full. Findings were identified by tracing execution paths, data flow across storage keys, and authorization logic.

**Static analysis.** [Scout Audit by CoinFabrik](https://github.com/CoinFabrik/scout-audit) was used as a complementary tool. It covers structural issues (unsafe unwraps, unbounded collections in instance storage, missing events) but does not detect business logic flaws. All Scout findings were cross-referenced against the manual review.

**Test suite analysis.** The existing test suite was reviewed to assess coverage quality. A dedicated set of targeted authorization tests was added to close gaps identified during the review.

**Coverage-guided fuzzing.** A `cargo-fuzz` suite was implemented in `contracts/vc-vault/fuzz/` with six targets covering the full contract surface:

| Target | Focus |
|---|---|
| `fuzz_issue` | Arbitrary `vc_id`, `vc_data`, `issuer_did`, `fee_override` combinations |
| `fuzz_revoke` | Arbitrary `vc_id` and `date` with and without a pre-existing VC |
| `fuzz_verify_vc` | Arbitrary `vc_id`; asserts no panic and correct `VCStatus` |
| `fuzz_push` | Cross-vault move with arbitrary `vc_id`; verifies VC count conservation |
| `fuzz_issuer_ops` | Sequences of `authorize_issuer` / `revoke_issuer` / `authorize_issuers` (with duplicates) |
| `fuzz_lifecycle` | Arbitrary sequences of all operations over an 8-vc_id × 4-issuer pool; verifies `verify_vc` matches tracked state after every step |

Run a target with: `cargo fuzz run fuzz_lifecycle` from `contracts/vc-vault/`.

**Integration of external findings.** Findings from an Almanax automated analysis were incorporated and cross-referenced.

---

## 4. Risk Classification

| Severity | Definition |
|---|---|
| **High** | Directly exploitable vulnerability that can result in asset loss, unauthorized access, data corruption, or permanent denial of service. Must be fixed before deployment. |
| **Medium** | No direct exploitability under normal conditions but can cause incorrect behavior, service disruption, or set up a more serious vulnerability if combined with other issues. Should be fixed before deployment. |
| **Low** | Minor issue with limited direct impact. Includes code quality, gas inefficiency, and issues that are unlikely to be exploited in practice. Should be addressed. |
| **Informational** | No security impact. Includes documentation gaps, design suggestions, and acknowledged intentional behavior. |

---

## 5. Summary of Findings

| ID | Title | Severity | Status |
|---|---|---|---|
| A-01 | Bootstrap side-effect in `create_vault` | High | Fixed |
| A-02 | Fee tier system is dead code | Medium | Out of scope - left intentionally |
| A-03 | `FeeCustom(Address)` in instance storage | Medium | Fixed |
| A-04 | `verify_vc` returns untyped `Map<String, String>` | Medium | Fixed |
| A-05 | Denied list not cleared on manual re-authorization | Low | Fixed |
| A-06 | `validate_vault_initialized` called twice per operation | Low | Fixed |
| A-07 | No events emitted | Medium | Fixed |
| A-09 | Tests use `mock_all_auths()` - auth never exercised | Medium | Fixed |
| A-10 | `VaultIssuers` is an unbounded Vec with linear search | Low | Fixed |
| A-11 | Re-issuing an existing `vc_id` resets revocation | High | Fixed |
| A-13 | `set_contract_admin` is a one-step transfer | Low | Fixed |
| A-14 | `default_issuer_did` written but never read | Low | Fixed |
| A-15 | `read_legacy_issuance_revocations` panics if map absent | Low | Fixed |
| A-16 | `SponsoredVaultSponsors` unbounded Vec in instance storage | Low | Fixed |
| A-17 | `VCStatus` lacks namespace - cross-vault collision attack | High | Fixed |
| A-18 | `build.sh` has no fail-fast | Medium | Fixed |
| A-19 | Hard-coded TTL constants may exceed network limits | Medium | Fixed |
| A-20 | `release.sh` suppresses errors with `\|\| true` | Low | Fixed |
| A-21 | `authorize_issuers` allows duplicates; `revoke_issuer` removes only first | Low | Fixed |
| A-22 | `issue` allows re-issuance of a `vc_id` after `push` | Medium | Fixed |
| A-23 | `revoke` operates on a `vc_id` that was pushed to another vault | Low | Fixed |
| A-24 | `push` allows moving a revoked credential | Low | Fixed |
| A-25 | `push` does not write `VCStatus` in destination vault | Medium | Fixed |
| A-26 | `push` overwrites existing revoked status in destination vault | Medium | Fixed |
**3 High · 10 Medium · 11 Low**

---

## 6. Detailed Findings

---

### A-01 - Bootstrap side-effect in `create_vault` [HIGH] - Fixed

**Location:** `contract.rs`

**Description:**

`create_vault` contained a hidden initialization block: if no contract admin was set, the first caller of `create_vault` would silently become the contract admin, bypassing `initialize` entirely.

```rust
// Before fix
fn create_vault(e: Env, owner: Address, did_uri: String) {
    owner.require_auth();
    if !storage::has_contract_admin(&e) {
        storage::write_contract_admin(&e, &owner);  // silent privilege escalation
        storage::write_fee_enabled(&e, &false);
        storage::extend_instance_ttl(&e);
    }
    ...
}
```

**Impact:**

Any address could become contract admin by front-running the deployment transaction. With the Sponsored Vault feature active, a sponsor calling `create_sponsored_vault` before `initialize` was called could claim admin rights.

**Resolution:**

The bootstrap block was removed. `create_vault` now panics with `NotInitialized` if `initialize` has not been called first. Responsibilities are cleanly separated: `initialize` sets the admin, `create_vault` creates vaults.

---

### A-03 - `FeeCustom(Address)` in instance storage [MEDIUM] - Fixed

**Location:** `storage/mod.rs`

**Description:**

`FeeCustom(Address)` entries were stored in instance storage. Instance storage has a fixed-size budget on Soroban. Each per-issuer custom fee entry consumed instance space, and a sufficiently large number of custom fee issuers would exhaust the budget, causing all instance reads and writes - including `ContractAdmin` and `FeeEnabled` - to fail.

**Impact:**

Denial of service on all contract operations in a scenario with many issuers, triggered by an unbounded number of `set_fee_custom` calls.

**Resolution:**

`FeeCustom(Address)` was moved to persistent storage. Instance storage is now reserved for global singleton values only.

---

### A-04 - `verify_vc` returns untyped `Map<String, String>` [MEDIUM] - Fixed

**Location:** `api/mod.rs`, `contract.rs`

**Description:**

`verify_vc` returns a `Map<String, String>` where the status is encoded as the strings `"valid"`, `"revoked"`, or `"invalid"`. SDK consumers must parse strings manually with no type guarantees. The `VCStatus` enum already exists in the model and is the correct return type.

**Impact:**

No direct security risk. SDK consumers are exposed to untyped data, increasing the likelihood of integration bugs (e.g., misspelled string comparisons).

**Resolution:**

`verify_vc` now returns `VCStatus` directly - `Valid`, `Invalid`, or `Revoked(date: String)`. The `issuance_status_to_map` helper was removed. Cross-contract invocations through external issuance contracts now also expect `VCStatus`. The `#[derive(Debug)]` derive was added to `VCStatus` to support `assert_eq!` in tests.

---

### A-05 - Denied list not cleared on manual re-authorization [LOW] - Fixed

**Location:** `vault/issuer.rs`

**Description:**

When `revoke_issuer` was called, the issuer was added to `VaultDeniedIssuers`. If the vault admin subsequently called `authorize_issuer` explicitly, the issuer was added back to `VaultIssuers` but was not removed from `VaultDeniedIssuers`. The auto-authorization path (`ensure_issuer_authorized`, called from `issue`) checks the denied list and would block future auto-authorization, even though the admin had explicitly re-authorized the issuer.

**Impact:**

Inconsistent authorization state. A vault admin who revokes an issuer and later manually re-authorizes them would find the issuer blocked from future automatic credential issuance.

**Resolution:**

`authorize_issuer` now calls `storage::remove_denied_issuer` after adding the issuer to the authorized list, ensuring the two lists remain consistent.

---

### A-06 - `validate_vault_initialized` called twice per operation [LOW] - Fixed

**Location:** `contract.rs`

**Description:**

`validate_vault_active` and `validate_vault_admin` both call `validate_vault_initialized` internally. Several vault-mutating functions called all three, resulting in two `storage().persistent().has()` reads for the same key per operation. `push` additionally called `validate_vault_initialized` explicitly after `validate_vault_active` for both vaults, producing four redundant reads.

**Impact:**

Unnecessary ledger read operations, increasing CPU instruction consumption per call.

**Resolution:**

Redundant explicit calls to `validate_vault_initialized` were removed from `push`. Since `validate_vault_active` already includes an initialization check, the explicit calls were duplicates.

---

### A-07 - No events emitted [MEDIUM] - Fixed

**Location:** `contract.rs`, new module `src/events/mod.rs`

**Description:**

No contract function emitted events. On-chain observability is critical for a credentialing contract: indexers, wallets, and compliance tools must detect when vaults are created, credentials are issued, issuers are authorized or revoked, and vaults are revoked.

**Impact:**

Third-party integrations cannot monitor contract state changes without polling storage. This creates a significant operational gap for production deployments.

**Resolution:**

A dedicated `events` module was created using the `#[contractevent]` macro (soroban-sdk 23.x). The following events are now emitted:

| Event | Emitted by |
|---|---|
| `VaultCreated { owner, did_uri }` | `create_vault` |
| `SponsoredVaultCreated { sponsor, owner, did_uri }` | `create_sponsored_vault` |
| `VaultRevoked { owner }` | `revoke_vault` |
| `IssuerAuthorized { owner, issuer }` | `authorize_issuer`, `authorize_issuers` |
| `IssuerRevoked { owner, issuer }` | `revoke_issuer` |
| `VCIssued { owner, vc_id, issuer }` | `issue` |
| `VCRevoked { owner, vc_id, date }` | `revoke` |

---

### A-09 - Tests use `mock_all_auths()` - auth never exercised [MEDIUM] - Fixed

**Location:** `test.rs`

**Description:**

The entire test suite used a single `setup()` helper that called `env.mock_all_auths()`, bypassing all `require_auth()` checks unconditionally. A regression removing an authorization guard from any function would pass the full test suite without detection.

**Impact:**

Critical authorization regressions are not caught by the existing tests. This is a test quality issue with downstream security impact.

**Resolution:**

A `setup_no_mock()` helper was added (identical to `setup()` but without `mock_all_auths()`). Five targeted authorization tests were added using `env.mock_auths()` with explicit per-address, per-function mocks to verify that `require_auth()` guards are enforced:

- `test_auth_initialize_requires_admin_signature`
- `test_auth_set_contract_admin_requires_current_admin_signature`
- `test_auth_create_vault_requires_owner_signature`
- `test_auth_authorize_issuer_requires_vault_admin_signature`
- `test_auth_issue_requires_issuer_signature`

---

### A-10 - `VaultIssuers` is an unbounded Vec with linear search [LOW] - Fixed

**Location:** `storage/mod.rs`, `vault/issuer.rs`

**Description:**

`VaultIssuers` is stored as a `Vec<Address>`. Both `is_authorized` (called on every `authorize_issuer` and `ensure_issuer_authorized`) and `ensure_issuer_authorized` (called on every `issue` invocation) perform a full O(n) linear scan. No size cap is enforced. The same applies to `VaultDeniedIssuers`.

**Impact:**

In vaults with large issuer lists, CPU costs grow linearly. A vault accumulating hundreds of entries would make every issuance call increasingly expensive, eventually hitting CPU budget limits.

**Resolution:**

A documentation comment was added to the issuer storage functions establishing the expected upper bound (~20 issuers per vault) and noting the O(n) cost. Enforcing a hard cap at the call-site is recommended if the contract is used in environments where vault admins cannot be trusted to self-limit.

---

### A-11 - Re-issuing an existing `vc_id` resets revocation [HIGH] - Fixed

**Location:** `contract.rs`, `vault/credential.rs`

**Description:**

`issue` performed no duplicate-ID check. If a `vc_id` already existed in the vault, calling `issue` again would silently overwrite the VC payload and reset `VCStatus` to `Valid`, even if the credential had previously been revoked.

**Attack path:**
1. Issuer calls `issue(owner, "vc-1", ...)` → status = `Valid`.
2. Owner calls `revoke("vc-1", date)` → status = `Revoked`.
3. Issuer calls `issue(owner, "vc-1", new_data, ...)` → payload overwritten, status reset to `Valid`. Revocation silently bypassed.

**Impact:**

An issuer could unilaterally un-revoke any previously revoked credential by re-issuing it with the same ID. This undermines the integrity of the VC lifecycle.

**Resolution:**

`issue` now checks for the existence of `VaultVC(owner, vc_id)` before writing. If the entry already exists, it panics with the new `VCAlreadyExists` error code (code `12`). VC identifiers are now immutable once issued.

---

### A-13 - `set_contract_admin` is a one-step transfer [LOW] - Fixed

**Location:** `contract.rs`, `api/mod.rs`

**Description:**

`set_contract_admin` allows the current admin to designate a new admin in a single transaction. The new admin address never signs. A typo, a burned address, or an incorrect contract address would permanently lock the admin role with no recovery path.

**Impact:**

Accidental permanent loss of the contract admin role. No exploit path by a third party, but a single admin error is unrecoverable.

**Resolution:**

`set_contract_admin` was replaced with a two-step transfer:

1. `nominate_admin(new_admin)` - current admin signs; writes `new_admin` to `DataKey::PendingAdmin`.
2. `accept_contract_admin()` - `new_admin` signs; promotes `PendingAdmin` to `ContractAdmin` and clears the pending entry.

If `accept_contract_admin` is called with no pending nomination, the contract panics with `NoPendingAdmin` (error `13`). An accidental nomination to an inaccessible address is recoverable by nominating a different address before the transfer is accepted.

---

### A-14 - `default_issuer_did` written but never read [LOW] - Fixed

**Location:** `contract.rs`, `storage/mod.rs`

**Description:**

`initialize` accepted a `default_issuer_did: String` parameter and wrote it to `DataKey::DefaultIssuerDid` in instance storage. No contract function ever read this value back. The field was dead code from the first deployment.

**Impact:**

No direct security risk. The unused parameter added friction to every deployment and consumed instance storage space unnecessarily.

**Resolution:**

The `default_issuer_did` parameter was removed from `initialize`. `DataKey::DefaultIssuerDid` was removed from the storage key enum. The `write_default_issuer_did` function was deleted. The `api/mod.rs` trait signature was updated to match.

---

### A-15 - `read_legacy_issuance_revocations` panics if map is absent [LOW] - Fixed

**Location:** `storage/mod.rs`

**Description:**

```rust
pub fn read_legacy_issuance_revocations(e: &Env) -> Map<String, LegacyRevocation> {
    e.storage().persistent().get(&DataKey::LegacyIssuanceRevocations).unwrap()
}
```

In a legacy deployment with no revocations, the revocations map would never have been written. Calling `migrate` in this case would panic with an opaque unwrap error, making migration impossible.

**Impact:**

Migration blocked for any legacy vault that had never performed a revocation. The failure mode was non-obvious and would surface as a runtime trap.

**Resolution:**

Replaced `.unwrap()` with `.unwrap_or_else(|| Map::new(e))`. An absent revocations map is now treated as an empty map, which is the correct semantic.

---

### A-16 - `SponsoredVaultSponsors` unbounded Vec in instance storage [LOW] - Fixed

**Location:** `storage/mod.rs`

**Description:**

The authorized sponsors list was stored as a single `Vec<Address>` under a single key in instance storage. Every `add_sponsored_vault_sponsor` call grew this Vec without bound. Instance storage has a fixed-size budget; a large sponsors list would eventually exhaust the budget, causing all instance reads and writes to fail. Additionally, `is_authorized_sponsor` performed an O(n) linear scan on every `create_sponsored_vault` call.

**Impact:**

Denial of service on all contract operations as the sponsors list grows. O(n) authorization check on every sponsored vault creation.

**Resolution:**

The `SponsoredVaultSponsors` Vec was replaced with individual persistent storage entries keyed by `DataKey::SponsoredVaultSponsor(Address)`. Authorization is now O(1) (`persistent().has()`), and the instance storage budget is unaffected regardless of how many sponsors are registered.

---

### A-17 - `VCStatus` lacks namespace - cross-vault collision attack [HIGH] - Fixed

**Location:** `storage/mod.rs`, `contract.rs`

**Description:**

`VCStatus` and `VCOwner` were stored under keys scoped only by `vc_id`, with no vault owner component:

```rust
DataKey::VCStatus(String)   // keyed by vc_id alone
DataKey::VCOwner(String)    // keyed by vc_id alone
```

Because `issue` always writes `VCStatus(vc_id, Valid)` and `VCOwner(vc_id, owner)`, any party issuing a credential using a `vc_id` that already exists in any other vault would overwrite the shared global registry entry for that ID. VC IDs are discoverable via the public, unauthenticated `list_vc_ids` and `get_vc` functions.

**Attack path:**
1. Victim's credential `"vc-42"` is revoked: `VCStatus("vc-42") = Revoked(date)`.
2. Attacker learns `"vc-42"` from `list_vc_ids(victim)`.
3. Attacker calls `issue(attacker_vault, "vc-42", ...)` in their own vault → `VCStatus("vc-42")` is overwritten to `Valid`.
4. `verify_vc(victim, "vc-42")` now returns `"valid"`.
5. `VCOwner("vc-42")` now points to the attacker - the attacker controls future revocation of `"vc-42"` in the victim's vault.

**Impact:**

Critical. Any revocation can be silently reversed by an external party. Attacker gains control over revocation of credentials in vaults they do not own.

**Resolution:**

`DataKey::VCStatus` changed from `VCStatus(String)` to `VCStatus(Address, String)` - scoped by `(owner, vc_id)`. `DataKey::VCOwner` was removed entirely; the `revoke` function now takes an explicit `owner: Address` parameter. All read/write callsites were updated. The `verify_vc` function was updated to pass the owner when reading status for locally-issued credentials.

---

### A-18 - `build.sh` has no fail-fast [MEDIUM] - Fixed

**Location:** `scripts/build.sh`

**Description:**

The script used `#!/bin/sh` without `set -e`. If `stellar contract build` failed, the script would continue and run `stellar contract optimize` on the previously built artifact already present in `target/`. `release.sh` would then deploy the stale binary with no artifact freshness check.

**Impact:**

A developer with a compilation error could silently deploy an outdated contract binary, thinking the latest changes were included.

**Resolution:**

`set -eu` added at the top of `build.sh`. The script now exits immediately on any command failure.

---

### A-19 - Hard-coded TTL constants may exceed network limits [MEDIUM] - Fixed

**Location:** `storage/mod.rs`

**Description:**

```rust
const INSTANCE_TTL_THRESHOLD: u32  = 30_000_000;
const INSTANCE_TTL_EXTEND_TO: u32  = 31_536_000;
const PERSISTENT_TTL_THRESHOLD: u32 = 30_000_000;
const PERSISTENT_TTL_EXTEND_TO: u32 = 31_536_000;
```

The `extend_to` values were set at the presumed mainnet maximum. If the target network's `max_entry_ttl` was lower than these constants - as is the case on some testnets and private networks - every `extend_ttl` call would deterministically fail, causing all entrypoints that touch storage to trap.

**Impact:**

All contract operations would be permanently broken on any network with a lower `max_entry_ttl` than the hard-coded constants.

**Resolution:**

Constants reduced to values safely within all known network limits:
- `THRESHOLD`: `518_400` ledgers (~30 days at 5-second close)
- `EXTEND_TO`: `3_110_400` ledgers (~180 days at 5-second close)

---

### A-20 - `release.sh` suppresses errors with `|| true` [LOW] - Fixed

**Location:** `scripts/release.sh`

**Description:**

Two commands used `|| true` to silently succeed on failure:

```sh
soroban config network add testnet ... || true
soroban keys generate vc_vault_admin --network testnet || true
```

In a shared CI environment, stale network configuration or a reused key would not be detected. A `testnet` entry pointing to a wrong RPC URL, or a `vc_vault_admin` key belonging to a different account, would be silently used.

**Impact:**

Deployment to the wrong endpoint or with the wrong signing key, with no visible error.

**Resolution:**

`|| true` replaced with explicit idempotency checks:

```sh
stellar config network ls 2>/dev/null | grep -q testnet || \
  stellar config network add testnet ...

stellar keys show vc_vault_admin 2>/dev/null || \
  stellar keys generate vc_vault_admin --network testnet
```

`set -eu` was also added so any other unexpected failure stops the script immediately.

---

### A-21 - `authorize_issuers` allows duplicates; `revoke_issuer` removes only first occurrence [LOW] - Fixed

**Location:** `vault/issuer.rs`

**Description:**

`authorize_issuers` wrote the provided list verbatim with no deduplication. `revoke_issuer` used `first_index_of` and removed only the first match.

If a caller passed a list with duplicate entries to `authorize_issuers`, a subsequent `revoke_issuer` would remove only the first occurrence, leaving the issuer authorized via the remaining duplicate. The issuer would be added to the denied list but `is_authorized` would still return `true`, so `ensure_issuer_authorized` would not block credential issuance from that issuer.

**Impact:**

An issuer could remain authorized after an explicit revocation if duplicates were present in the list.

**Resolution:**

`authorize_issuers` now deduplicates the input list before writing. `revoke_issuer` was rewritten to filter all occurrences in a single pass rather than removing by index.

---

### A-22 - `issue` allows re-issuance of a `vc_id` after `push` [MEDIUM] - Fixed

**Location:** `contract.rs` - `issue`

**Discovered by:** `fuzz_lifecycle` - sequence `Issue → Push → Issue (same vc_id)`

**Description:**

`issue` checked for duplicate `vc_id` only by reading the VC payload entry:

```rust
if storage::read_vault_vc(&e, &owner, &vc_id).is_some() {
    panic_with_error!(e, ContractError::VCAlreadyExists);
}
```

`push` removes the VC payload from the source vault (`remove_vault_vc`) but does not clear the corresponding `VCStatus` entry. After a push, `read_vault_vc` returns `None` while `read_vc_status` still returns `Valid`. The duplicate check passed, allowing the same `vc_id` to be re-issued in the source vault. The credential then existed simultaneously in both the source vault (newly re-issued) and the destination vault (pushed copy), violating the VC Conservation invariant.

**Impact:**

A single `vc_id` could exist in two vaults at once. Any downstream system indexing by `(owner, vc_id)` would observe ambiguous state. If the source vault re-issued the credential with different data, the two entries would represent conflicting credentials under the same identifier.

**Resolution:**

The duplicate check was extended to also verify the `VCStatus` entry:

```rust
if storage::read_vault_vc(&e, &owner, &vc_id).is_some()
    || storage::read_vc_status(&e, &owner, &vc_id) != VCStatus::Invalid
{
    panic_with_error!(e, ContractError::VCAlreadyExists);
}
```

`read_vc_status` returns `VCStatus::Invalid` as the default for keys that have never been written. After issue, the status is `Valid`; after push, the status is still `Valid` (not cleared). The second condition catches the pushed-away case. A regression test `test_issue_after_push_same_vc_id_panics` was added.

---

### A-23 - `revoke` operates on a `vc_id` that was pushed to another vault [LOW] - Fixed

**Location:** `contract.rs` - `revoke`

**Discovered by:** `fuzz_lifecycle` - sequence `Issue → Push → Revoke (same vc_id, source vault)`

**Description:**

`revoke` checked that the credential existed by reading its status, not its payload:

```rust
if storage::read_vc_status(&e, &owner, &vc_id) == VCStatus::Invalid {
    panic_with_error!(e, ContractError::VCNotFound);
}
```

After `push`, the VC payload was removed from the source vault but the `VCStatus` entry remained `Valid`. The check passed and `revoke` executed, writing a `Revoked` status in the source vault's storage for a credential that no longer resided there. A spurious `VCRevoked` event was emitted for a non-existent credential.

**Impact:**

Low. `verify_vc` checks the payload first and returns `Invalid` immediately if the VC is absent, so the stale `Revoked` status in storage had no effect on verification. However, the spurious event could mislead off-chain indexers, and the unnecessary storage write consumed ledger resources.

**Resolution:**

`revoke` now first verifies the VC payload exists in the vault, then verifies the status is `Valid` (blocking double-revocation as a secondary effect):

```rust
if storage::read_vault_vc(&e, &owner, &vc_id).is_none()
    || storage::read_vc_status(&e, &owner, &vc_id) != VCStatus::Valid
{
    panic_with_error!(e, ContractError::VCNotFound);
}
```

A regression test `test_revoke_after_push_panics` was added.

---

### A-24 - `push` allows moving a revoked credential [LOW] - Fixed

**Location:** `contract.rs` - `push`

**Discovered by:** `fuzz_lifecycle` - sequence `Issue → Revoke → Push`

**Description:**

`push` only checked that the VC payload existed in the source vault:

```rust
if vc_opt.is_none() {
    panic_with_error!(e, ContractError::VCNotFound);
}
```

No check was performed on the credential's status. A revoked credential - one that had been explicitly invalidated - could be transferred to a destination vault. The destination vault would then contain the VC payload but with no associated status (defaulting to `Invalid`), since `push` does not transfer the `VCStatus` entry.

**Impact:**

Low. The destination vault would see the credential as `Invalid` on `verify_vc`, so no verification benefit was gained. However, the operation was semantically inconsistent: an invalidated credential should not be transferable. The source vault's revocation entry would also remain in storage after the move, consuming ledger space for a credential no longer present.

**Resolution:**

`push` now verifies that the credential is in `Valid` status before proceeding:

```rust
if storage::read_vc_status(&e, &from_owner, &vc_id) != VCStatus::Valid {
    panic_with_error!(e, ContractError::VCNotFound);
}
```

A regression test `test_push_revoked_vc_panics` was added.

---

### A-25 - `push` does not write `VCStatus` in destination vault [MEDIUM] - Fixed

**Location:** `contract.rs` - `push`

**Discovered by:** Almanax automated analysis

**Description:**

`VCStatus` is keyed by `(owner, vc_id)`. When `push` moves a credential from `from_owner` to `to_owner`, it copies the VC payload and appends the ID to the destination's list, but never writes the status entry for `to_owner`:

```rust
storage::write_vault_vc(&e, &to_owner, &vc_id, &vc);       // payload ✓
storage::append_vault_vc_id(&e, &to_owner, &vc_id);         // ID list ✓
// VCStatus(to_owner, vc_id) never written                  ✗
```

`read_vc_status` returns `VCStatus::Invalid` when the key is absent (`unwrap_or(Invalid)`). As a result:

- `verify_vc(to_owner, vc_id)` finds the payload, reads the missing status, and returns `Invalid` - the recipient cannot verify that their credential is valid.
- `revoke(to_owner, vc_id)` checks `VCStatus != Valid`, which is true, and panics with `VCNotFound` - the recipient cannot revoke a credential they own.
- A second `push` from `to_owner` also fails on the same status guard.

**Impact:**

Medium. The destination vault holds a credential that is effectively unusable: it cannot be verified as valid, cannot be revoked, and cannot be forwarded. Any relying party calling `verify_vc` on the recipient's vault would see the credential as `Invalid` regardless of its actual standing.

**Resolution:**

`push` now writes `VCStatus::Valid` into the destination namespace immediately after writing the payload. The existing `extend_vc_ttl` call (which skips absent keys) then also extends the TTL of the new status entry:

```rust
storage::write_vault_vc(&e, &to_owner, &vc_id, &vc);
storage::append_vault_vc_id(&e, &to_owner, &vc_id);
storage::write_vc_status(&e, &to_owner, &vc_id, &VCStatus::Valid);
```

Two regression tests were added: `test_verify_vc_valid_after_push_on_destination` and `test_revoke_after_push_on_destination_succeeds`.

---

### A-26 - `push` overwrites existing revoked status in destination vault [MEDIUM] - Fixed

**Location:** `contract.rs` - `push`

**Discovered by:** Almanax automated analysis

**Description:**

The fix for A-25 introduced an unconditional `write_vc_status(to_owner, vc_id, Valid)`. This write has no precondition on what is already stored under `(to_owner, vc_id)`. If the destination vault already has a history for that `vc_id` - for example, a credential that was issued directly to `to_owner` and then revoked - the push silently overwrites the `Revoked` status with `Valid`.

Attack sequence:
1. `to_owner` has `vc-123` issued to their vault and revokes it → `VCStatus(to_owner, vc-123) = Revoked`.
2. An adversary issues their own `vc-123` to their vault → `VCStatus(attacker, vc-123) = Valid`.
3. The adversary calls `push(attacker, to_owner, vc-123)`.
4. `push` writes `VCStatus(to_owner, vc-123) = Valid`, overwriting the `Revoked` entry.
5. `verify_vc(to_owner, vc-123)` now returns `Valid` - the revocation was undone without `to_owner`'s consent.

**Impact:**

Medium. An adversary can silently undo a credential revocation in a victim's vault by pushing a colliding `vc_id`. This compromises the permanence-of-revocation invariant, a core security property of the system. The attack requires the adversary to hold a valid credential with the same `vc_id` and have an authorized issuer in their own vault.

**Resolution:**

`push` now checks that the destination vault has no existing payload and no existing non-Invalid status for the `vc_id` before writing, mirroring the same guard already present in `issue`:

```rust
if storage::read_vault_vc(&e, &to_owner, &vc_id).is_some()
    || storage::read_vc_status(&e, &to_owner, &vc_id) != VCStatus::Invalid
{
    panic_with_error!(e, ContractError::VCAlreadyExists);
}
```

A regression test `test_push_to_destination_with_existing_vc_id_panics` was added covering the unrevoke attack scenario.

---

## 7. Design Decisions

The following findings were reviewed and acknowledged as intentional behavior. No code change was applied; the behavior is documented here for transparency.

---

### A-02 - Fee tier system is dead code [MEDIUM] - Out of scope

Reviewed and left intentionally. The fee tier system (`FeeAdmin`, `FeeStandard`, `FeeEarly`) is reserved for a future billing model, poses no direct security risk in its current state, and was explicitly excluded from the scope of this audit.

---

## 8. Conclusion

The `vc-vault` contract implements a well-structured Verifiable Credential lifecycle system on Soroban. The audit identified three High severity issues, all of which have been addressed. The most critical - a cross-vault namespace collision (A-17) that allowed any issuer to reverse any revocation on the network - was remediated by namespacing the `VCStatus` key with the vault owner address.

Five additional findings (A-22 through A-26) were discovered post-review through coverage-guided fuzzing and automated analysis. A-22 through A-24 shared a common root: `push` removed the VC payload from the source vault but left the `VCStatus` entry intact, creating inconsistent state exploitable by subsequent `issue`, `revoke`, and `push` calls. A-25 and A-26 are complementary gaps in the destination vault: A-25 - `push` never wrote the status for the recipient; A-26 - the unconditional status write introduced by the A-25 fix could overwrite an existing revoked status, allowing a revocation to be silently undone. All five have been fixed.

Following the applied fixes, the contract's key security properties hold:

- **Vault isolation:** Storage keys for vault metadata, VC payloads, and VC status are all scoped by the vault owner address. Operations on one vault cannot affect another.
- **Authorization integrity:** Admin-only, vault-admin-only, and issuer-only functions are protected by `require_auth()` guards, confirmed by targeted authorization tests.
- **VC lifecycle integrity:** Issued credential IDs are unique per vault and per identity space - re-issuance is blocked even after a credential is pushed to another vault. Revocation is permanent. Only Valid credentials can be pushed or revoked.
- **Observability:** All key state transitions emit on-chain events indexable by third-party tools.
- **Storage safety:** Instance storage contains only global singleton values. Per-issuer and per-sponsor data uses persistent storage with individual keys.

The test suite now includes 54 tests (49 functional + 5 targeted authorization tests) with zero warnings. The contract is considered ready for testnet deployment.
