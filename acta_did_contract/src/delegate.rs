use soroban_sdk::{Address, Env, Symbol};

use crate::{
    identity::read_identity,
    storage_types::{DataKey, Delegation},
};

pub fn set_delegate(
    env: &Env,
    id: Address,
    delegate: Address,
    delegate_type: Symbol,
    valid_to: u64,
) {
    let mut identity = read_identity(&env, id.clone());

    identity.delegates.set(
        delegate.clone(),
        Delegation {
            delegate_type: delegate_type.clone(),
            valid_to: valid_to,
        },
    );

    env.storage().persistent().set(&DataKey::Identity(id), &identity);
}

pub fn remove_delegate(env: &Env, id: Address, delegate: Address) {
    let mut identity = read_identity(&env, id.clone());

    identity.delegates.remove(delegate.clone());

    env.storage().persistent().set(&DataKey::Identity(id), &identity);
}
