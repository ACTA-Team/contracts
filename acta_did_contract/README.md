# ACTA Soroban DID Contract

W3C-compliant DID smart contract to manage decentralized identifiers on Soroban.

## Features
- Create a DID and its associated DID Document.
- Update verification methods and services.
- Retrieve the DID Document.
- Administration: `set_admin`, `upgrade`, `version`.

## Notes
- This contract can be deployed and referenced by ACTA Vault during `initialize`.
- Examples should use the `did:acta:` method.

## License
Apache 2.0.