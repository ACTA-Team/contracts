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

/// Main contract struct. All public functions are exposed via the trait impl.
#[allow(dead_code)]
#[contract]
pub struct VcVaultContract;

#[contractimpl]
impl VcVaultContract {
    pub fn __constructor(e: Env, contract_admin: Address) {
        storage::write_contract_admin(&e, &contract_admin);
        storage::write_fee_enabled(&e, &false);
        storage::extend_instance_ttl(&e);
        events::contract_initialized(&e, &contract_admin);
    }
}

#[contractimpl]
impl VcVaultTrait for VcVaultContract {
    // --- Global config ---

    /// Nominate a new contract admin. Current admin must sign.
    /// The nominee must call accept_contract_admin to complete the transfer.
    fn nominate_admin(e: Env, new_admin: Address) {
        let current = require_contract_admin(&e);
        storage::write_pending_admin(&e, &new_admin);
        storage::extend_instance_ttl(&e);
        events::admin_nominated(&e, &current, &new_admin);
    }

    /// Accept a pending admin nomination. Nominee must sign.
    fn accept_contract_admin(e: Env) {
        let pending = match storage::read_pending_admin(&e) {
            Some(a) => a,
            None => panic_with_error!(e, ContractError::NoPendingAdmin),
        };
        pending.require_auth();
        // Capture the outgoing admin before overwriting so the event carries both
        // sides of the transfer.
        let old_admin = storage::read_contract_admin(&e);
        storage::write_contract_admin(&e, &pending);
        storage::remove_pending_admin(&e);
        storage::extend_instance_ttl(&e);
        events::admin_transferred(&e, &old_admin, &pending);
    }

    /// Configure fee: token, destination, amount. Admin only.
    fn set_fee_config(e: Env, token_contract: Address, fee_dest: Address, fee_amount: i128) {
        require_fee_amount(&e, fee_amount);
        require_contract_admin(&e);
        storage::write_fee_token_contract(&e, &token_contract);
        storage::write_fee_dest(&e, &fee_dest);
        storage::write_fee_amount(&e, &fee_amount);
        storage::extend_instance_ttl(&e);
        events::fee_config_set(&e, &token_contract, &fee_dest, fee_amount);
    }

    /// Enable or disable fee charging on issue. Admin only.
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

    /// Upgrade contract WASM. Admin only.
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

    fn create_vault(e: Env, owner: Address, did_uri: String) {
        require_did_uri_len(&e, &did_uri);
        if !storage::has_contract_admin(&e) {
            panic_with_error!(e, ContractError::NotInitialized);
        }
        owner.require_auth();
        if storage::has_vault_admin(&e, &owner) {
            panic_with_error!(e, ContractError::VaultAlreadyExists);
        }
        storage::write_vault_admin(&e, &owner, &owner);
        storage::write_vault_did(&e, &owner, &did_uri);
        storage::write_vault_revoked(&e, &owner, &false);
        storage::extend_vault_ttl(&e, &owner);
        events::vault_created(&e, &owner, &did_uri);
    }

    /// Set vault admin. Current vault admin must sign.
    fn set_vault_admin(e: Env, owner: Address, new_admin: Address) {
        require_vault_admin(&e, &owner);
        require_vault_active(&e, &owner);
        let old_admin = storage::read_vault_admin(&e, &owner);
        storage::write_vault_admin(&e, &owner, &new_admin);
        storage::extend_vault_ttl(&e, &owner);
        events::vault_admin_changed(&e, &owner, &old_admin, &new_admin);
    }

    /// Replace full issuer list. Vault admin only.
    fn authorize_issuers(e: Env, owner: Address, issuers: Vec<Address>) {
        require_issuers_list_len(&e, &issuers);
        require_vault_admin(&e, &owner);
        require_vault_active(&e, &owner);
        vault::authorize_issuers(&e, &owner, &issuers);
        storage::extend_vault_ttl(&e, &owner);
        for issuer in issuers.iter() {
            events::issuer_authorized(&e, &owner, &issuer);
        }
    }

    /// Add single issuer. Vault admin only.
    fn authorize_issuer(e: Env, owner: Address, issuer_addr: Address) {
        require_vault_admin(&e, &owner);
        require_vault_active(&e, &owner);
        vault::authorize_issuer(&e, &owner, &issuer_addr);
        storage::extend_vault_ttl(&e, &owner);
        events::issuer_authorized(&e, &owner, &issuer_addr);
    }

    /// Remove issuer from list. Vault admin only.
    fn revoke_issuer(e: Env, owner: Address, issuer_addr: Address) {
        require_vault_admin(&e, &owner);
        require_vault_active(&e, &owner);
        vault::revoke_issuer(&e, &owner, &issuer_addr);
        storage::extend_vault_ttl(&e, &owner);
        events::issuer_revoked(&e, &owner, &issuer_addr);
    }

    /// Revoke vault. Blocks all writes. Vault admin only.
    fn revoke_vault(e: Env, owner: Address) {
        require_vault_admin(&e, &owner);
        require_vault_active(&e, &owner);
        storage::write_vault_revoked(&e, &owner, &true);
        storage::extend_vault_ttl(&e, &owner);
        events::vault_revoked(&e, &owner);
    }

    /// List vc_ids active in owner's vault, paginated.
    ///
    /// Returns the slice `[offset, min(offset + limit, vc_count(owner)))`.
    /// Empty when `offset >= vc_count(owner)` or `limit == 0`. Panics with
    /// `LimitTooLarge` if `limit > MAX_LIST_LIMIT` so callers can't blow the
    /// CPU budget by asking for thousands of slots in a single call.
    ///
    /// Each enumerated slot has its TTL refreshed so vaults that are only
    /// ever listed (without `get_vc` calls on individual VCs) keep the
    /// index alive — otherwise `VaultVCIndex` entries could age out while
    /// `VaultVCCount` remains live, silently truncating future results.
    ///
    /// Use `vc_count(owner)` to size the iteration without reading any
    /// slot.
    fn list_vc_ids(e: Env, owner: Address, offset: u32, limit: u32) -> Vec<String> {
        if limit > storage::MAX_LIST_LIMIT {
            panic_with_error!(e, ContractError::LimitTooLarge);
        }
        storage::extend_vault_ttl(&e, &owner);
        let mut ids = Vec::new(&e);
        if limit == 0 {
            return ids;
        }
        let count = storage::read_vc_count(&e, &owner);
        if offset >= count {
            return ids;
        }
        let end = offset.saturating_add(limit).min(count);
        for i in offset..end {
            if let Some(vc_id) = storage::read_vc_id_at_extend(&e, &owner, i) {
                ids.push_back(vc_id);
            }
        }
        ids
    }

    /// Number of active vc_ids in owner's vault. O(1) — reads `VaultVCCount`
    /// directly without enumerating any slot. Returns 0 for unknown vaults
    /// (consistent with `read_vc_count`'s default).
    fn vc_count(e: Env, owner: Address) -> u32 {
        storage::extend_vault_ttl(&e, &owner);
        storage::read_vc_count(&e, &owner)
    }

    /// Get VC payload by ID. Returns None if not found.
    fn get_vc(
        e: Env,
        owner: Address,
        vc_id: String,
    ) -> Option<crate::types::VerifiableCredential> {
        require_vc_id_len(&e, &vc_id);
        storage::extend_vault_ttl(&e, &owner);
        let vc = storage::read_vault_vc(&e, &owner, &vc_id);
        if vc.is_some() {
            storage::extend_vc_ttl(&e, &owner, &vc_id);
        }
        vc
    }

    /// Verify VC status. Returns VCStatus::Valid, VCStatus::Revoked(date), or VCStatus::Invalid.
    fn verify_vc(e: Env, owner: Address, vc_id: String) -> VCStatus {
        require_vc_id_len(&e, &vc_id);
        storage::extend_vault_ttl(&e, &owner);
        let vc_opt = storage::read_vault_vc(&e, &owner, &vc_id);
        if vc_opt.is_none() {
            return VCStatus::Invalid;
        }
        let vc = vc_opt.unwrap();
        storage::extend_vc_ttl(&e, &owner, &vc_id);
        let issuance_contract = vc.issuance_contract;
        if issuance_contract == e.current_contract_address() {
            return storage::read_vc_status(&e, &owner, &vc_id);
        }
        e.invoke_contract::<VCStatus>(
            &issuance_contract,
            &symbol_short!("verify"),
            (vc_id,).into_val(&e),
        )
    }

    /// Moves a Valid VC from one vault to another; source owner and an authorized issuer must sign.
    fn push(e: Env, from_owner: Address, to_owner: Address, vc_id: String, issuer_addr: Address) {
        require_vc_id_len(&e, &vc_id);
        require_vault_active(&e, &from_owner);
        require_vault_active(&e, &to_owner);
        from_owner.require_auth();
        require_issuer_authorized(&e, &from_owner, &issuer_addr);
        vault::push_vc(&e, &from_owner, &to_owner, &vc_id);
    }

    // --- Issuance ---

    /// Issues a VC into the owner's vault; auto-authorizes the issuer if not already present.
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
        require_vc_id_len(&e, &vc_id);
        require_vc_data_len(&e, &vc_data);
        require_issuer_did_len(&e, &issuer_did);
        require_fee_amount(&e, fee_override);
        issuer_addr.require_auth();
        let this = e.current_contract_address();
        if vault_contract != this {
            panic_with_error!(e, ContractError::InvalidVaultContract);
        }
        require_vault_active(&e, &owner);
        ensure_issuer_authorized(&e, &owner, &issuer_addr);

        if storage::read_vault_vc(&e, &owner, &vc_id).is_some()
            || storage::read_vc_status(&e, &owner, &vc_id) != VCStatus::Invalid
        {
            panic_with_error!(e, ContractError::VCAlreadyExists);
        }

        vault::store_vc_with_fee(
            &e,
            &owner,
            vc_id.clone(),
            vc_data,
            &issuer_addr,
            issuer_did,
            this.clone(),
            fee_override,
        );

        storage::write_vc_status(&e, &owner, &vc_id, &VCStatus::Valid);
        storage::extend_vault_ttl(&e, &owner);
        storage::extend_vc_ttl(&e, &owner, &vc_id);
        events::vc_issued(&e, &owner, &vc_id, &issuer_addr);

        vc_id
    }

    /// Issues up to `MAX_BATCH_SIZE` VCs into owner's vault in a single
    /// transaction. Returns the issued vc_ids in input order.
    ///
    /// Compared to N sequential `issue()` calls this path:
    ///
    /// - takes one `issuer.require_auth()` for the whole batch,
    /// - charges a single fee transfer of `fee_override × n` (when fees are
    ///   enabled and `fee_override > 0`) instead of one per VC,
    /// - extends the vault TTL once at the end,
    /// - still emits a per-VC `VCIssued` event so off-chain indexers see
    ///   each credential individually.
    ///
    /// Cap rationale: Soroban allows ~25 ledger entry writes per
    /// transaction. Each VC writes 4 entries (`VaultVC`, `VaultVCIndex`,
    /// `VaultVCPosition`, `VCStatus`) plus 1 shared `VaultVCCount`, so
    /// `MAX_BATCH_SIZE = 5` lands at 21 entries with margin for the fee
    /// transfer. Larger batches must be split client-side.
    fn batch_issue(
        e: Env,
        issuer_addr: Address,
        owner: Address,
        vault_contract: Address,
        issuer_did: String,
        fee_override: i128,
        vcs: Vec<(String, String)>,
    ) -> Vec<String> {
        require_issuer_did_len(&e, &issuer_did);
        require_fee_amount(&e, fee_override);
        // Validate every (vc_id, vc_data) pair up front so an oversize entry
        // late in the batch doesn't waste CPU on the earlier valid ones.
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
        require_vault_active(&e, &owner);
        ensure_issuer_authorized(&e, &owner, &issuer_addr);

        // Single fee transfer for the entire batch, if enabled and the
        // caller requested a positive override. The contract already trusts
        // `issuer` to set fee_override per call (same as `issue`), so the
        // batch behaves like N issues at the same per-VC fee.
        if storage::read_fee_enabled(&e) && fee_override > 0 {
            let fee_token = storage::read_fee_token_contract(&e);
            let fee_dest = storage::read_fee_dest(&e);
            // saturating_mul is safe: at MAX_BATCH_SIZE=5 and any plausible
            // fee_override (≤ 10^16 stroops, the entire USDC supply order
            // of magnitude), the product fits in i128 by ~22 orders of
            // magnitude. The saturate-to-i128::MAX path would only fire
            // under deliberately absurd inputs, where the token contract
            // would reject the transfer for insufficient balance anyway.
            let total = fee_override.saturating_mul(n as i128);
            e.invoke_contract::<()>(
                &fee_token,
                &symbol_short!("transfer"),
                (issuer_addr.clone(), fee_dest, total).into_val(&e),
            );
        }

        // Issue each VC. Duplicate vc_ids — both within the batch and
        // against existing entries — are caught by the existence check on
        // the second iteration: the first write populates VaultVC and
        // VCStatus, so the next attempt with the same id fails with
        // VCAlreadyExists.
        let mut result = Vec::new(&e);
        for entry in vcs.iter() {
            let (vc_id, vc_data) = entry;
            if storage::read_vault_vc(&e, &owner, &vc_id).is_some()
                || storage::read_vc_status(&e, &owner, &vc_id) != VCStatus::Invalid
            {
                panic_with_error!(e, ContractError::VCAlreadyExists);
            }
            vault::store_vc(
                &e,
                &owner,
                vc_id.clone(),
                vc_data,
                this.clone(),
                issuer_did.clone(),
            );
            storage::write_vc_status(&e, &owner, &vc_id, &VCStatus::Valid);
            storage::extend_vc_ttl(&e, &owner, &vc_id);
            events::vc_issued(&e, &owner, &vc_id, &issuer_addr);
            result.push_back(vc_id);
        }

        // One vault TTL extend after all per-VC writes.
        storage::extend_vault_ttl(&e, &owner);
        result
    }

    /// Revoke VC. Owner must sign. The VC payload remains queryable via
    /// `get_vc(owner, vc_id)`; only the active index entry is removed so the
    /// vault doesn't fill up with revoked entries (each free slot can be
    /// reissued under a new vc_id, preserving the `MAX_VCS_PER_VAULT` cap as
    /// a *concurrent active* limit).
    fn revoke(e: Env, owner: Address, vc_id: String, date: String) {
        require_vc_id_len(&e, &vc_id);
        require_date_len(&e, &date);
        owner.require_auth();
        // VC must exist in this vault (not pushed away) and must not have been
        // revoked already. Checking vault_vc guards against the pushed-away case
        // since push removes the vc entry; checking status == Valid guards
        // against double-revocation.
        if storage::read_vault_vc(&e, &owner, &vc_id).is_none()
            || storage::read_vc_status(&e, &owner, &vc_id) != VCStatus::Valid
        {
            panic_with_error!(e, ContractError::VCNotFound);
        }
        vault::revoke_vc(&e, &owner, vc_id.clone(), date.clone());
        storage::remove_vc_from_index(&e, &owner, &vc_id);
        // remove_vc_from_index rewrites VaultVCCount and a moved VaultVCIndex
        // slot. write_vc_count and write_vc_id_at extend their own TTLs, but
        // the surrounding vault metadata (admin, did, revoked, issuers) also
        // benefits from a refresh on any mutation path so a near-expiry vault
        // stays consistent across all keys.
        storage::extend_vault_ttl(&e, &owner);
        storage::extend_vc_status_ttl(&e, &owner, &vc_id);
        events::vc_revoked(&e, &owner, &vc_id, &date);
    }

    // --- Linked VCs ---

    /// Issues a VC into owner's vault that references a parent VC in another vault.
    /// Validates that the parent VC is Valid before issuing. Issuer must sign.
    fn issue_linked(
        e: Env,
        issuer: Address,
        owner: Address,
        vc_id: String,
        data: String,
        issuance_contract: Address,
        issuer_did: String,
        parent_owner: Address,
        parent_vc_id: String,
    ) {
        require_vc_id_len(&e, &vc_id);
        require_vc_data_len(&e, &data);
        require_issuer_did_len(&e, &issuer_did);
        require_vc_id_len(&e, &parent_vc_id);
        issuer.require_auth();
        let this = e.current_contract_address();
        if issuance_contract != this {
            panic_with_error!(e, ContractError::InvalidVaultContract);
        }
        require_vault_active(&e, &owner);
        require_vault_initialized(&e, &parent_owner);

        // Both checks are required. The status keeps a Valid tombstone at the
        // source after `push` so vc_ids stay unique within a vault's history;
        // checking only status would let an attacker pass a vc_id that has
        // moved away (payload gone, status stale) and link a child to it. The
        // payload presence check pins the parent to its current holder.
        if storage::read_vault_vc(&e, &parent_owner, &parent_vc_id).is_none()
            || storage::read_vc_status(&e, &parent_owner, &parent_vc_id) != VCStatus::Valid
        {
            panic_with_error!(e, ContractError::ParentVCInvalid);
        }

        ensure_issuer_authorized(&e, &owner, &issuer);

        if storage::read_vault_vc(&e, &owner, &vc_id).is_some()
            || storage::read_vc_status(&e, &owner, &vc_id) != VCStatus::Invalid
        {
            panic_with_error!(e, ContractError::VCAlreadyExists);
        }

        vault::store_vc_with_fee(&e, &owner, vc_id.clone(), data, &issuer, issuer_did, this, 0);

        storage::write_vc_status(&e, &owner, &vc_id, &VCStatus::Valid);
        storage::write_vc_parent(&e, &owner, &vc_id, &parent_owner, &parent_vc_id);
        storage::extend_vault_ttl(&e, &owner);
        storage::extend_vc_ttl(&e, &owner, &vc_id);
        events::linked_vc_issued(&e, &issuer, &owner, &vc_id, &parent_owner, &parent_vc_id);
    }

    /// Returns Some((parent_owner, parent_vc_id)) if the VC was issued via issue_linked,
    /// or None if it is a regular VC with no parent link.
    fn get_vc_parent(e: Env, owner: Address, vc_id: String) -> Option<(Address, String)> {
        require_vc_id_len(&e, &vc_id);
        storage::extend_instance_ttl(&e);
        storage::read_vc_parent(&e, &owner, &vc_id)
    }

    // --- Sponsored vault ---

    /// Creates a vault on behalf of owner; sponsor must sign and be authorized unless open_to_all is enabled.
    fn create_sponsored_vault(e: Env, sponsor: Address, owner: Address, did_uri: String) {
        require_did_uri_len(&e, &did_uri);
        sponsor.require_auth();
        if !storage::has_contract_admin(&e) {
            panic_with_error!(e, ContractError::NotInitialized);
        }
        if !storage::read_sponsored_vault_open_to_all(&e) {
            let admin = storage::read_contract_admin(&e);
            if sponsor != admin && !storage::is_authorized_sponsor(&e, &sponsor) {
                panic_with_error!(e, ContractError::NotAuthorizedSponsor);
            }
        }
        if storage::has_vault_admin(&e, &owner) {
            panic_with_error!(e, ContractError::VaultAlreadyExists);
        }
        storage::write_vault_admin(&e, &owner, &owner);
        storage::write_vault_did(&e, &owner, &did_uri);
        storage::write_vault_revoked(&e, &owner, &false);
        storage::extend_vault_ttl(&e, &owner);
        storage::extend_instance_ttl(&e);
        events::sponsored_vault_created(&e, &sponsor, &owner, &did_uri);
    }

    /// Sets whether sponsored vault creation is restricted to authorized sponsors or open to all. Admin only.
    fn set_sponsored_vault_open_to_all(e: Env, open: bool) {
        require_contract_admin(&e);
        storage::write_sponsored_vault_open_to_all(&e, &open);
        storage::extend_instance_ttl(&e);
        events::sponsor_open_to_all_changed(&e, open);
    }

    /// Query whether sponsored vault creation is open to all.
    fn get_sponsored_vault_open_to_all(e: Env) -> bool {
        storage::extend_instance_ttl(&e);
        storage::read_sponsored_vault_open_to_all(&e)
    }

    /// Add an address to the authorized sponsors list. Admin only.
    fn add_sponsored_vault_sponsor(e: Env, sponsor: Address) {
        require_contract_admin(&e);
        storage::add_sponsored_vault_sponsor(&e, &sponsor);
        storage::extend_instance_ttl(&e);
        events::sponsor_added(&e, &sponsor);
    }

    /// Remove an address from the authorized sponsors list. Admin only.
    fn remove_sponsored_vault_sponsor(e: Env, sponsor: Address) {
        require_contract_admin(&e);
        storage::remove_sponsored_vault_sponsor(&e, &sponsor);
        storage::extend_instance_ttl(&e);
        events::sponsor_removed(&e, &sponsor);
    }

    fn list_authorized_issuers(e: Env, owner: Address, offset: u32, limit: u32) -> Vec<Address> {
        if limit > storage::MAX_LIST_LIMIT {
            panic_with_error!(e, ContractError::LimitTooLarge);
        }
        storage::extend_vault_ttl(&e, &owner);
        let mut result = Vec::new(&e);
        if limit == 0 {
            return result;
        }
        let count = storage::read_issuer_count(&e, &owner);
        if offset >= count {
            return result;
        }
        let end = offset.saturating_add(limit).min(count);
        for i in offset..end {
            if let Some(addr) = storage::read_issuer_at_extend(&e, &owner, i) {
                result.push_back(addr);
            }
        }
        result
    }

    fn list_denied_issuers(e: Env, owner: Address, offset: u32, limit: u32) -> Vec<Address> {
        if limit > storage::MAX_LIST_LIMIT {
            panic_with_error!(e, ContractError::LimitTooLarge);
        }
        storage::extend_vault_ttl(&e, &owner);
        let mut result = Vec::new(&e);
        if limit == 0 {
            return result;
        }
        let count = storage::read_denied_issuer_count(&e, &owner);
        if offset >= count {
            return result;
        }
        let end = offset.saturating_add(limit).min(count);
        for i in offset..end {
            if let Some(addr) = storage::read_denied_issuer_at_extend(&e, &owner, i) {
                result.push_back(addr);
            }
        }
        result
    }

    fn authorized_issuer_count(e: Env, owner: Address) -> u32 {
        storage::extend_vault_ttl(&e, &owner);
        storage::read_issuer_count(&e, &owner)
    }

    fn denied_issuer_count(e: Env, owner: Address) -> u32 {
        storage::extend_vault_ttl(&e, &owner);
        storage::read_denied_issuer_count(&e, &owner)
    }
}


