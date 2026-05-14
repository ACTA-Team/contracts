//! VC Vault Contract
//!
//! Soroban contract for Verifiable Credential storage and issuance status registry.
//! Provides per-owner vaults, issuer authorization, and VC lifecycle (issue, verify, revoke).

#![no_std]
#![allow(dead_code)]

pub mod constants;
mod interface;
pub mod contract;
mod validator;
mod error;
mod events;
pub mod types;
mod storage;
mod vault;

#[cfg(test)]
mod test;
