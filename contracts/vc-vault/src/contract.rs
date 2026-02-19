//! Contract implementation: public entrypoints and validation helpers.

use crate::api::VcVaultTrait;
use crate::error::ContractError;
use crate::issuance;
use crate::model::VCStatus;
use crate::storage;
use crate::vault;
use soroban_sdk::{
    contract, contractimpl, contractmeta, panic_with_error, symbol_short, Address, BytesN, Env,
    IntoVal, Map, String, Vec,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

contractmeta!(
    key = "Description",
    val = "VC Vault: Verifiable Credential storage + issuance status registry",
);

/// Main contract struct. All public functions are exposed via the trait impl.
#[allow(dead_code)]
#[contract]
pub struct VcVaultContract;

#[contractimpl]
impl VcVaultTrait for VcVaultContract {
    // --- Global config ---

    fn initialize(e: Env, contract_admin: Address, default_issuer_did: String) {
        contract_admin.require_auth();
        if storage::has_contract_admin(&e) {
            panic_with_error!(e, ContractError::AlreadyInitialized);
        }
        storage::write_contract_admin(&e, &contract_admin);
        storage::write_default_issuer_did(&e, &default_issuer_did);
        storage::write_fee_enabled(&e, &false);
        storage::extend_instance_ttl(&e);
    }

    /// Set new contract admin. Caller must be current admin.
    fn set_contract_admin(e: Env, new_admin: Address) {
        let _ = validate_contract_admin(&e);
        storage::write_contract_admin(&e, &new_admin);
        storage::extend_instance_ttl(&e);
    }

    /// Configure fee: token, destination, amount. Admin only.
    fn set_fee_config(e: Env, token_contract: Address, fee_dest: Address, fee_amount: i128) {
        validate_contract_admin(&e);
        storage::write_fee_token_contract(&e, &token_contract);
        storage::write_fee_dest(&e, &fee_dest);
        storage::write_fee_amount(&e, &fee_amount);
        storage::extend_instance_ttl(&e);
    }

    /// Enable or disable fee charging on issue. Admin only.
    fn set_fee_enabled(e: Env, enabled: bool) {
        validate_contract_admin(&e);
        storage::write_fee_enabled(&e, &enabled);
        storage::extend_instance_ttl(&e);
    }

    fn set_fee_admin(e: Env, fee_amount: i128) {
        validate_contract_admin(&e);
        storage::write_fee_admin(&e, &fee_amount);
        storage::extend_instance_ttl(&e);
    }

    fn set_fee_standard(e: Env, fee_amount: i128) {
        validate_contract_admin(&e);
        storage::write_fee_standard(&e, &fee_amount);
        storage::extend_instance_ttl(&e);
    }

    fn set_fee_early(e: Env, fee_amount: i128) {
        validate_contract_admin(&e);
        storage::write_fee_early(&e, &fee_amount);
        storage::extend_instance_ttl(&e);
    }

    fn set_fee_custom(e: Env, issuer: Address, fee_amount: i128) {
        validate_contract_admin(&e);
        storage::write_fee_custom(&e, &issuer, &fee_amount);
        storage::extend_instance_ttl(&e);
    }

    fn get_fee_admin(e: Env) -> i128 {
        storage::extend_instance_ttl(&e);
        storage::read_fee_admin(&e)
    }

    fn get_fee_standard(e: Env) -> i128 {
        storage::extend_instance_ttl(&e);
        storage::read_fee_standard(&e)
    }

    fn get_fee_early(e: Env) -> i128 {
        storage::extend_instance_ttl(&e);
        storage::read_fee_early(&e)
    }

    fn get_fee_custom(e: Env, issuer: Address) -> i128 {
        storage::extend_instance_ttl(&e);
        storage::read_fee_custom(&e, &issuer)
    }

    /// Upgrade contract WASM. Admin only.
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        validate_contract_admin(&e);
        storage::extend_instance_ttl(&e);
        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn version(e: Env) -> String {
        String::from_str(&e, VERSION)
    }

    fn fee_config(e: Env) -> storage::FeeConfig {
        storage::extend_instance_ttl(&e);
        storage::read_fee_config(&e)
    }

    fn create_vault(e: Env, owner: Address, did_uri: String) {
        owner.require_auth();
        if !storage::has_contract_admin(&e) {
            storage::write_contract_admin(&e, &owner);
            storage::write_fee_enabled(&e, &false);
            storage::extend_instance_ttl(&e);
        }
        if storage::has_vault_admin(&e, &owner) {
            panic_with_error!(e, ContractError::AlreadyInitialized);
        }
        storage::write_vault_admin(&e, &owner, &owner);
        storage::write_vault_did(&e, &owner, &did_uri);
        storage::write_vault_revoked(&e, &owner, &false);
        storage::write_vault_issuers(&e, &owner, &Vec::new(&e));
        storage::extend_vault_ttl(&e, &owner);
    }

    /// Set vault admin. Current vault admin must sign.
    fn set_vault_admin(e: Env, owner: Address, new_admin: Address) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        storage::write_vault_admin(&e, &owner, &new_admin);
        storage::extend_vault_ttl(&e, &owner);
    }

    /// Replace full issuer list. Vault admin only.
    fn authorize_issuers(e: Env, owner: Address, issuers: Vec<Address>) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        vault::authorize_issuers(&e, &owner, &issuers);
        storage::extend_vault_ttl(&e, &owner);
    }

    /// Add single issuer. Vault admin only.
    fn authorize_issuer(e: Env, owner: Address, issuer_addr: Address) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        vault::authorize_issuer(&e, &owner, &issuer_addr);
        storage::extend_vault_ttl(&e, &owner);
    }

    /// Remove issuer from list. Vault admin only.
    fn revoke_issuer(e: Env, owner: Address, issuer_addr: Address) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        vault::revoke_issuer(&e, &owner, &issuer_addr);
        storage::extend_vault_ttl(&e, &owner);
    }

    /// Revoke vault. Blocks all writes. Vault admin only.
    fn revoke_vault(e: Env, owner: Address) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        storage::write_vault_revoked(&e, &owner, &true);
        storage::extend_vault_ttl(&e, &owner);
    }

    /// List VC IDs in owner's vault.
    fn list_vc_ids(e: Env, owner: Address) -> Vec<String> {
        storage::extend_vault_ttl(&e, &owner);
        storage::read_vault_vc_ids(&e, &owner)
    }

    /// Get VC payload by ID. Returns None if not found.
    fn get_vc(
        e: Env,
        owner: Address,
        vc_id: String,
    ) -> Option<crate::model::VerifiableCredential> {
        storage::extend_vault_ttl(&e, &owner);
        let vc = storage::read_vault_vc(&e, &owner, &vc_id);
        if vc.is_some() {
            storage::extend_vc_ttl(&e, &owner, &vc_id);
        }
        vc
    }

    /// Verify VC status. Returns map with "status" (valid/revoked/invalid) and optionally "since".
    fn verify_vc(e: Env, owner: Address, vc_id: String) -> Map<String, String> {
        storage::extend_vault_ttl(&e, &owner);
        let vc_opt = storage::read_vault_vc(&e, &owner, &vc_id);
        if vc_opt.is_none() {
            return issuance_status_to_map(&e, VCStatus::Invalid);
        }
        let vc = vc_opt.unwrap();
        storage::extend_vc_ttl(&e, &owner, &vc_id);
        let issuance_contract = vc.issuance_contract;
        if issuance_contract == e.current_contract_address() {
            let status = storage::read_vc_status(&e, &vc_id);
            return issuance_status_to_map(&e, status);
        }
        e.invoke_contract::<Map<String, String>>(
            &issuance_contract,
            &symbol_short!("verify"),
            (vc_id,).into_val(&e),
        )
    }

    /// Move VC from one vault to another. From-owner must sign. Issuer must be authorized in source.
    fn push(e: Env, from_owner: Address, to_owner: Address, vc_id: String, issuer_addr: Address) {
        validate_vault_active(&e, &from_owner);
        validate_vault_active(&e, &to_owner);
        validate_vault_initialized(&e, &from_owner);
        validate_vault_initialized(&e, &to_owner);
        from_owner.require_auth();
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

        storage::extend_vault_ttl(&e, &from_owner);
        storage::extend_vault_ttl(&e, &to_owner);
        storage::extend_vc_ttl(&e, &to_owner, &vc_id);
    }

    // --- Issuance ---

    /// Issue VC: store in vault, set status Valid. Issuer must sign and be authorized.
    fn issue(
        e: Env,
        owner: Address,
        vc_id: String,
        vc_data: String,
        vault_contract: Address,
        issuer_addr: Address,
        issuer_did: String,
        fee_override: i128,
    ) -> String {
        issuer_addr.require_auth();
        let this = e.current_contract_address();
        if vault_contract != this {
            panic_with_error!(e, ContractError::InvalidVaultContract);
        }
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
            fee_override,
        );

        storage::write_vc_status(&e, &vc_id, &VCStatus::Valid);
        storage::write_vc_owner(&e, &vc_id, &owner);
        storage::extend_vault_ttl(&e, &owner);
        storage::extend_vc_ttl(&e, &owner, &vc_id);

        vc_id
    }

    /// Revoke VC. Owner or contract admin must sign.
    fn revoke(e: Env, vc_id: String, date: String) {
        validate_vc_exists(&e, &vc_id);
        match storage::read_vc_owner(&e, &vc_id) {
            Some(owner) => owner.require_auth(),
            None => {
                let _ = validate_contract_admin(&e);
            }
        }
        issuance::revoke_vc(&e, vc_id.clone(), date);
        storage::extend_vc_status_ttl(&e, &vc_id);
    }

    // --- Migrations ---

    /// Migrate legacy storage. Some(owner) = vault migration; None = issuance registry migration.
    fn migrate(e: Env, owner: Option<Address>) {
        match owner {
            Some(owner) => {
                validate_vault_admin(&e, &owner);
                let vcs = storage::read_legacy_vault_vcs(&e, &owner);
                if vcs.is_none() {
                    panic_with_error!(e, ContractError::VCSAlreadyMigrated)
                }
                for vc in vcs.unwrap().iter() {
                    vault::store_vc(
                        &e,
                        &owner,
                        vc.id.clone(),
                        vc.data.clone(),
                        vc.issuance_contract.clone(),
                        vc.issuer_did.clone(),
                    );
                }
                storage::remove_legacy_vault_vcs(&e, &owner);
                storage::extend_vault_ttl(&e, &owner);
            }
            None => {
                validate_contract_admin(&e);
                storage::extend_instance_ttl(&e);
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
                    storage::extend_vc_status_ttl(&e, &vc_id);
                }
                storage::remove_legacy_issuance_vcs(&e);
                storage::remove_legacy_issuance_revocations(&e);
            }
        }
    }
}

// --- Validation helpers ---

/// Ensure contract admin exists and has signed. Returns admin address.
fn validate_contract_admin(e: &Env) -> Address {
    if !storage::has_contract_admin(e) {
        panic_with_error!(e, ContractError::NotInitialized)
    }
    let admin = storage::read_contract_admin(e);
    admin.require_auth();
    admin
}

/// Ensure vault exists for owner.
fn validate_vault_initialized(e: &Env, owner: &Address) {
    if !storage::has_vault_admin(e, owner) {
        panic_with_error!(e, ContractError::VaultNotInitialized)
    }
}

/// Ensure vault exists and caller is vault admin (has signed).
fn validate_vault_admin(e: &Env, owner: &Address) {
    validate_vault_initialized(e, owner);
    let admin = storage::read_vault_admin(e, owner);
    admin.require_auth();
}

/// Ensure vault exists and is not revoked.
fn validate_vault_active(e: &Env, owner: &Address) {
    validate_vault_initialized(e, owner);
    if storage::read_vault_revoked(e, owner) {
        panic_with_error!(e, ContractError::VaultRevoked)
    }
}

/// Ensure issuer is in vault's authorized list. No signature check.
fn validate_issuer_authorized_only(e: &Env, owner: &Address, issuer_addr: &Address) {
    validate_vault_initialized(e, owner);
    let issuers = storage::read_vault_issuers(e, owner);
    if !vault::is_authorized(&issuers, issuer_addr) {
        panic_with_error!(e, ContractError::IssuerNotAuthorized)
    }
}

/// Ensure VC exists in status registry (not Invalid).
fn validate_vc_exists(e: &Env, vc_id: &String) {
    if storage::read_vc_status(e, vc_id) == VCStatus::Invalid {
        panic_with_error!(e, ContractError::VCNotFound)
    }
}

/// Convert VCStatus to map for verify_vc return value.
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

/// Store VC in vault and charge fee if enabled.
fn store_vc_payload(
    e: &Env,
    owner: &Address,
    vc_id: String,
    vc_data: String,
    issuer_addr: &Address,
    issuer_did: String,
    issuance_contract: Address,
    fee_override: i128,
) {
    if storage::read_fee_enabled(e) {
        let fee_token = storage::read_fee_token_contract(e);
        let fee_dest = storage::read_fee_dest(e);
        if fee_override > 0 {
            e.invoke_contract::<()>(
                &fee_token,
                &symbol_short!("transfer"),
                (issuer_addr.clone(), fee_dest, fee_override).into_val(e),
            );
        }
    }
    vault::store_vc(e, owner, vc_id, vc_data, issuance_contract, issuer_did);
}
