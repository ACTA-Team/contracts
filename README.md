# ACTA Smart Contracts

Issuance, storage, and verification of Verifiable Credentials (VC) on Soroban.

This monorepo contains the ACTA contracts:
- `ACTA Issuance`: issue, verify, and revoke VCs.
- `ACTA Vault`: encrypted storage and issuer authorization.
- `Deployer`: atomic deployment of contracts and initialization.
 - (Optional) Off-chain DID resolution: the vault stores a DID URI string.

## Build

```bash
chmod +x build.sh
sh build.sh
```

## Release (Testnet)

```bash
chmod +x release.sh
sh release.sh
```

The script prints the WASM IDs and the Deployer address.

## Deployed Contract IDs (Testnet)

- Issuance: `CAULJ65QZR4FCHAZGBUHMDACT7PODYIE54HGGOQWJRQFATAJ4U2HOUQK`
  - Explorer: https://stellar.expert/explorer/testnet/contract/CAULJ65QZR4FCHAZGBUHMDACT7PODYIE54HGGOQWJRQFATAJ4U2HOUQK
- Vault: `CCDAKJJROTWOEQS42TULG23YSM2OLGFKK43OQ3FRL6TQWQCC3QX4EUDH`
  - Explorer: https://stellar.expert/explorer/testnet/contract/CCDAKJJROTWOEQS42TULG23YSM2OLGFKK43OQ3FRL6TQWQCC3QX4EUDH
- Deployer: `CDSZBXTZQ6LHD2O5LKERPOXU226VE3IMYFGHAZGDGOSY3MU4SQHIIY5Y`
  - Explorer: https://stellar.expert/explorer/testnet/contract/CDSZBXTZQ6LHD2O5LKERPOXU226VE3IMYFGHAZGDGOSY3MU4SQHIIY5Y

## License
This software is licensed under the [Apache License 2.0](./LICENSE).
