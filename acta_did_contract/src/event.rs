use soroban_sdk::{vec, Address, Env, String, Symbol};

pub(crate) fn transfer_ownership_event(env: &Env, from: Address, to: Address) {
    let topics: (Symbol, Address, Address) = (Symbol::new(env, "transfer_ownership"), from, to);
    env.events().publish(topics, true)
}

pub(crate) fn set_delegate_event(
    env: &Env,
    user: Address,
    delegate: Address,
    delegate_type: Symbol,
    validity: u64,
) {
    let topics: (Symbol, Address, Address) = (Symbol::new(&env, "add_delegate"), user, delegate);
    env.events()
        .publish(topics, vec![&env, (delegate_type, validity)]);
}

pub(crate) fn remove_delegate_event(env: &Env, user: Address, delegate: Address) {
    let topics: (Symbol, Address, Address) = (Symbol::new(&env, "remove_delegate"), user, delegate);
    env.events().publish(topics, vec![&env, false]);
}

pub(crate) fn set_attribute_event(
    env: &Env,
    user: Address,
    name: String,
    value: String,
    validity: u64,
) {
    let topics: (Symbol, Address, String) = (Symbol::new(&env, "set_attribute"), user, name);
    env.events().publish(topics, vec![&env, (value, validity)]);
}

pub(crate) fn remove_attribute_event(env: &Env, user: Address, name: String) {
    let topics: (Symbol, Address, String) = (Symbol::new(&env, "modify_attribute"), user, name);
    env.events().publish(topics, vec![&env, false]);
}
