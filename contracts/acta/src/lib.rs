#![no_std]
#![allow(dead_code)]

// Public contract entrypoint.
mod contract;

// Shared error codes.
mod error;

// Persistent/instance storage layout and helpers.
mod storage;

// Issuer authorization list management for vaults.
mod issuer;

// VC status registry (valid/revoked/invalid) for issued credentials.
mod vc_status;

// Verifiable Credential payload model stored in vaults.
mod verifiable_credential;

// Public interface (documented) for all external functions.
mod acta_trait;

#[cfg(test)]
mod test;
