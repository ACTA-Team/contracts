# Security Policy

This repository holds the ACTA Soroban smart contracts and the normative
`did:stellar` specification. **The contracts are deployed on Stellar mainnet
and hold real value** — issuance fees are paid in USDC, and credential status
is read from them by third parties. Reports against this repository get the
highest priority of anything in the ACTA organization.

## Supported Versions

| Contract | Version | Supported |
| -------- | ------- | --------- |
| `vc-vault` | 0.4.x | :white_check_mark: |
| `vc-vault-factory` | 0.1.x | :white_check_mark: |
| `did-stellar-registry` | 0.2.x | :white_check_mark: |
| `did-stellar-registry` | 0.1.x (legacy testnet) | :x: |
| `vc-vault` | <= 0.3.x | :x: |
| `did:stellar` spec | v0.1 | :white_check_mark: |

Note what "supported" can and cannot mean on-chain: **`vc-vault` 0.4.0 is
immutable by design.** The `upgrade` entry point was removed to eliminate
code-swap rug-pull risk. A vulnerability in a deployed vault cannot be patched
in place — the response is a new vault version, a migration path via
`push`/`receive_push`, and public disclosure. Treat that as a reason to report
early, not a reason not to report.

### Deployed addresses in scope

**Mainnet**

| Contract | Address |
| -------- | ------- |
| `vc-vault-factory` | `CCWNZ6UMUXCDOVP2TWOPVLI4KP4VY4YF7VKPN6XLYVHNFAT24NDB33CX` |
| `did-stellar-registry` | `CD6LSWW5ZSXOO5WAIHKQLQ262TW7BPI37PNEVMMA273BAPC65NN2AYXQ` |
| `vc-vault` template WASM | `2bd0323a98acb8469606808368da6c79824f2dd8391494b94ddbeb3d22c1a957` |

**Testnet**

| Contract | Address |
| -------- | ------- |
| `vc-vault-factory` | `CDRFQRIP4FA3WMPWCSAM3XEY6EM6EGKRYZRSCSVZ5NHCF6AGEVR2XEPQ` |
| `did-stellar-registry` | `CBUNQ3GX3ZQ4MF64H7JCYZMXLGOS47VPIQQS7NCR6V3KX6YP7O72L5QF` |

## Scope

**In scope — report these**

- Any path that mutates a DID without its controller's authorization
- Any path that lets the contract admin mutate an individual DID. The admin is
  a governance role only and must never be able to do this
- Bypassing `require_auth()` on any state-changing entry point
- Issuing into a vault as a denied issuer, or bypassing the deny-list
- Fee bypass: issuing without paying, or redirecting the fee destination
- Breaking the optimistic-concurrency `expected_version` check to clobber a
  concurrent update
- Vault address derivation collisions, or front-running a `deploy_sponsored`
  so a vault lands under an attacker's control
- Reviving a deactivated (tombstoned) DID
- Cross-vault `push` accepting a source that is not a real vault, or an owner
  mismatch
- Arithmetic overflow, storage exhaustion, or TTL manipulation
- Errors or ambiguities in the spec that would let two conforming
  implementations disagree on whether a DID resolves

**Out of scope**

- Denial of service against public Stellar RPC endpoints, which we do not
  operate
- Griefing that costs the attacker more than the target
- Findings that require the victim to sign a transaction they can plainly read
- Gas and fee optimization without a security consequence
- Anything in `contracts-acta-spikes`, which is explicitly experimental and
  not deployed

## Reporting a Vulnerability

**Do not open a public issue, and do not exploit against mainnet.** If you
need to demonstrate an issue, do it on testnet.

Use GitHub's private reporting:
[**Report a vulnerability**](https://github.com/ACTA-Team/contracts-acta/security/advisories/new)

If you cannot use that form, email **acta.xyz@gmail.com** with `SECURITY` in
the subject.

Please include:

- Which contract and version, and the network you observed it on
- A failing test, a `cargo-fuzz` input, or a transaction hash on testnet
- The impact: what an attacker gains, and what a victim loses
- Whether funds, credential integrity, or DID control are affected

### What happens next

| Stage | Timeline |
| ----- | -------- |
| We acknowledge your report | Within **24 hours** |
| We confirm or reject it, with reasoning | Within **5 business days** |
| Mitigation deployed for a confirmed critical | Target **7 days** from confirmation |
| Fix released for high | Target **30 days** from confirmation |
| Fix released for moderate or low | Target **90 days** from confirmation |

Critical here means loss of funds, forged credentials, or unauthorized DID
control. Because vaults are immutable, "mitigation" for a deployed contract
may mean a factory-level change, a new vault version plus a migration, or a
public advisory telling issuers to stop using an affected path — whichever
protects users fastest.

You will get an update at each stage, and at least weekly while a critical or
high report stays open.

**If we accept the report:** we work the fix in a private advisory, credit you
however you prefer, and publish a GitHub Security Advisory alongside the fix
and any redeployment. We will tell you what we are deploying and when before
we do it.

**If we decline it:** we explain why in writing, in enough detail for you to
argue back. If you think we are wrong, say so — we will re-examine.

### Disclosure

We ask for **90 days** before public disclosure, or until a fix is deployed
and users can migrate, whichever comes first. For an issue affecting live
mainnet funds we may ask for more time and will tell you exactly why and for
how long. If we go silent or miss the timelines above, disclose — that is your
right and we will not contest it.

## Security practices

- 146 unit tests across the workspace (vc-vault 61, factory 27,
  did-registry 58), plus `cargo-fuzz` targets on the vault
- Release profile hardened: `overflow-checks = true`, `panic = "abort"`,
  `lto = true`, `strip = "symbols"`
- Two-step admin transfer (`propose_admin` / `accept_admin`) on the registry
  and factory, so a single mistaken transaction cannot hand over control
- Roles separated throughout: `owner` != `issuer` != `contract_admin`
- Prior audit reports live in [`audits/`](./audits)
