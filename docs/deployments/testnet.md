# Testnet Deployments

Network passphrase: `Test SDF Network ; September 2015`  
RPC: `https://soroban-testnet.stellar.org:443`

---

## did-stellar-registry

| Version | Contract ID | Date | Notes |
|---|---|---|---|
| 0.1.0 | `CB7ATU7SF5QUKJMSULJDJVWJZVDXC23HTZX6NFUDTSFPVT6MA575NNZJ` | 2026-05-06 | Tranche 1 initial deploy |
| 0.2.0 | `CBUNQ3GX3ZQ4MF64H7JCYZMXLGOS47VPIQQS7NCR6V3KX6YP7O72L5QF` | 2026-06-22 | Allow key reuse across verification relationships; reject duplicate service `id_suffix`; admin-role spec; clippy. WASM hash `6835c23806075288284c89e133b271a3ac9c61977fbe49121f92c5431f29a0e7`. Superseded by 0.3.0. |
| 0.3.0 | `CAJQFHGAJR5Q2NMGM7IYGM2KK6FLQXT634XZMGEYKKOYN2E2ONCFRSQK` | 2026-08-10 | **Current.** Adds `register_sponsored` (sponsor pays and signs; controller owns the DID from version 1 without signing), error 22 `SponsorIsController`, event `DidRegisteredSponsored`. WASM hash `acf948403720ab652d9c20cf0dcf094287b15d1a7e330e584060d93b99fa78af`. |

## vc-vault-factory

| Version | Contract ID | Date | Notes |
|---|---|---|---|
| 0.1.0 | `CDRFQRIP4FA3WMPWCSAM3XEY6EM6EGKRYZRSCSVZ5NHCF6AGEVR2XEPQ` | 2026-06-22 | First release. Deploys single-tenant vaults (deterministic address from `keccak(salt‖XDR(owner))`), centralizes fee config, `is_vault` registry. Constructed with vault template WASM hash `2bd0323a98acb8469606808368da6c79824f2dd8391494b94ddbeb3d22c1a957`. WASM hash `f94a77905d87f9a195ea837414b4995c7d3d66bed0e287481710246bc1d5bdcd`. Superseded by the 2026-08-10 redeploy. |
| 0.1.0 | `CB23E4GXNRJ367BVPDRMGLADKBQLCWMAWP6SZVGFJPMR2D6I3KBA3B4H` | 2026-08-10 | **Current.** Redeploy alongside registry 0.3.0. Byte-identical WASM to the 2026-06-22 factory (same hash `f94a77905d87f9a195ea837414b4995c7d3d66bed0e287481710246bc1d5bdcd`) and same vault template hash `2bd0323a98acb8469606808368da6c79824f2dd8391494b94ddbeb3d22c1a957`; only the contract ID changed. Vaults deployed by the previous factory are NOT tracked by this one. |

### Fee configuration (factory `CB23E4GX…`, set 2026-08-10)

| Field | Value |
|---|---|
| enabled | `true` |
| token | `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` (native XLM SAC) |
| dest | _(fee destination wallet)_ |
| standard | `50000000` stroops = **5 XLM ≈ 1 USD** @ ~$0.20/XLM |

Charged per credential issued; the issuer pays. The amount is fixed on-chain (does
not track the USD price). Adjust with `set_fee_standard` / `set_fee_custom`, or
`set_fee_enabled false` to disable.

Values replicated verbatim from the 2026-06-22 factory. `set_fee_config` does not
enable fees on its own - `set_fee_enabled true` must follow it, or `quote_fee`
keeps returning `enabled: false`.

## vc-vault

Since v0.4.0 the vault is **single-tenant** and is no longer deployed standalone - individual vaults are instantiated by `vc-vault-factory.deploy(...)`. The release
publishes the vault as an installed WASM **template**; the factory is constructed
with its hash.

| Version | Contract ID / WASM hash | Date | Notes |
|---|---|---|---|
| 0.1.0 | `CC3SQ7UTAQQDQF6PUQMQIGK3BMPB22OKMHE5Y5XELEX3JFAKC72SQOAM` | 2026-05-06 | Tranche 1 initial deploy (multi-tenant). |
| 0.2.0 | `CBXC6LXBY5FGEG46VZ4AJ2AH2EJBINBA7BMILIEO4EJYI6ZTY7K7J5D5` | 2026-05-07 | SOW D2: O(1) index + pagination + migrate + batch_issue. WASM hash `c8da61dd3dd46b2810a743d50a388c09a00f0b7e8e2df7ceb5a71c8ce5dc4dd8`. |
| 0.3.0 | `CATL4IDH7XXPDC2UHSEX2GP45PPBVDFSKUDTKCSQICDOJVDLYNKISXFH` | 2026-05-14 | Refactoring: constructor, types rename, push_vc extraction, input caps, event emissions, O(1) issuer storage, tombstone TTL fix. WASM hash `775a141520de56fb4b1ebeb55d63e49fadf03f467ea8444cddb2caed2756ca8c`. |
| 0.4.0 | WASM hash `2bd0323a98acb8469606808368da6c79824f2dd8391494b94ddbeb3d22c1a957` (template; deployed via factory) | 2026-06-22 | Single-tenant rearchitecture + open issuance (deny-by-exception) + fees moved to factory; `upgrade` entrypoint removed (immutable). Installed as a template; instances created by `vc-vault-factory`. |

The template hash is unchanged as of the 2026-08-10 redeploy - the vault WASM
rebuilt from `dev` is byte-identical to the one installed on 2026-06-22, so the
upload was a no-op and the current factory `CB23E4GX…` was constructed with the
same hash.
