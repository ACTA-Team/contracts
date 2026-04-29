//! Contract error codes. Exposed as `Error(Contract, #code)` by Soroban.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// initialize() has already been called.
    AlreadyInitialized = 1,
    /// Issuer address not found in the registry.
    IssuerNotFound = 2,
    /// Issuer address already registered.
    IssuerAlreadyExists = 3,
    /// Contract has not been initialized yet.
    NotInitialized = 4,
    /// Metadata field exceeds maximum allowed size.
    InvalidMetadata = 5,
}
