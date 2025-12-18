use soroban_sdk::{Address, BytesN, Env, Map, String, Vec};

/// ACTA unified contract interface.
///
/// This contract merges the previous **Issuance** and **Vault** contracts into a
/// single Soroban contract.
///
/// High-level model:
/// - **Global (contract-level)** configuration: `initialize`, `set_contract_admin`, fee config, `upgrade`.
/// - **Per-owner vaults**: created with `create_vault`, managed with per-vault admin and issuer lists.
/// - **Issuance registry**: `issue`, `verify`, `revoke` keep a VC status registry by `vc_id`.
///
/// Privacy:
/// - `vc_data` is stored on-chain in the vault. Store **ciphertext only** (never plaintext PII).
#[allow(dead_code)]
pub trait ActaTrait {
    // -----------------------------
    // Global configuration
    // -----------------------------

    /// Initializes global configuration.
    ///
    /// - Sets the contract admin.
    /// - Stores a default issuer DID (optional metadata for off-chain UX).
    ///
    /// Can be called only once.
    fn initialize(e: Env, contract_admin: Address, default_issuer_did: String);

    /// Updates the global contract admin (admin-only).
    fn set_contract_admin(e: Env, new_admin: Address);

    /// Enables/disables global fee charging (admin-only).
    fn set_fee_enabled(e: Env, enabled: bool);

    /// Sets global fee configuration (admin-only).
    ///
    /// Fee charging happens inside `issue` when enabled.
    fn set_fee_config(e: Env, token_contract: Address, fee_dest: Address, fee_amount: i128);

    /// Upgrades the contract WASM (admin-only).
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>);

    /// Returns contract version.
    fn version(e: Env) -> String;

    // -----------------------------
    // Vault (per owner)
    // -----------------------------

    /// Creates/initializes a vault for `owner`.
    ///
    /// - Default vault admin = `owner`.
    /// - Sets DID URI metadata.
    /// - Initializes issuer list empty.
    fn create_vault(e: Env, owner: Address, did_uri: String);

    /// Sets the vault admin for `owner` (current vault admin-only).
    fn set_vault_admin(e: Env, owner: Address, new_admin: Address);

    /// Adds/overwrites authorized issuer list (vault admin-only).
    fn authorize_issuers(e: Env, owner: Address, issuers: Vec<Address>);

    /// Authorizes a single issuer (vault admin-only).
    fn authorize_issuer(e: Env, owner: Address, issuer: Address);

    /// Revokes an authorized issuer (vault admin-only).
    fn revoke_issuer(e: Env, owner: Address, issuer: Address);

    /// Revokes the whole vault (vault admin-only). Blocks writes.
    fn revoke_vault(e: Env, owner: Address);

    /// Lists VC IDs for the owner's vault.
    fn list_vc_ids(e: Env, owner: Address) -> Vec<String>;

    /// Reads a VC by ID for the owner's vault (public read).
    fn get_vc(
        e: Env,
        owner: Address,
        vc_id: String,
    ) -> Option<crate::verifiable_credential::VerifiableCredential>;

    /// Verifies VC status.
    ///
    /// If the VC exists in the vault, the contract:
    /// - uses `vc.issuance_contract` and calls `verify(vc_id)` on it.
    /// - if `issuance_contract` == this contract, it resolves locally.
    fn verify_vc(e: Env, owner: Address, vc_id: String) -> Map<String, String>;

    /// Push: moves a VC from one owner's vault to another.
    ///
    /// Requirements:
    /// - Both vaults must be active.
    /// - `from_owner` must sign.
    /// - `issuer` must be authorized in `from_owner` vault (signature not required).
    fn push(e: Env, from_owner: Address, to_owner: Address, vc_id: String, issuer: Address);

    // -----------------------------
    // Issuance (status registry)
    // -----------------------------

    /// Issues a new VC:
    /// - Stores payload in the owner's vault.
    /// - Stores status in this contract: `Valid`.
    /// - Records VC owner.
    ///
    /// Note: `vault_contract` is kept for backwards-compatibility but the unified contract
    /// always stores in its own vaults.
    fn issue(
        e: Env,
        owner: Address,
        vc_id: String,
        vc_data: String,
        vault_contract: Address,
        issuer: Address,
        issuer_did: String,
    ) -> String;

    /// Revokes a VC (owner-or-admin).
    fn revoke(e: Env, vc_id: String, date: String);

    // -----------------------------
    // Migrations
    // -----------------------------

    /// Migrates **legacy storage layouts**.
    ///
    /// - If `owner` is `Some`, migrates vault legacy VCs for that owner.
    /// - If `owner` is `None`, migrates legacy issuance status registry.
    fn migrate(e: Env, owner: Option<Address>);
}
