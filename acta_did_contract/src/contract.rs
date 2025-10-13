use crate::attribute::{remove_attribute, set_attribute};
use crate::delegate::{remove_delegate, set_delegate};
use crate::event::{
    remove_attribute_event, remove_delegate_event, set_attribute_event, set_delegate_event,
    transfer_ownership_event,
};
use crate::identity::{read_identity, transfer_identity_ownership};
use crate::storage_types::{Error, Identity};
use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, String, Symbol};

#[contract]
pub struct DIDStellarRegistry;

contractmeta!(
    key = "Description",
    val = "DID method registry for the Stellar did method"
);

#[contractimpl]
impl DIDStellarRegistry {

    pub fn identity(env: Env, user: Address) -> Identity {
        return read_identity(&env, user);
    }

    // transfers ownership of Id from one user to another

    pub fn transfer(
        env: Env,
        id: Address,
        from: Address,
        to: Address,
    ) -> Result<(), Error> {
        from.require_auth();
        let identity = read_identity(&env, id.clone());

        if identity.owner != from {
            return Err(Error::NotAuthorized);
        }

        transfer_identity_ownership(&env, id.clone(), to.clone());
        transfer_ownership_event(&env, id, to);
        return Ok(());
    }

    pub fn delegate(
        env: Env,
        user: Address,
        delegate: Address,
        delegate_type: Symbol,
        valid_to: u64,
    ) {
        let identity = read_identity(&env, user.clone());
        identity.owner.require_auth();
        set_delegate(
            &env,
            user.clone(),
            delegate.clone(),
            delegate_type.clone(),
            valid_to.clone(),
        );
        set_delegate_event(&env, user, delegate, delegate_type, valid_to);
    }

    pub fn remove_delegate(env: Env, user: Address, delegate: Address) {
        let identity = read_identity(&env, user.clone());
        identity.owner.require_auth();
        remove_delegate(&env, user.clone(), delegate.clone());
        remove_delegate_event(&env, user, delegate);
    }

    pub fn set_attribute(
        env: Env,
        user: Address,
        name: String,
        value: String,
        valid_to: u64,
    ) -> Result<(), Error> {
        let identity = read_identity(&env, user.clone());
        identity.owner.require_auth();
        let result = set_attribute(
            &env,
            user.clone(),
            name.clone(),
            value.clone(),
            valid_to.clone(),
        );

        set_attribute_event(&env, user, name, value, valid_to);

        result
    }

    pub fn remove_attribute(env: Env, user: Address, name: String) -> Result<(), Error> {
        let identity = read_identity(&env, user.clone());
        identity.owner.require_auth();
        let result = remove_attribute(&env, user.clone(), name.clone());
        remove_attribute_event(&env, user, name);

        result
    }
}
