# ACTA Smart Contracts

Emisión, almacenamiento y verificación de credenciales verificables (VC) sobre Soroban.

Este monorepo contiene los contratos ACTA:
- `ACTA Issuance`: emisión, verificación y revocación de VCs.
- `ACTA Vault`: almacenamiento cifrado y autorización de emisores.
- `Deployer`: despliegue atómico de contratos y su inicialización.
- `DID` (opcional): contrato DID W3C (se compila/instala por separado).

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

El script imprime los WASM IDs y la dirección del Deployer.

## Licencia
Este software está licenciado bajo [Apache License 2.0](./LICENSE).
