use crate::error::ContractError;
use crate::issuer;
use crate::storage;
use crate::vault_trait::VaultTrait;
use crate::verifiable_credential;
use soroban_sdk::{
    contract, contractimpl, contractmeta, panic_with_error, Address, BytesN, Env, String, Vec,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

contractmeta!(
    key = "Description",
    val = "Smart contract for Chaincerts Vault",
);

#[allow(dead_code)]
#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultTrait for VaultContract {
    fn initialize(e: Env, owner: Address, did_uri: String) {
        if storage::has_admin(&e, &owner) {
            panic_with_error!(e, ContractError::AlreadyInitialized);
        }

        // Por defecto, el admin inicial es el owner
        storage::write_admin(&e, &owner, &owner);
        // Si no hay admin de contrato, establecemos el primero que inicializa
        if !storage::has_contract_admin(&e) {
            storage::write_contract_admin(&e, &owner);
        }
        storage::write_did(&e, &owner, &did_uri);
        storage::write_revoked(&e, &owner, &false);
        storage::write_issuers(&e, &owner, &Vec::new(&e));
    }

    fn authorize_issuers(e: Env, owner: Address, issuers: Vec<Address>) {
        validate_admin(&e, &owner);
        validate_vault_revoked(&e, &owner);

        issuer::authorize_issuers(&e, &owner, &issuers);
    }

    fn authorize_issuer(e: Env, owner: Address, issuer: Address) {
        validate_admin(&e, &owner);
        validate_vault_revoked(&e, &owner);

        issuer::authorize_issuer(&e, &owner, &issuer);
    }

    fn revoke_issuer(e: Env, owner: Address, issuer: Address) {
        validate_admin(&e, &owner);
        validate_vault_revoked(&e, &owner);

        issuer::revoke_issuer(&e, &owner, &issuer)
    }

    fn store_vc(
        e: Env,
        owner: Address,
        vc_id: String,
        vc_data: String,
        issuer: Address,
        issuer_did: String,
        issuance_contract: Address,
    ) {
        validate_vault_revoked(&e, &owner);
        validate_issuer(&e, &owner, &issuer);

        verifiable_credential::store_vc(&e, &owner, vc_id, vc_data, issuance_contract, issuer_did);
    }

    fn list_vc_ids(e: Env, owner: Address) -> Vec<String> {
        // Only the owner can list their credential IDs
        owner.require_auth();
        storage::read_vc_ids(&e, &owner)
    }

    fn get_vc(e: Env, owner: Address, vc_id: String) -> Option<verifiable_credential::VerifiableCredential> {
        // Only the owner can read their credential content
        owner.require_auth();
        storage::read_vc(&e, &owner, &vc_id)
    }

    fn revoke_vault(e: Env, owner: Address) {
        validate_admin(&e, &owner);
        validate_vault_revoked(&e, &owner);

        storage::write_revoked(&e, &owner, &true);
    }

    fn migrate(e: Env, owner: Address) {
        validate_admin(&e, &owner);

        let vcs = storage::read_old_vcs(&e, &owner);

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

        storage::remove_old_vcs(&e, &owner);
    }

    fn set_admin(e: Env, owner: Address, new_admin: Address) {
        validate_admin(&e, &owner);

        storage::write_admin(&e, &owner, &new_admin);
    }

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let admin = storage::read_contract_admin(&e);
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn version(e: Env) -> String {
        String::from_str(&e, VERSION)
    }
}

fn validate_admin(e: &Env, owner: &Address) {
    let contract_admin = storage::read_admin(e, owner);
    contract_admin.require_auth();
}

fn validate_issuer(e: &Env, owner: &Address, issuer: &Address) {
    let issuers: Vec<Address> = storage::read_issuers(e, owner);

    if !issuer::is_authorized(&issuers, issuer) {
        panic_with_error!(e, ContractError::IssuerNotAuthorized)
    }

    issuer.require_auth();
}

fn validate_vault_revoked(e: &Env, owner: &Address) {
    let vault_revoked: bool = storage::read_revoked(e, owner);
    if vault_revoked {
        panic_with_error!(e, ContractError::VaultRevoked)
    }
}

// DID generativo: ya no se despliega contrato DID ni se invoca.
