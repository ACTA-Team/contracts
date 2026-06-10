//! Contract implementation: public entrypoints.

use crate::interface::VcVaultTrait;
use crate::error::ContractError;
use crate::events;
use crate::types::VCStatus;
use crate::storage;
use crate::validator::*;
use crate::vault;
use soroban_sdk::{
    contract, contractimpl, contractmeta, panic_with_error, symbol_short, Address, BytesN, Env,
    IntoVal, String, Vec,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

contractmeta!(
    key = "Description",
    val = "VC Vault: Verifiable Credential storage + issuance status registry",
);

#[allow(dead_code)]
#[contract]
pub struct VcVaultContract;

#[contractimpl]
impl VcVaultContract {
    pub fn __constructor(e: Env, vault_owner: Address, contract_admin: Address, did_uri: String, factory_address: Address) {
        require_did_uri_len(&e, &did_uri);
        storage::write_vault_owner(&e, &vault_owner);
        storage::write_factory_address(&e, &factory_address);
        storage::write_contract_admin(&e, &contract_admin);
        storage::write_vault_did(&e, &did_uri);
        storage::write_vault_admin(&e, &vault_owner);
        storage::write_fee_enabled(&e, &false);
        storage::extend_instance_ttl(&e);
        events::contract_initialized(&e, &contract_admin);
        events::vault_created(&e, &vault_owner, &did_uri);
    }
}

#[contractimpl]
impl VcVaultTrait for VcVaultContract {
    // --- Global config ---

    fn nominate_admin(e: Env, new_admin: Address) {
        let current = require_contract_admin(&e);
        storage::write_pending_admin(&e, &new_admin);
        storage::extend_instance_ttl(&e);
        events::admin_nominated(&e, &current, &new_admin);
    }

    fn accept_contract_admin(e: Env) {
        let pending = match storage::read_pending_admin(&e) {
            Some(a) => a,
            None => panic_with_error!(e, ContractError::NoPendingAdmin),
        };
        pending.require_auth();
        let old_admin = storage::read_contract_admin(&e);
        storage::write_contract_admin(&e, &pending);
        storage::remove_pending_admin(&e);
        storage::extend_instance_ttl(&e);
        events::admin_transferred(&e, &old_admin, &pending);
    }

    fn set_fee_config(e: Env, token_contract: Address, fee_dest: Address, fee_amount: i128) {
        require_fee_amount(&e, fee_amount);
        require_contract_admin(&e);
        storage::write_fee_token_contract(&e, &token_contract);
        storage::write_fee_dest(&e, &fee_dest);
        storage::write_fee_amount(&e, &fee_amount);
        storage::extend_instance_ttl(&e);
        events::fee_config_set(&e, &token_contract, &fee_dest, fee_amount);
    }

    fn set_fee_enabled(e: Env, enabled: bool) {
        require_contract_admin(&e);
        storage::write_fee_enabled(&e, &enabled);
        storage::extend_instance_ttl(&e);
        events::fee_enabled_changed(&e, enabled);
    }

    fn set_fee_admin(e: Env, fee_amount: i128) {
        require_fee_amount(&e, fee_amount);
        require_contract_admin(&e);
        storage::write_fee_admin(&e, &fee_amount);
        storage::extend_instance_ttl(&e);
        events::fee_admin_set(&e, fee_amount);
    }

    fn set_fee_standard(e: Env, fee_amount: i128) {
        require_fee_amount(&e, fee_amount);
        require_contract_admin(&e);
        storage::write_fee_standard(&e, &fee_amount);
        storage::extend_instance_ttl(&e);
        events::fee_standard_set(&e, fee_amount);
    }

    fn set_fee_early(e: Env, fee_amount: i128) {
        require_fee_amount(&e, fee_amount);
        require_contract_admin(&e);
        storage::write_fee_early(&e, &fee_amount);
        storage::extend_instance_ttl(&e);
        events::fee_early_set(&e, fee_amount);
    }

    fn set_fee_custom(e: Env, issuer: Address, fee_amount: i128) {
        require_fee_amount(&e, fee_amount);
        require_contract_admin(&e);
        storage::write_fee_custom(&e, &issuer, &fee_amount);
        storage::extend_instance_ttl(&e);
        events::fee_custom_set(&e, &issuer, fee_amount);
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

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        require_contract_admin(&e);
        storage::extend_instance_ttl(&e);
        events::contract_upgraded(&e, &new_wasm_hash);
        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn version(e: Env) -> String {
        String::from_str(&e, VERSION)
    }

    fn fee_config(e: Env) -> storage::FeeConfig {
        storage::extend_instance_ttl(&e);
        storage::read_fee_config(&e)
    }

    // --- Vault management ---

    fn set_vault_admin(e: Env, new_admin: Address) {
        require_vault_admin(&e);
        require_vault_active(&e);
        let old_admin = storage::read_vault_admin(&e);
        storage::write_vault_admin(&e, &new_admin);
        storage::extend_vault_ttl(&e);
        events::vault_admin_changed(&e, &old_admin, &new_admin);
    }

    fn authorize_issuers(e: Env, issuers: Vec<Address>) {
        require_issuers_list_len(&e, &issuers);
        require_vault_admin(&e);
        require_vault_active(&e);
        vault::authorize_issuers(&e, &issuers);
        storage::extend_vault_ttl(&e);
        for issuer in issuers.iter() {
            events::issuer_authorized(&e, &issuer);
        }
    }

    fn authorize_issuer(e: Env, issuer_addr: Address) {
        require_vault_admin(&e);
        require_vault_active(&e);
        vault::authorize_issuer(&e, &issuer_addr);
        storage::extend_vault_ttl(&e);
        events::issuer_authorized(&e, &issuer_addr);
    }

    fn revoke_issuer(e: Env, issuer_addr: Address) {
        require_vault_admin(&e);
        require_vault_active(&e);
        vault::revoke_issuer(&e, &issuer_addr);
        storage::extend_vault_ttl(&e);
        events::issuer_revoked(&e, &issuer_addr);
    }

    fn revoke_vault(e: Env) {
        require_vault_admin(&e);
        require_vault_active(&e);
        storage::write_vault_revoked(&e, &true);
        storage::extend_vault_ttl(&e);
        events::vault_revoked(&e);
    }

    // --- Credential queries ---

    fn list_vc_ids(e: Env, offset: u32, limit: u32) -> Vec<String> {
        if limit > storage::MAX_LIST_LIMIT {
            panic_with_error!(e, ContractError::LimitTooLarge);
        }
        storage::extend_vault_ttl(&e);
        let mut ids = Vec::new(&e);
        if limit == 0 {
            return ids;
        }
        let count = storage::read_vc_count(&e);
        if offset >= count {
            return ids;
        }
        let end = offset.saturating_add(limit).min(count);
        for i in offset..end {
            if let Some(vc_id) = storage::read_vc_id_at_extend(&e, i) {
                ids.push_back(vc_id);
            }
        }
        ids
    }

    fn vc_count(e: Env) -> u32 {
        storage::extend_vault_ttl(&e);
        storage::read_vc_count(&e)
    }

    fn get_vc(e: Env, vc_id: String) -> Option<crate::types::VerifiableCredential> {
        require_vc_id_len(&e, &vc_id);
        storage::extend_vault_ttl(&e);
        let vc = storage::read_vault_vc(&e, &vc_id);
        if vc.is_some() {
            storage::extend_vc_ttl(&e, &vc_id);
        }
        vc
    }

    fn verify_vc(e: Env, vc_id: String) -> VCStatus {
        require_vc_id_len(&e, &vc_id);
        storage::extend_vault_ttl(&e);
        let vc_opt = storage::read_vault_vc(&e, &vc_id);
        if vc_opt.is_none() {
            return VCStatus::Invalid;
        }
        let vc = vc_opt.unwrap();
        storage::extend_vc_ttl(&e, &vc_id);
        let issuance_contract = vc.issuance_contract;
        if issuance_contract == e.current_contract_address() {
            return storage::read_vc_status(&e, &vc_id);
        }
        e.invoke_contract::<VCStatus>(
            &issuance_contract,
            &symbol_short!("verify"),
            (vc_id,).into_val(&e),
        )
    }

    // --- Issuance ---

    fn issue(
        e: Env,
        vc_id: String,
        vc_data: String,
        vault_contract: Address,
        issuer_addr: Address,
        issuer_did: String,
    ) -> String {
        require_vc_id_len(&e, &vc_id);
        require_vc_data_len(&e, &vc_data);
        require_issuer_did_len(&e, &issuer_did);
        issuer_addr.require_auth();
        let this = e.current_contract_address();
        if vault_contract != this {
            panic_with_error!(e, ContractError::InvalidVaultContract);
        }
        require_vault_active(&e);
        require_issuer_authorized(&e, &issuer_addr);

        if storage::read_vault_vc(&e, &vc_id).is_some()
            || storage::read_vc_status(&e, &vc_id) != VCStatus::Invalid
        {
            panic_with_error!(e, ContractError::VCAlreadyExists);
        }

        let factory = storage::read_factory_address(&e);
        vault::charge_fee(&e, &factory, &issuer_addr);

        vault::store_vc(&e, vc_id.clone(), vc_data, this.clone(), issuer_did);
        storage::write_vc_status(&e, &vc_id, &VCStatus::Valid);
        storage::extend_vault_ttl(&e);
        storage::extend_vc_ttl(&e, &vc_id);
        events::vc_issued(&e, &vc_id, &issuer_addr);
        vc_id
    }

    fn batch_issue(
        e: Env,
        issuer_addr: Address,
        vault_contract: Address,
        issuer_did: String,
        fee_override: i128,
        vcs: Vec<(String, String)>,
    ) -> Vec<String> {
        require_issuer_did_len(&e, &issuer_did);
        require_fee_amount(&e, fee_override);
        for entry in vcs.iter() {
            let (vc_id, vc_data) = entry;
            require_vc_id_len(&e, &vc_id);
            require_vc_data_len(&e, &vc_data);
        }
        issuer_addr.require_auth();
        let n = vcs.len();
        if n == 0 {
            panic_with_error!(e, ContractError::BatchEmpty);
        }
        if n > storage::MAX_BATCH_SIZE {
            panic_with_error!(e, ContractError::BatchTooLarge);
        }
        let this = e.current_contract_address();
        if vault_contract != this {
            panic_with_error!(e, ContractError::InvalidVaultContract);
        }
        require_vault_active(&e);
        ensure_issuer_authorized(&e, &issuer_addr);

        if storage::read_fee_enabled(&e) && fee_override > 0 {
            let fee_token = storage::read_fee_token_contract(&e);
            let fee_dest = storage::read_fee_dest(&e);
            let total = fee_override.saturating_mul(n as i128);
            e.invoke_contract::<()>(
                &fee_token,
                &symbol_short!("transfer"),
                (issuer_addr.clone(), fee_dest, total).into_val(&e),
            );
        }

        let mut result = Vec::new(&e);
        for entry in vcs.iter() {
            let (vc_id, vc_data) = entry;
            if storage::read_vault_vc(&e, &vc_id).is_some()
                || storage::read_vc_status(&e, &vc_id) != VCStatus::Invalid
            {
                panic_with_error!(e, ContractError::VCAlreadyExists);
            }
            vault::store_vc(
                &e,
                vc_id.clone(),
                vc_data,
                this.clone(),
                issuer_did.clone(),
            );
            storage::write_vc_status(&e, &vc_id, &VCStatus::Valid);
            storage::extend_vc_ttl(&e, &vc_id);
            events::vc_issued(&e, &vc_id, &issuer_addr);
            result.push_back(vc_id);
        }

        storage::extend_vault_ttl(&e);
        result
    }

    fn revoke(e: Env, vc_id: String, date: String) {
        require_vc_id_len(&e, &vc_id);
        require_date_len(&e, &date);
        let owner = storage::read_vault_owner(&e);
        owner.require_auth();
        if storage::read_vault_vc(&e, &vc_id).is_none()
            || storage::read_vc_status(&e, &vc_id) != VCStatus::Valid
        {
            panic_with_error!(e, ContractError::VCNotFound);
        }
        vault::revoke_vc(&e, vc_id.clone(), date.clone());
        storage::remove_vc_from_index(&e, &vc_id);
        storage::extend_vault_ttl(&e);
        storage::extend_vc_status_ttl(&e, &vc_id);
        events::vc_revoked(&e, &vc_id, &date);
    }

    fn push(e: Env, vc_id: String, dest_vault: Address) {
        require_vc_id_len(&e, &vc_id);
        require_vault_admin(&e);
        require_vault_active(&e);
        let vc = match storage::read_vault_vc(&e, &vc_id) {
            Some(v) => v,
            None => panic_with_error!(e, ContractError::VCNotFound),
        };
        if storage::read_vc_status(&e, &vc_id) != VCStatus::Valid {
            panic_with_error!(e, ContractError::VCNotFound);
        }
        e.invoke_contract::<()>(
            &dest_vault,
            &soroban_sdk::Symbol::new(&e, "receive_push"),
            (
                e.current_contract_address(),
                vc_id.clone(),
                vc.data,
                vc.issuer_did,
            ).into_val(&e),
        );
        storage::remove_vc_from_index(&e, &vc_id);
        storage::remove_vault_vc(&e, &vc_id);
        storage::remove_vc_status(&e, &vc_id);
        storage::extend_vault_ttl(&e);
        events::vc_pushed(&e, &vc_id, &dest_vault);
    }

    fn receive_push(e: Env, source_vault: Address, vc_id: String, vc_data: String, issuer_did: String) {
        require_vc_id_len(&e, &vc_id);
        require_vc_data_len(&e, &vc_data);
        require_issuer_did_len(&e, &issuer_did);
        require_vault_active(&e);
        source_vault.require_auth();
        // Verify source_vault was deployed by the same factory.
        let factory = storage::read_factory_address(&e);
        let is_legit: bool = e.invoke_contract(
            &factory,
            &soroban_sdk::Symbol::new(&e, "is_vault"),
            (source_vault.clone(),).into_val(&e),
        );
        if !is_legit {
            panic_with_error!(e, ContractError::SourceNotAVault);
        }
        // Mirror the duplicate guard used by issue()/batch_issue(): the index
        // entry is removed on revoke() but the Revoked status persists, so an
        // index-only check would let a pushed VC silently overwrite a revoked
        // credential back to Valid. Checking the status closes that bypass.
        if storage::read_vault_vc(&e, &vc_id).is_some()
            || storage::read_vc_status(&e, &vc_id) != VCStatus::Invalid
        {
            panic_with_error!(e, ContractError::VCAlreadyExists);
        }
        let dest = e.current_contract_address();
        vault::store_vc(&e, vc_id.clone(), vc_data, dest.clone(), issuer_did);
        storage::write_vc_status(&e, &vc_id, &VCStatus::Valid);
        storage::extend_vault_ttl(&e);
        storage::extend_vc_ttl(&e, &vc_id);
        events::vc_issued(&e, &vc_id, &dest);
    }

    // --- Issuer queries ---

    fn list_authorized_issuers(e: Env, offset: u32, limit: u32) -> Vec<Address> {
        if limit > storage::MAX_LIST_LIMIT {
            panic_with_error!(e, ContractError::LimitTooLarge);
        }
        storage::extend_vault_ttl(&e);
        let mut result = Vec::new(&e);
        if limit == 0 {
            return result;
        }
        let count = storage::read_issuer_count(&e);
        if offset >= count {
            return result;
        }
        let end = offset.saturating_add(limit).min(count);
        for i in offset..end {
            if let Some(addr) = storage::read_issuer_at_extend(&e, i) {
                result.push_back(addr);
            }
        }
        result
    }

    fn list_denied_issuers(e: Env, offset: u32, limit: u32) -> Vec<Address> {
        if limit > storage::MAX_LIST_LIMIT {
            panic_with_error!(e, ContractError::LimitTooLarge);
        }
        storage::extend_vault_ttl(&e);
        let mut result = Vec::new(&e);
        if limit == 0 {
            return result;
        }
        let count = storage::read_denied_issuer_count(&e);
        if offset >= count {
            return result;
        }
        let end = offset.saturating_add(limit).min(count);
        for i in offset..end {
            if let Some(addr) = storage::read_denied_issuer_at_extend(&e, i) {
                result.push_back(addr);
            }
        }
        result
    }

    fn authorized_issuer_count(e: Env) -> u32 {
        storage::extend_vault_ttl(&e);
        storage::read_issuer_count(&e)
    }

    fn denied_issuer_count(e: Env) -> u32 {
        storage::extend_vault_ttl(&e);
        storage::read_denied_issuer_count(&e)
    }
}
