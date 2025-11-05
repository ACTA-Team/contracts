use soroban_sdk::{Address, BytesN, Env, String, Vec};

#[allow(dead_code)]
pub trait VaultTrait {
    /// Initializes a vault for an owner by setting the admin and DID.
    fn initialize(e: Env, owner: Address, did_uri: String);

    /// Authorizes a list of issuers for the owner's vault.
    fn authorize_issuers(e: Env, owner: Address, issuers: Vec<Address>);

    /// Authorizes an issuer for the owner's vault.
    fn authorize_issuer(e: Env, owner: Address, issuer: Address);

    /// Revokes an issuer for the owner's vault.
    fn revoke_issuer(e: Env, owner: Address, issuer: Address);

    /// Stores a verifiable credential in the owner's vault.
    fn store_vc(
        e: Env,
        owner: Address,
        vc_id: String,
        vc_data: String,
        issuer: Address,
        issuer_did: String,
        issuance_contract: Address,
    );

    /// Lists stored verifiable credential IDs for the owner's vault.
    fn list_vc_ids(e: Env, owner: Address) -> Vec<String>;

    /// Gets a verifiable credential by ID for the owner's vault.
    fn get_vc(e: Env, owner: Address, vc_id: String) -> Option<crate::verifiable_credential::VerifiableCredential>;

    /// Push: moves a VC from one owner's vault to another (issuer-only). No target authorization required.
    fn push(
        e: Env,
        from_owner: Address,
        to_owner: Address,
        vc_id: String,
        issuer: Address,
    );

    /// Revokes the owner's vault.
    fn revoke_vault(e: Env, owner: Address);

    /// Migrates the owner's VCs from single vector to keyed vectors.
    fn migrate(e: Env, owner: Address);

    /// Sets the new admin for the owner's vault.
    fn set_admin(e: Env, owner: Address, new_admin: Address);

    /// Upgrades WASM code.
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>);

    /// Returns the version of the contract.
    fn version(e: Env) -> String;

    /// Sets global fee configuration (admin only): token contract, destination, amount.
    fn set_fee_config(e: Env, token_contract: Address, fee_dest: Address, fee_amount: i128);

    /// Enables or disables global fee charging (admin only).
    fn set_fee_enabled(e: Env, enabled: bool);
}
