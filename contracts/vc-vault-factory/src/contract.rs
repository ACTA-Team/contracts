use soroban_sdk::{
    contract, contractclient, contractimpl, Address, Bytes, BytesN, Env, IntoVal, String,
};

use crate::{events, storage};
pub use crate::storage::VaultInitMeta;

#[contract]
pub struct VaultFactoryContract;

#[contractclient(name = "VaultFactoryClient")]
pub trait VaultFactory {
    /// Deploy a new single-tenant vault for `owner` and register it in the factory.
    ///
    /// `user_salt` is mixed with `owner` via keccak256 so each (owner, salt) pair
    /// produces a unique deterministic address and frontrunning is prevented.
    fn deploy(e: Env, owner: Address, did_uri: String, user_salt: BytesN<32>) -> Address;

    /// Deploy a vault on behalf of `owner`. `deployer` signs and pays; the vault
    /// belongs to `owner` from creation. Anyone can be a deployer — no whitelist.
    fn deploy_sponsored(e: Env, deployer: Address, owner: Address, did_uri: String, user_salt: BytesN<32>) -> Address;

    /// Return true if `vault_address` was deployed by this factory.
    fn is_vault(e: Env, vault_address: Address) -> bool;
}

#[contractimpl]
impl VaultFactoryContract {
    pub fn __constructor(e: Env, vault_init_meta: VaultInitMeta) {
        storage::set_vault_init_meta(&e, &vault_init_meta);
    }
}

fn derive_salt(e: &Env, user_salt: BytesN<32>, owner: &Address) -> BytesN<32> {
    // Salt = keccak256(user_salt || owner_bytes) — prevents frontrunning.
    let mut owner_bytes: [u8; 56] = [0; 56];
    owner.to_string().copy_into_slice(&mut owner_bytes);
    let mut salt_bytes: Bytes = user_salt.into_val(e);
    salt_bytes.extend_from_array(&owner_bytes);
    e.crypto().keccak256(&salt_bytes).into()
}

fn deploy_vault(e: &Env, owner: &Address, did_uri: String, user_salt: BytesN<32>) -> Address {
    let meta = storage::get_vault_init_meta(e);
    let new_salt = derive_salt(e, user_salt, owner);
    let vault_address = e
        .deployer()
        .with_current_contract(new_salt)
        .deploy_v2(meta.vault_hash, (owner.clone(), meta.contract_admin, did_uri));
    storage::set_deployed(e, &vault_address);
    vault_address
}

#[contractimpl]
impl VaultFactory for VaultFactoryContract {
    fn deploy(e: Env, owner: Address, did_uri: String, user_salt: BytesN<32>) -> Address {
        owner.require_auth();
        storage::extend_instance(&e);
        let vault_address = deploy_vault(&e, &owner, did_uri, user_salt);
        events::vault_deployed(&e, &owner, &vault_address);
        vault_address
    }

    fn deploy_sponsored(e: Env, deployer: Address, owner: Address, did_uri: String, user_salt: BytesN<32>) -> Address {
        deployer.require_auth();
        storage::extend_instance(&e);
        let vault_address = deploy_vault(&e, &owner, did_uri, user_salt);
        events::sponsored_vault_deployed(&e, &deployer, &owner, &vault_address);
        vault_address
    }

    fn is_vault(e: Env, vault_address: Address) -> bool {
        storage::extend_instance(&e);
        storage::is_deployed(&e, &vault_address)
    }
}
