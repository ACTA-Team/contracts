# Mainnet Deployments

Network passphrase: `Public Global Stellar Network ; September 2015`  
RPC: `https://mainnet.sorobanrpc.com`

---

## did-stellar-registry

| Version | Contract ID | Date | Notes |
|---|---|---|---|
| 0.2.0 | `CD6LSWW5ZSXOO5WAIHKQLQ262TW7BPI37PNEVMMA273BAPC65NN2AYXQ` | 2026-06-30 | Initial mainnet deploy. Admin = deployer wallet. WASM hash `6835c23806075288284c89e133b271a3ac9c61977fbe49121f92c5431f29a0e7`. |

## vc-vault-factory

| Version | Contract ID | Date | Notes |
|---|---|---|---|
| 0.1.0 | `CCWNZ6UMUXCDOVP2TWOPVLI4KP4VY4YF7VKPN6XLYVHNFAT24NDB33CX` | 2026-06-30 | Initial mainnet deploy. Constructed with vault template hash `2bd0323a98acb8469606808368da6c79824f2dd8391494b94ddbeb3d22c1a957`, admin = deployer wallet. WASM hash `f94a77905d87f9a195ea837414b4995c7d3d66bed0e287481710246bc1d5bdcd`. |

### Fee configuration (factory `CCWNZ6UM…`, set 2026-06-30)

| Field | Value |
|---|---|
| enabled | `true` |
| token | `CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75` (USDC SAC) |
| dest | _(deployer wallet)_ |
| standard | `10000000` = **1 USDC** per credential |

Charged per credential issued; the issuer pays. Fixed on-chain. Adjust with
`set_fee_standard` / `set_fee_custom`, or `set_fee_enabled false` to disable.

## vc-vault

| Version | Contract ID / WASM hash | Date | Notes |
|---|---|---|---|
| 0.4.0 | `2bd0323a98acb8469606808368da6c79824f2dd8391494b94ddbeb3d22c1a957` (template; deployed via factory) | 2026-06-30 | Installed as template on mainnet; instances created by factory `CCWNZ6UMUXCDOVP2TWOPVLI4KP4VY4YF7VKPN6XLYVHNFAT24NDB33CX`. |
