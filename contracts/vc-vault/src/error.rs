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
    /// Signer is not the contract admin nor an authorized sponsor.
    NotAuthorizedSponsor = 11,
    /// vc_id already exists in this vault; re-issuance is not allowed.
    VCAlreadyExists = 12,
    /// accept_contract_admin called but no admin nomination is pending.
    NoPendingAdmin = 13,
    /// The parent VC does not exist or has been revoked.
    ParentVCInvalid = 14,
    /// Vault has reached the maximum number of active VCs.
    VaultFull = 15,
    /// Pagination `limit` exceeds `MAX_LIST_LIMIT`.
    LimitTooLarge = 16,
    /// Batch issuance request exceeds `MAX_BATCH_SIZE`.
    BatchTooLarge = 17,
    /// Batch issuance called with an empty `vcs` list.
    BatchEmpty = 18,
    /// String input exceeds its per-field maximum length (vc_id, vc_data,
    /// did_uri, issuer_did, or date).
    InputTooLong = 19,
    /// `authorize_issuers` called with a list larger than `MAX_ISSUERS_LIST`.
    IssuerListTooLong = 20,
    /// Issuer index already migrated; nothing to do.
    IssuersAlreadyMigrated = 21,
    /// Fee amount is negative.
    InvalidFeeAmount = 22,
    /// Fee amount exceeds `MAX_FEE_AMOUNT`.
    FeeOutOfBounds = 23,
}
