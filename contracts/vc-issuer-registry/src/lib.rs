//! VC Issuer Registry Contract
//!
//! Soroban contract for on-chain issuer governance and discovery.
//! Provides an admin-controlled allowlist with per-issuer metadata
//! (name, DID, URL, allowed flag).

#![no_std]

pub mod contract;
pub mod error;
pub mod events;
pub mod storage;

#[cfg(test)]
mod test;
