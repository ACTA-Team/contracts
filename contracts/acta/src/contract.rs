use crate::acta_trait::ActaTrait;
use crate::error::ContractError;
use crate::{issuer, storage, vc_status, verifiable_credential};
use crate::vc_status::VCStatus;
use soroban_sdk::{
    contract, contractimpl, contractmeta, panic_with_error, symbol_short, Address, BytesN, Env,
    IntoVal, Map, String, Vec,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

contractmeta!(
    key = "Description",
    val = "ACTA Unified Contract: Vault + Issuance (VC storage & status registry)",
);

/// ACTA Unified Contract.
///
/// Combines:
/// - Vault features (multi-tenant per owner)
/// - Issuance status registry (valid/revoked/invalid)
#[allow(dead_code)]
#[contract]
pub struct ActaContract;

#[contractimpl]
impl ActaTrait for ActaContract {
    // -----------------------------
    // Global configuration
    // -----------------------------

    /// Initialize global configuration (one-time).
    ///
    /// Parameters:
    /// - `contract_admin`: global admin address (must sign).
    /// - `default_issuer_did`: default issuer DID metadata (string).
    fn initialize(e: Env, contract_admin: Address, default_issuer_did: String) {
        // Prevent hostile initialization: the chosen admin must sign.
        contract_admin.require_auth();

        if storage::has_contract_admin(&e) {
            panic_with_error!(e, ContractError::AlreadyInitialized);
        }
        storage::write_contract_admin(&e, &contract_admin);
        storage::write_default_issuer_did(&e, &default_issuer_did);

        // Default fee disabled.
        storage::write_fee_enabled(&e, &false);
    }

    /// Set global contract admin (admin-only).
    ///
    /// Parameters:
    /// - `new_admin`: new admin address.
    fn set_contract_admin(e: Env, new_admin: Address) {
        let admin = validate_contract_admin(&e);
        let _ = admin; // keep for readability
        storage::write_contract_admin(&e, &new_admin);
    }

    /// Configure global fee (admin-only).
    ///
    /// Parameters:
    /// - `token_contract`: Soroban token contract address used for charging.
    /// - `fee_dest`: destination address to receive fees.
    /// - `fee_amount`: amount to transfer on each issuance/store (i128).
    fn set_fee_config(e: Env, token_contract: Address, fee_dest: Address, fee_amount: i128) {
        validate_contract_admin(&e);
        storage::write_fee_token_contract(&e, &token_contract);
        storage::write_fee_dest(&e, &fee_dest);
        storage::write_fee_amount(&e, &fee_amount);
    }

    /// Enable/disable fee charging (admin-only).
    ///
    /// Parameters:
    /// - `enabled`: `true` to charge fees on issuance, `false` otherwise.
    fn set_fee_enabled(e: Env, enabled: bool) {
        validate_contract_admin(&e);
        storage::write_fee_enabled(&e, &enabled);
    }

    /// Upgrade contract WASM (admin-only).
    ///
    /// Parameters:
    /// - `new_wasm_hash`: hash of the new WASM code.
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        validate_contract_admin(&e);
        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    /// Return the deployed contract version string.
    fn version(e: Env) -> String {
        String::from_str(&e, VERSION)
    }

    // -----------------------------
    // Vault (per owner)
    // -----------------------------

    /// Create/initialize a vault for an owner (one-time per owner).
    ///
    /// Parameters:
    /// - `owner`: vault owner address (must sign).
    /// - `did_uri`: DID URI metadata for the owner.
    fn create_vault(e: Env, owner: Address, did_uri: String) {
        // Prevent griefing: only the owner can initialize their own vault metadata.
        owner.require_auth();

        // If global admin is not initialized, keep backwards-compat UX:
        // first vault creator becomes global admin.
        if !storage::has_contract_admin(&e) {
            storage::write_contract_admin(&e, &owner);
            // fee disabled by default
            storage::write_fee_enabled(&e, &false);
        }

        if storage::has_vault_admin(&e, &owner) {
            panic_with_error!(e, ContractError::AlreadyInitialized);
        }

        storage::write_vault_admin(&e, &owner, &owner);
        storage::write_vault_did(&e, &owner, &did_uri);
        storage::write_vault_revoked(&e, &owner, &false);
        storage::write_vault_issuers(&e, &owner, &Vec::new(&e));
    }

    /// Set the per-vault admin (current vault admin must sign).
    ///
    /// Parameters:
    /// - `owner`: vault owner address (selects which vault).
    /// - `new_admin`: new admin address for that vault.
    fn set_vault_admin(e: Env, owner: Address, new_admin: Address) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        storage::write_vault_admin(&e, &owner, &new_admin);
    }

    /// Replace the full authorized issuer list for a vault (vault admin-only).
    ///
    /// Parameters:
    /// - `owner`: vault owner address.
    /// - `issuers`: list of issuer addresses allowed to issue into this vault.
    fn authorize_issuers(e: Env, owner: Address, issuers: Vec<Address>) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        issuer::authorize_issuers(&e, &owner, &issuers);
    }

    /// Add a single authorized issuer to a vault (vault admin-only).
    ///
    /// Parameters:
    /// - `owner`: vault owner address.
    /// - `issuer_addr`: issuer address to authorize.
    fn authorize_issuer(e: Env, owner: Address, issuer_addr: Address) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        issuer::authorize_issuer(&e, &owner, &issuer_addr);
    }

    /// Remove a single issuer from the authorized issuer list (vault admin-only).
    ///
    /// Parameters:
    /// - `owner`: vault owner address.
    /// - `issuer_addr`: issuer address to revoke.
    fn revoke_issuer(e: Env, owner: Address, issuer_addr: Address) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        issuer::revoke_issuer(&e, &owner, &issuer_addr)
    }

    /// Revoke a vault (vault admin-only). Blocks future writes.
    ///
    /// Parameters:
    /// - `owner`: vault owner address.
    fn revoke_vault(e: Env, owner: Address) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        storage::write_vault_revoked(&e, &owner, &true);
    }

    /// List VC IDs stored in a vault.
    ///
    /// Parameters:
    /// - `owner`: vault owner address.
    fn list_vc_ids(e: Env, owner: Address) -> Vec<String> {
        storage::read_vault_vc_ids(&e, &owner)
    }

    /// Get a VC payload from a vault (public read).
    ///
    /// Parameters:
    /// - `owner`: vault owner address.
    /// - `vc_id`: VC identifier.
    fn get_vc(
        e: Env,
        owner: Address,
        vc_id: String,
    ) -> Option<verifiable_credential::VerifiableCredential> {
        storage::read_vault_vc(&e, &owner, &vc_id)
    }

    /// Verify a VC status via the issuance registry (public read).
    ///
    /// Parameters:
    /// - `owner`: vault owner address (used only to check that the VC exists in that vault).
    /// - `vc_id`: VC identifier.
    fn verify_vc(e: Env, owner: Address, vc_id: String) -> Map<String, String> {
        // if not present in vault => invalid
        let vc_opt = storage::read_vault_vc(&e, &owner, &vc_id);
        if vc_opt.is_none() {
            return issuance_status_to_map(&e, VCStatus::Invalid);
        }

        let vc = vc_opt.unwrap();
        let issuance_contract = vc.issuance_contract;

        // If issuance contract is this contract, resolve locally.
        if issuance_contract == e.current_contract_address() {
            let status = storage::read_vc_status(&e, &vc_id);
            return issuance_status_to_map(&e, status);
        }

        // Otherwise, delegate to the external issuance contract's `verify(vc_id)`.
        e.invoke_contract::<Map<String, String>>(
            &issuance_contract,
            &symbol_short!("verify"),
            (vc_id,).into_val(&e),
        )
    }

    /// Move a VC from one owner's vault to another.
    ///
    /// Parameters:
    /// - `from_owner`: origin vault owner (must sign).
    /// - `to_owner`: destination vault owner.
    /// - `vc_id`: VC identifier to move.
    /// - `issuer_addr`: issuer address that must be authorized in `from_owner` vault (no signature required).
    fn push(e: Env, from_owner: Address, to_owner: Address, vc_id: String, issuer_addr: Address) {
        validate_vault_active(&e, &from_owner);
        validate_vault_active(&e, &to_owner);
        validate_vault_initialized(&e, &from_owner);
        validate_vault_initialized(&e, &to_owner);

        // Only the origin owner signs.
        from_owner.require_auth();

        // Issuer must be authorized in origin vault (signature not required).
        validate_issuer_authorized_only(&e, &from_owner, &issuer_addr);

        let vc_opt = storage::read_vault_vc(&e, &from_owner, &vc_id);
        if vc_opt.is_none() {
            panic_with_error!(e, ContractError::VCNotFound);
        }
        let vc = vc_opt.unwrap();

        storage::remove_vault_vc(&e, &from_owner, &vc_id);
        storage::remove_vault_vc_id(&e, &from_owner, &vc_id);

        storage::write_vault_vc(&e, &to_owner, &vc_id, &vc);
        storage::append_vault_vc_id(&e, &to_owner, &vc_id);
    }

    // -----------------------------
    // Issuance
    // -----------------------------

    /// Issue a VC: stores payload in vault + writes issuance status = `valid`.
    ///
    /// Parameters:
    /// - `owner`: vault owner that will receive the VC.
    /// - `vc_id`: VC identifier (application-defined).
    /// - `vc_data`: VC payload (ciphertext only).
    /// - `vault_contract`: kept for backwards-compat; must be this contract.
    /// - `issuer_addr`: issuer address (must sign and be authorized in owner's vault).
    /// - `issuer_did`: issuer DID metadata.
    fn issue(
        e: Env,
        owner: Address,
        vc_id: String,
        vc_data: String,
        vault_contract: Address,
        issuer_addr: Address,
        issuer_did: String,
    ) -> String {
        // Require issuer signature once (avoid double-auth when calling local vault).
        issuer_addr.require_auth();

        let this = e.current_contract_address();

        // Unified contract only: vault must be this contract (param kept for older clients).
        if vault_contract != this {
            panic_with_error!(e, ContractError::InvalidVaultContract);
        }

        // Local vault path:
        // - issuer already signed above
        // - we still must ensure issuer is authorized for the owner's vault
        validate_vault_active(&e, &owner);
        validate_vault_initialized(&e, &owner);
        validate_issuer_authorized_only(&e, &owner, &issuer_addr);

        store_vc_payload(
            &e,
            &owner,
            vc_id.clone(),
            vc_data,
            &issuer_addr,
            issuer_did,
            this.clone(),
        );

        // Update status registry in this contract.
        storage::write_vc_status(&e, &vc_id, &VCStatus::Valid);
        storage::write_vc_owner(&e, &vc_id, &owner);

        vc_id
    }

    /// Revoke a VC by ID.
    ///
    /// Parameters:
    /// - `vc_id`: VC identifier.
    /// - `date`: revocation date string (recommended ISO-8601).
    fn revoke(e: Env, vc_id: String, date: String) {
        validate_vc_exists(&e, &vc_id);

        match storage::read_vc_owner(&e, &vc_id) {
            Some(owner) => owner.require_auth(),
            None => {
                // Fallback to contract admin if owner not recorded.
                validate_contract_admin(&e);
            }
        }

        vc_status::revoke_vc(&e, vc_id, date);
    }

    // -----------------------------
    // Migrations
    // -----------------------------

    /// Migrate legacy storage layouts.
    ///
    /// Parameters:
    /// - `owner`: `Some(owner)` migrates that owner's vault legacy VCs; `None` migrates legacy issuance registry.
    fn migrate(e: Env, owner: Option<Address>) {
        match owner {
            Some(owner) => {
                // Vault legacy migration is per-owner and requires vault admin.
                validate_vault_admin(&e, &owner);

                let vcs = storage::read_legacy_vault_vcs(&e, &owner);
                if vcs.is_none() {
                    panic_with_error!(e, ContractError::VCSAlreadyMigrated)
                }

                for vc in vcs.unwrap().iter() {
                    verifiable_credential::store_vc(
                        &e,
                        &owner,
                        vc.id.clone(),
                        vc.data.clone(),
                        vc.issuance_contract.clone(),
                        vc.issuer_did.clone(),
                    );
                }

                storage::remove_legacy_vault_vcs(&e, &owner);
            }
            None => {
                // Issuance legacy migration is contract-admin only.
                validate_contract_admin(&e);

                let vcs = storage::read_legacy_issuance_vcs(&e);
                if vcs.is_none() {
                    panic_with_error!(e, ContractError::VCSAlreadyMigrated)
                }

                let revocations = storage::read_legacy_issuance_revocations(&e);

                for vc_id in vcs.unwrap().iter() {
                    match revocations.get(vc_id.clone()) {
                        Some(revocation) => {
                            storage::write_vc_status(&e, &vc_id.clone(), &VCStatus::Revoked(revocation.date))
                        }
                        None => storage::write_vc_status(&e, &vc_id, &VCStatus::Valid),
                    }
                }

                storage::remove_legacy_issuance_vcs(&e);
                storage::remove_legacy_issuance_revocations(&e);
            }
        }
    }
}

// -----------------------------
// Validation helpers
// -----------------------------

fn validate_contract_admin(e: &Env) -> Address {
    if !storage::has_contract_admin(e) {
        panic_with_error!(e, ContractError::NotInitialized)
    }

    let admin = storage::read_contract_admin(e);
    admin.require_auth();
    admin
}

fn validate_vault_initialized(e: &Env, owner: &Address) {
    if !storage::has_vault_admin(e, owner) {
        panic_with_error!(e, ContractError::VaultNotInitialized)
    }
}

fn validate_vault_admin(e: &Env, owner: &Address) {
    validate_vault_initialized(e, owner);
    let admin = storage::read_vault_admin(e, owner);
    admin.require_auth();
}

fn validate_vault_active(e: &Env, owner: &Address) {
    validate_vault_initialized(e, owner);
    let revoked = storage::read_vault_revoked(e, owner);
    if revoked {
        panic_with_error!(e, ContractError::VaultRevoked)
    }
}

fn validate_issuer_signed_and_authorized(e: &Env, owner: &Address, issuer_addr: &Address) {
    validate_vault_initialized(e, owner);

    let issuers: Vec<Address> = storage::read_vault_issuers(e, owner);
    if !issuer::is_authorized(&issuers, issuer_addr) {
        panic_with_error!(e, ContractError::IssuerNotAuthorized)
    }

    issuer_addr.require_auth();
}

fn validate_issuer_authorized_only(e: &Env, owner: &Address, issuer_addr: &Address) {
    validate_vault_initialized(e, owner);

    let issuers: Vec<Address> = storage::read_vault_issuers(e, owner);
    if !issuer::is_authorized(&issuers, issuer_addr) {
        panic_with_error!(e, ContractError::IssuerNotAuthorized)
    }
}

fn validate_vc_exists(e: &Env, vc_id: &String) {
    let status = storage::read_vc_status(e, vc_id);
    if status == VCStatus::Invalid {
        panic_with_error!(e, ContractError::VCNotFound)
    }
}

fn issuance_status_to_map(e: &Env, status: VCStatus) -> Map<String, String> {
    let status_k = String::from_str(e, "status");
    let since_k = String::from_str(e, "since");

    let revoked_v = String::from_str(e, "revoked");
    let valid_v = String::from_str(e, "valid");
    let invalid_v = String::from_str(e, "invalid");

    match status {
        VCStatus::Invalid => {
            let mut m = Map::new(e);
            m.set(status_k, invalid_v);
            m
        }
        VCStatus::Valid => {
            let mut m = Map::new(e);
            m.set(status_k, valid_v);
            m
        }
        VCStatus::Revoked(date) => {
            let mut m = Map::new(e);
            m.set(status_k, revoked_v);
            m.set(since_k, date);
            m
        }
    }
}

/// Stores a VC payload into a vault (local storage) and optionally charges fees.
///
/// Preconditions (must be enforced by caller):
/// - vault exists and is active
/// - issuer is authorized for the vault
/// - issuer has signed if this call path requires it
fn store_vc_payload(
    e: &Env,
    owner: &Address,
    vc_id: String,
    vc_data: String,
    issuer_addr: &Address,
    issuer_did: String,
    issuance_contract: Address,
) {
    // Fee charging (if enabled): transfer from issuer -> fee_dest.
    // Note: token contract itself will require auth from `issuer_addr` on transfer.
    if storage::read_fee_enabled(e) {
        let fee_token = storage::read_fee_token_contract(e);
        let fee_dest = storage::read_fee_dest(e);
        let fee_amount = storage::read_fee_amount(e);

        e.invoke_contract::<()>(
            &fee_token,
            &symbol_short!("transfer"),
            (issuer_addr.clone(), fee_dest, fee_amount).into_val(e),
        );
    }

    verifiable_credential::store_vc(
        e,
        owner,
        vc_id,
        vc_data,
        issuance_contract,
        issuer_did,
    );
}
