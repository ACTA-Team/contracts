use soroban_sdk::{
    contract, contractclient, contractimpl, xdr::ToXdr, Address, Bytes, BytesN, Env, IntoVal,
    String,
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
    // deploy_salt = keccak256( user_salt(32 bytes) || XDR(owner) )
    //
    // Mixing the owner into the salt binds the deterministic vault address to a
    // specific owner (so a sponsored deploy lands on the same address the owner
    // would compute, and two owners can't collide on one address). The owner is
    // serialized via its canonical XDR form rather than to_string(): XDR is the
    // stable wire encoding an off-chain client reproduces directly, and it
    // matches the preimage documented in the README. to_string() is a display
    // (StrKey) encoding and ties determinism to a fixed 56-byte assumption.
    let mut preimage: Bytes = user_salt.into_val(e);
    preimage.append(&owner.clone().to_xdr(e));
    e.crypto().keccak256(&preimage).into()
}

fn deploy_vault(e: &Env, owner: &Address, did_uri: String, user_salt: BytesN<32>) -> Address {
    let meta = storage::get_vault_init_meta(e);
    let new_salt = derive_salt(e, user_salt, owner);
    let factory_address = e.current_contract_address();
    let vault_address = e
        .deployer()
        .with_current_contract(new_salt)
        .deploy_v2(meta.vault_hash, (owner.clone(), meta.contract_admin, did_uri, factory_address));
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
