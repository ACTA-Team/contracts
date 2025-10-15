# ACTA Contracts — W3C Standards Support

This document describes how the ACTA smart contracts integrate with W3C standards for Decentralized Identifiers (DIDs) and Verifiable Credentials (VCs), what parts are currently supported on-chain, and what is expected in the credential payloads.

## Overview

- The ACTA contract suite includes:
  - `VCIssuanceContract` — issues and manages VC status (valid/invalid/revoked) and exposes `issue`, `verify`, and `revoke` operations.
- `VaultContract` — multi-tenant vault that stores VC data by ID per `owner`, enforces issuer authorization per `owner`, and holds DID metadata for each vault owner.
  - (Optional off-chain DID resolution) — the vault stores a DID URI string only; it does not deploy nor persist a DID contract.

- The design is aligned with W3C DID Core (v1.0) and W3C Verifiable Credentials Data Model (v1.1) concepts:
  - DID Document fields (verification methods, services, context).
  - VC status lifecycle (valid, revoked) and verification surface.
  - VC payload stored on-chain as a string (typically JSON), referenced by `vc_id`.

## DID (W3C DID Core)

The `VaultContract` initializes and maintains DID-related metadata per vault/`owner`:

- During `initialize(owner, did_uri)`, the contract:
  - Sets an `admin` address for that vault (controller/owner).
  - Accepts a generative `did_uri` string and persists the **DID URI** scoped to the `owner`.

- DID Document content (as constructed in tests and initialization args):
  - `@context`: includes standard entries such as `https://www.w3.org/ns/did/v1` and security suite contexts, e.g. `https://w3id.org/security/suites/ed25519-2020/v1` and `https://w3id.org/security/suites/x25519-2020/v1`.
  - `verificationMethod`: entries with:
    - `id`: a fragment or identifier for the key (e.g., `keys-1`).
    - `type`: commonly `Ed25519VerificationKey2020`.
    - `publicKeyMultibase`: multibase-encoded public key material.
    - `controller`: optional controller field.
    - `verificationRelationship`: e.g. `Authentication`, `AssertionMethod`.
  - `service`: entries supporting W3C DID services, e.g., `LinkedDomains` with a `serviceEndpoint` URL.
  - `id`: DID URI (e.g., `did:acta:...`). The DID method string is configurable (tests use `acta`).

Notes:
- The vault does not deploy a DID contract; DID resolution is expected off-chain via the stored DID URI.
- Authentication for administrative operations is enforced per vault (`admin.require_auth()`), aligning with DID controller authorization semantics.

## Verifiable Credentials (W3C VC)

### Data Model on Chain

`VaultContract` stores verifiable credentials using the `VerifiableCredential` struct:

```
VerifiableCredential {
  id: String,             // VC identifier (maps to `vc_id`)
  data: String,           // VC payload (typically JSON aligned with W3C VC)
  issuance_contract: Address, // Address of the issuance contract that stored this VC
  issuer_did: String      // DID of the issuer
}
```

- `store_vc(e, owner, vc_id, vc_data, issuer, issuer_did, issuance_contract)`:
  - Validates vault is not revoked.
  - Validates the `issuer` is authorized and requires issuer auth (`issuer.require_auth()`).
  - Writes the VC under key `VC(vc_id)` in persistent storage.

### Issuance and Status

`VCIssuanceContract` exposes W3C-aligned issuance and status endpoints:

- `issue(e, vc_id, vc_data, vault_contract) -> String`:
  - Requires admin auth.
  - Invokes `VaultContract.store_vc(...)` to persist VC data.
  - Records VC status as `Valid`.
  - Returns the `vc_id`.

- `verify(e, vc_id) -> Map<String, String>`:
  - Reads VC status and returns a map resembling VC status data:
    - `{"status": "valid"}`
    - `{"status": "invalid"}`
    - `{"status": "revoked", "since": <revocation_date>}`

- `revoke(e, vc_id, date)`:
  - Requires admin auth.
  - Marks the VC as `Revoked(date)`.

### Authorization Model

`VaultContract` maintains an allow-list of issuers por bóveda/`owner`:
 
- `authorize_issuer(e, owner, issuer)` and `authorize_issuers(e, owner, issuers)` update `Issuers(owner)` storage.
- `revoke_issuer(e, owner, issuer)` removes an issuer.
- All write operations to the vault require issuer authorization and authentication.

### Read Access / Getters

- Current public interface does not expose getters for reading VCs from the vault (no `get_vc`/`list_vcs`).
- This means VC content on-chain cannot be retrieved via RPC unless dedicated read methods are added.
- The `verify` surface is provided via the issuance contract (status only).

## Expected VC Payload (`vc_data`)

Although `vc_data` is stored as a plain string, it should represent a JSON object aligned with W3C VC Data Model 1.1. A minimal recommended shape:

```json
{
  "@context": [
    "https://www.w3.org/2018/credentials/v1",
    "https://w3id.org/security/suites/ed25519-2020/v1"
  ],
  "type": ["VerifiableCredential"],
  "id": "urn:uuid:...",                
  "issuer": "did:acta:...",           
  "issuanceDate": "2025-01-01T00:00:00Z",
  "credentialSubject": {
    "id": "did:example:subject",
    "name": "Jane Doe",
    "achievement": "Blockchain Technology"
  },
  "proof": {                            
    "type": "Ed25519Signature2020",
    "created": "2025-01-01T00:00:00Z",
    "verificationMethod": "did:acta:...#keys-1",
    "proofPurpose": "assertionMethod",
    "jws": "eyJ..."                    
  }
}
```

Notes:
- The contract does not validate or parse the VC JSON; it stores it as provided.
- Proof verification (Linked Data Proofs / JWS) is not performed on-chain in the current implementation.

## Storage Layout

`VaultContract` uses Soroban storage keys con scope por `owner`:

- `ContractAdmin` (instance): Address del admin del contrato (autoriza `upgrade`).
- `Admin(owner)` (instance): Address del admin de la bóveda de `owner`.
- `Did(owner)` (instance): DID URI del `owner`.
- `Revoked(owner)` (instance): bandera booleana de revocación de la bóveda.
- `Issuers(owner)` (persistent): `Vec<Address>` de emisores autorizados para `owner`.
- `VC(owner, vc_id)` (persistent): `VerifiableCredential` almacenado por ID para `owner`.
- Ayudas de migración (`VCs(owner)` vector) existen para formatos de datos legacy.

Instance vs Persistent:
- Instance storage is tied to the current contract instance lifecycle.
- Persistent storage remains across upgrades and is intended for data such as issuers and VCs.

## Interoperability & Design Notes

- DID Alignment:
  - Uses standard DID contexts and verification methods.
  - Supports services like `LinkedDomains` to signal domain binding.

- VC Alignment:
  - On-chain storage is raw JSON in `vc_data`; status management via issuance contract.
  - `verify` surface returns simple maps for easy client consumption.

- Privacy & Access:
  - VC content is publicly readable if/when getters are added; consider storing hashes or encrypted payloads if sensitive.
  - Current design restricts write access via issuer authorization and admin controls.

- Upgradability:
  - Both `VaultContract` and `VCIssuanceContract` expose `upgrade(new_wasm_hash)`.
  - `VaultContract` also allows `set_admin(owner, new_admin)` (per‑vault admin).

## Roadmap / Potential Enhancements

- Add read methods to the vault (e.g., `get_vc(id)`, `list_vcs(page,size)`).
- On-chain proof verification or proof reference binding (hash of proof).
- DID resolution helpers and service discovery functions.
- Pagination and query filters for VCs (issuer DID, issuance date ranges).
- Optional encrypted `vc_data` or hash-only storage with off-chain retrieval.

## References

- W3C DID Core: https://www.w3.org/TR/did-core/
- W3C Verifiable Credentials Data Model 1.1: https://www.w3.org/TR/vc-data-model/
- Linked Data Proofs (Ed25519VerificationKey2020): https://w3id.org/security/suites/ed25519-2020/v1