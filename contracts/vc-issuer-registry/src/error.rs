//! Contract error codes. Exposed as `Error(Contract, #code)` by Soroban.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// Caller is not the contract admin.
    Unauthorized = 1,
    /// initialize() has already been called.
    AlreadyInitialized = 2,
    /// Issuer address not found in the registry.
    IssuerNotFound = 3,
    /// Issuer address already registered.
    IssuerAlreadyExists = 4,
    /// Provided metadata fields are invalid (e.g. empty name).
    InvalidMetadata = 5,
    /// Contract has not been initialized yet.
    NotInitialized = 6,
}
