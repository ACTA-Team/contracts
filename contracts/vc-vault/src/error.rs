//! Contract error codes. Exposed as `Error(Contract, #code)` by Soroban.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// Resource already initialized (contract or vault).
    AlreadyInitialized = 1,
    /// Issuer not in vault's authorized list.
    IssuerNotAuthorized = 2,
    /// Issuer already authorized.
    IssuerAlreadyAuthorized = 3,
    /// Vault is revoked; writes blocked.
    VaultRevoked = 4,
    /// Migration already done; nothing to migrate.
    VCSAlreadyMigrated = 5,
    /// VC not found in vault or status registry.
    VCNotFound = 6,
    /// VC already revoked.
    VCAlreadyRevoked = 7,
    /// Vault not initialized for this owner.
    VaultNotInitialized = 8,
    /// Contract not initialized (no admin).
    NotInitialized = 9,
    /// vault_contract param is not this contract.
    InvalidVaultContract = 10,
}
