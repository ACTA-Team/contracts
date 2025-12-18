use soroban_sdk::contracterror;

/// Contract error codes.
///
/// Notes:
/// - Soroban exposes errors as `Error(Contract, #<code>)`.
/// - Keep these codes stable once deployed.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// The resource/namespace you are trying to initialize already exists.
    AlreadyInitialized = 1,

    /// The given issuer is not authorized for this vault.
    IssuerNotAuthorized = 2,

    /// The given issuer is already authorized for this vault.
    IssuerAlreadyAuthorized = 3,

    /// The vault is revoked; write operations are blocked.
    VaultRevoked = 4,

    /// Migration has already been executed (nothing to migrate).
    VCSAlreadyMigrated = 5,

    /// VC does not exist (status registry or vault lookup).
    VCNotFound = 6,

    /// VC is already revoked (cannot revoke twice).
    VCAlreadyRevoked = 7,

    /// Vault is not initialized for this owner.
    VaultNotInitialized = 8,

    /// Global configuration missing.
    NotInitialized = 9,

    /// `vault_contract` parameter is not this contract (kept only for backwards-compat).
    InvalidVaultContract = 10,
}
