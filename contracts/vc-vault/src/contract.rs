//! Contract implementation: public entrypoints and validation helpers.

use crate::api::VcVaultTrait;
use crate::error::ContractError;
use crate::events;
use crate::issuance;
use crate::model::VCStatus;
use crate::storage;
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
impl VcVaultTrait for VcVaultContract {
    // --- Global config ---

    fn initialize(e: Env, contract_admin: Address) {
        contract_admin.require_auth();
        if storage::has_contract_admin(&e) {
            panic_with_error!(e, ContractError::AlreadyInitialized);
        }
        storage::write_contract_admin(&e, &contract_admin);
        storage::write_fee_enabled(&e, &false);
        storage::extend_instance_ttl(&e);
    }

    /// Nominate a new contract admin. Current admin must sign.
    /// The nominee must call accept_contract_admin to complete the transfer.
    fn nominate_admin(e: Env, new_admin: Address) {
        let _ = validate_contract_admin(&e);
        storage::write_pending_admin(&e, &new_admin);
        storage::extend_instance_ttl(&e);
    }

    /// Accept a pending admin nomination. Nominee must sign.
    fn accept_contract_admin(e: Env) {
        let pending = match storage::read_pending_admin(&e) {
            Some(a) => a,
            None => panic_with_error!(e, ContractError::NoPendingAdmin),
        };
        pending.require_auth();
        storage::write_contract_admin(&e, &pending);
        storage::remove_pending_admin(&e);
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
        if !storage::has_contract_admin(&e) {
            panic_with_error!(e, ContractError::NotInitialized);
        }
        owner.require_auth();
        if storage::has_vault_admin(&e, &owner) {
            panic_with_error!(e, ContractError::AlreadyInitialized);
        }
        storage::write_vault_admin(&e, &owner, &owner);
        storage::write_vault_did(&e, &owner, &did_uri);
        storage::write_vault_revoked(&e, &owner, &false);
        storage::write_vault_issuers(&e, &owner, &Vec::new(&e));
        storage::extend_vault_ttl(&e, &owner);
        events::vault_created(&e, &owner, &did_uri);
    }

    /// Set vault admin. Current vault admin must sign.
    fn set_vault_admin(e: Env, owner: Address, new_admin: Address) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        let old_admin = storage::read_vault_admin(&e, &owner);
        storage::write_vault_admin(&e, &owner, &new_admin);
        storage::extend_vault_ttl(&e, &owner);
        events::vault_admin_changed(&e, &owner, &old_admin, &new_admin);
    }

    /// Replace full issuer list. Vault admin only.
    fn authorize_issuers(e: Env, owner: Address, issuers: Vec<Address>) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        vault::authorize_issuers(&e, &owner, &issuers);
        storage::extend_vault_ttl(&e, &owner);
        for issuer in issuers.iter() {
            events::issuer_authorized(&e, &owner, &issuer);
        }
    }

    /// Add single issuer. Vault admin only.
    fn authorize_issuer(e: Env, owner: Address, issuer_addr: Address) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        vault::authorize_issuer(&e, &owner, &issuer_addr);
        storage::extend_vault_ttl(&e, &owner);
        events::issuer_authorized(&e, &owner, &issuer_addr);
    }

    /// Remove issuer from list. Vault admin only.
    fn revoke_issuer(e: Env, owner: Address, issuer_addr: Address) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
        vault::revoke_issuer(&e, &owner, &issuer_addr);
        storage::extend_vault_ttl(&e, &owner);
        events::issuer_revoked(&e, &owner, &issuer_addr);
    }

    /// Revoke vault. Blocks all writes. Vault admin only.
    fn revoke_vault(e: Env, owner: Address) {
        validate_vault_admin(&e, &owner);
        validate_vault_active(&e, &owner);
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
    ) -> Option<crate::model::VerifiableCredential> {
        storage::extend_vault_ttl(&e, &owner);
        let vc = storage::read_vault_vc(&e, &owner, &vc_id);
        if vc.is_some() {
            storage::extend_vc_ttl(&e, &owner, &vc_id);
        }
        vc
    }

    /// Verify VC status. Returns VCStatus::Valid, VCStatus::Revoked(date), or VCStatus::Invalid.
    fn verify_vc(e: Env, owner: Address, vc_id: String) -> VCStatus {
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
        validate_vault_active(&e, &from_owner);
        validate_vault_active(&e, &to_owner);
        from_owner.require_auth();
        validate_issuer_authorized_only(&e, &from_owner, &issuer_addr);

        let vc_opt = storage::read_vault_vc(&e, &from_owner, &vc_id);
        if vc_opt.is_none() {
            panic_with_error!(e, ContractError::VCNotFound);
        }
        // Only Valid VCs may be pushed. A revoked VC cannot be transferred to
        // another vault; use the dedicated VCAlreadyRevoked error so callers
        // can distinguish "not found" from "found but revoked".
        if storage::read_vc_status(&e, &from_owner, &vc_id) != VCStatus::Valid {
            panic_with_error!(e, ContractError::VCAlreadyRevoked);
        }
        if storage::read_vault_vc(&e, &to_owner, &vc_id).is_some()
            || storage::read_vc_status(&e, &to_owner, &vc_id) != VCStatus::Invalid
        {
            panic_with_error!(e, ContractError::VCAlreadyExists);
        }
        let vc = vc_opt.unwrap();

        // Move the parent link with the VC so `get_vc_parent(to_owner, vc_id)`
        // resolves correctly post-push and the source vault stops claiming a
        // parent for a payload it no longer holds.
        let parent = storage::read_vc_parent(&e, &from_owner, &vc_id);

        storage::remove_vault_vc(&e, &from_owner, &vc_id);
        storage::remove_vc_from_index(&e, &from_owner, &vc_id);
        if parent.is_some() {
            storage::remove_vc_parent(&e, &from_owner, &vc_id);
        }
        // VCStatus(from_owner, vc_id) intentionally stays Valid as a tombstone
        // marker. It preserves vc_id uniqueness within the source vault — a
        // future `issue(from_owner, vc_id, ...)` panics with VCAlreadyExists
        // because the second check below trips on the stale status. Code paths
        // that need to know whether the payload still exists at the source
        // (verify_vc, revoke, issue_linked) check the payload directly so this
        // tombstone never causes a false-positive validation.

        storage::write_vault_vc(&e, &to_owner, &vc_id, &vc);
        storage::append_vc_to_index(&e, &to_owner, &vc_id);
        storage::write_vc_status(&e, &to_owner, &vc_id, &VCStatus::Valid);
        if let Some((parent_owner, parent_vc_id)) = parent {
            storage::write_vc_parent(&e, &to_owner, &vc_id, &parent_owner, &parent_vc_id);
        }

        storage::extend_vault_ttl(&e, &from_owner);
        storage::extend_vault_ttl(&e, &to_owner);
        storage::extend_vc_ttl(&e, &to_owner, &vc_id);
        events::vc_pushed(&e, &from_owner, &to_owner, &vc_id);
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
        issuer_addr.require_auth();
        let this = e.current_contract_address();
        if vault_contract != this {
            panic_with_error!(e, ContractError::InvalidVaultContract);
        }
        validate_vault_active(&e, &owner);
        ensure_issuer_authorized(&e, &owner, &issuer_addr);

        if storage::read_vault_vc(&e, &owner, &vc_id).is_some()
            || storage::read_vc_status(&e, &owner, &vc_id) != VCStatus::Invalid
        {
            panic_with_error!(e, ContractError::VCAlreadyExists);
        }

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

        storage::write_vc_status(&e, &owner, &vc_id, &VCStatus::Valid);
        storage::extend_vault_ttl(&e, &owner);
        storage::extend_vc_ttl(&e, &owner, &vc_id);
        events::vc_issued(&e, &owner, &vc_id, &issuer_addr);

        vc_id
    }

    /// Revoke VC. Owner must sign. The VC payload remains queryable via
    /// `get_vc(owner, vc_id)`; only the active index entry is removed so the
    /// vault doesn't fill up with revoked entries (each free slot can be
    /// reissued under a new vc_id, preserving the `MAX_VCS_PER_VAULT` cap as
    /// a *concurrent active* limit).
    fn revoke(e: Env, owner: Address, vc_id: String, date: String) {
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
        issuance::revoke_vc(&e, &owner, vc_id.clone(), date.clone());
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
        issuer.require_auth();
        let this = e.current_contract_address();
        if issuance_contract != this {
            panic_with_error!(e, ContractError::InvalidVaultContract);
        }
        validate_vault_active(&e, &owner);
        validate_vault_initialized(&e, &parent_owner);

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

        store_vc_payload(&e, &owner, vc_id.clone(), data, &issuer, issuer_did, this, 0);

        storage::write_vc_status(&e, &owner, &vc_id, &VCStatus::Valid);
        storage::write_vc_parent(&e, &owner, &vc_id, &parent_owner, &parent_vc_id);
        storage::extend_vault_ttl(&e, &owner);
        storage::extend_vc_ttl(&e, &owner, &vc_id);
        events::linked_vc_issued(&e, &issuer, &owner, &vc_id, &parent_owner, &parent_vc_id);
    }

    /// Returns Some((parent_owner, parent_vc_id)) if the VC was issued via issue_linked,
    /// or None if it is a regular VC with no parent link.
    fn get_vc_parent(e: Env, owner: Address, vc_id: String) -> Option<(Address, String)> {
        storage::extend_instance_ttl(&e);
        storage::read_vc_parent(&e, &owner, &vc_id)
    }

    // --- Sponsored vault ---

    /// Creates a vault on behalf of owner; sponsor must sign and be authorized unless open_to_all is enabled.
    fn create_sponsored_vault(e: Env, sponsor: Address, owner: Address, did_uri: String) {
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
            panic_with_error!(e, ContractError::AlreadyInitialized);
        }
        storage::write_vault_admin(&e, &owner, &owner);
        storage::write_vault_did(&e, &owner, &did_uri);
        storage::write_vault_revoked(&e, &owner, &false);
        storage::write_vault_issuers(&e, &owner, &Vec::new(&e));
        storage::extend_vault_ttl(&e, &owner);
        storage::extend_instance_ttl(&e);
        events::sponsored_vault_created(&e, &sponsor, &owner, &did_uri);
    }

    /// Sets whether sponsored vault creation is restricted to authorized sponsors or open to all. Admin only.
    fn set_sponsored_vault_open_to_all(e: Env, open: bool) {
        validate_contract_admin(&e);
        storage::write_sponsored_vault_open_to_all(&e, &open);
        storage::extend_instance_ttl(&e);
    }

    /// Query whether sponsored vault creation is open to all.
    fn get_sponsored_vault_open_to_all(e: Env) -> bool {
        storage::extend_instance_ttl(&e);
        storage::read_sponsored_vault_open_to_all(&e)
    }

    /// Add an address to the authorized sponsors list. Admin only.
    fn add_sponsored_vault_sponsor(e: Env, sponsor: Address) {
        validate_contract_admin(&e);
        storage::add_sponsored_vault_sponsor(&e, &sponsor);
        storage::extend_instance_ttl(&e);
    }

    /// Remove an address from the authorized sponsors list. Admin only.
    fn remove_sponsored_vault_sponsor(e: Env, sponsor: Address) {
        validate_contract_admin(&e);
        storage::remove_sponsored_vault_sponsor(&e, &sponsor);
        storage::extend_instance_ttl(&e);
    }

    // --- Migrations ---

    /// Migrate legacy vault VCs from old storage format to current format. Vault admin must sign.
    fn migrate(e: Env, owner: Address) {
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

/// Auto-authorizes issuer if not present in vault's list; panics if issuer is in the denied list.
fn ensure_issuer_authorized(e: &Env, owner: &Address, issuer_addr: &Address) {
    validate_vault_initialized(e, owner);
    let issuers = storage::read_vault_issuers(e, owner);
    if !vault::is_authorized(&issuers, issuer_addr) {
        if storage::is_issuer_denied(e, owner, issuer_addr) {
            panic_with_error!(e, ContractError::IssuerNotAuthorized)
        }
        vault::authorize_issuer(e, owner, issuer_addr);
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
