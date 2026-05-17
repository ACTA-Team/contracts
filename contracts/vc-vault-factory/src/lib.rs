#![no_std]

mod contract;
mod events;
mod storage;

#[cfg(test)]
mod test;

pub use contract::{VaultFactoryContract, VaultFactoryContractClient, VaultFactoryClient, VaultInitMeta};
