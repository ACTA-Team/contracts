use soroban_sdk::{map, Address, Env};

use crate::storage_types::{DataKey, Identity};

pub fn read_identity(env: &Env, id: Address) -> Identity {
    let key = DataKey::Identity(id.clone());

    return env
        .storage()
        .persistent().get(&key)
        .unwrap_or(Identity {
            owner: id.clone(),
            delegates: map![&env],
            attributes: map![&env],
        })
}

pub fn transfer_identity_ownership(env: &Env, id: Address, to: Address) -> Address {
    let key = DataKey::Identity(id);

    env.storage().persistent().set(
        &key,
        &Identity {
            owner: to.clone(),
            delegates: map![&env],
            attributes: map![&env],
        },
    );

    return to;
}
