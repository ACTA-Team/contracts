# Testnet Deployments

Network passphrase: `Test SDF Network ; September 2015`  
RPC: `https://soroban-testnet.stellar.org:443`

---

## did-stellar-registry

| Version | Contract ID | Date | Notes |
|---|---|---|---|
| 0.1.0 | `CB7ATU7SF5QUKJMSULJDJVWJZVDXC23HTZX6NFUDTSFPVT6MA575NNZJ` | 2026-05-06 | Tranche 1 initial deploy |
| 0.2.0 | `CBUNQ3GX3ZQ4MF64H7JCYZMXLGOS47VPIQQS7NCR6V3KX6YP7O72L5QF` | 2026-06-22 | Allow key reuse across verification relationships; reject duplicate service `id_suffix`; admin-role spec; clippy. WASM hash `6835c23806075288284c89e133b271a3ac9c61977fbe49121f92c5431f29a0e7`. |

## vc-vault-factory

| Version | Contract ID | Date | Notes |
|---|---|---|---|
| 0.1.0 | `CBTNMBRD3TSGLPEZRD226U5LR7G3RWOYHCILAUEMRZAP7AHUZJ7CP4AB` | 2026-06-22 | First release. Deploys single-tenant vaults (deterministic address from `keccak(salt‖XDR(owner))`), centralizes fee config, `is_vault` registry. Constructed with vault template WASM hash `576b4c7b3d5aeafb9610bc569f45e1a007d2eff92855938a0bf933e657558ef9`. WASM hash `f94a77905d87f9a195ea837414b4995c7d3d66bed0e287481710246bc1d5bdcd`. |

## vc-vault

Since v0.4.0 the vault is **single-tenant** and is no longer deployed standalone —
individual vaults are instantiated by `vc-vault-factory.deploy(...)`. The release
publishes the vault as an installed WASM **template**; the factory is constructed
with its hash.

| Version | Contract ID / WASM hash | Date | Notes |
|---|---|---|---|
| 0.1.0 | `CC3SQ7UTAQQDQF6PUQMQIGK3BMPB22OKMHE5Y5XELEX3JFAKC72SQOAM` | 2026-05-06 | Tranche 1 initial deploy (multi-tenant). |
| 0.2.0 | `CBXC6LXBY5FGEG46VZ4AJ2AH2EJBINBA7BMILIEO4EJYI6ZTY7K7J5D5` | 2026-05-07 | SOW D2: O(1) index + pagination + migrate + batch_issue. WASM hash `c8da61dd3dd46b2810a743d50a388c09a00f0b7e8e2df7ceb5a71c8ce5dc4dd8`. |
| 0.3.0 | `CATL4IDH7XXPDC2UHSEX2GP45PPBVDFSKUDTKCSQICDOJVDLYNKISXFH` | 2026-05-14 | Refactoring: constructor, types rename, push_vc extraction, input caps, event emissions, O(1) issuer storage, tombstone TTL fix. WASM hash `775a141520de56fb4b1ebeb55d63e49fadf03f467ea8444cddb2caed2756ca8c`. |
| 0.4.0 | WASM hash `576b4c7b3d5aeafb9610bc569f45e1a007d2eff92855938a0bf933e657558ef9` (template; deployed via factory) | 2026-06-22 | Single-tenant rearchitecture + open issuance (deny-by-exception) + fees moved to factory. Installed as a template; instances created by `vc-vault-factory` `CBTNMBRD3TSGLPEZRD226U5LR7G3RWOYHCILAUEMRZAP7AHUZJ7CP4AB`. |
