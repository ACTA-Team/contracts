use soroban_sdk::{Address, Env, String};

use crate::{
    identity::read_identity,
    storage_types::{Attribute, DataKey, Error},
};

pub fn set_attribute(
    env: &Env,
    id: Address,
    name: String,
    value: String,
    valid_to: u64,
) -> Result<(), Error> {
    let mut identity = read_identity(&env, id.clone());

    let attr: Attribute = Attribute {
        value: value.clone(),
        valid_to: valid_to.clone(),
    };

    // if an attribute with the same name exists then modify it, else add it

    identity.attributes.set(name, attr);
    env.storage().persistent().set(&DataKey::Identity(id), &identity);
    Ok(())
}

pub fn remove_attribute(env: &Env, id: Address, name: String) -> Result<(), Error> {
    let mut identity = read_identity(&env, id.clone());

    identity.attributes.remove(name);

    env.storage().persistent().set(&DataKey::Identity(id), &identity);
    return Ok(());
}
