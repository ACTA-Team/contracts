//! `did-stellar-registry`
//!
//! On-chain registry for the `did:stellar` v0.1 method. Implements the five
//! ABI functions defined in the spec — `register`, `update`,
//! `transfer_controller`, `deactivate`, and `get` — backed by Soroban
//! persistent storage. See `docs/did-spec/did-stellar-v0.1.md` in the repo
//! root for the normative specification.

#![no_std]

pub mod contract;
pub mod errors;
mod events;
pub mod model;
mod storage;

#[cfg(test)]
mod test;

pub use contract::{DidStellarRegistry, DidStellarRegistryInterface};
pub use errors::RegistryError;
pub use model::{DidKey, DidRecord, DidService};
