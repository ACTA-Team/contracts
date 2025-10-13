#![cfg(test)]
extern crate std;
use crate::storage_types::{Attribute, Delegation, Identity};
use crate::{contract::DIDStellarRegistry, contract::DIDStellarRegistryClient};

use soroban_sdk::{
    log, map, symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, String, Symbol,
};

#[cfg(test)]
mod tests {

    use soroban_sdk::testutils::Logs;

    use super::*;

    #[test]
    fn get_default_identity() {
        let env = Env::default();
        let contract_id = env.register_contract(None, DIDStellarRegistry);

        let client = DIDStellarRegistryClient::new(&env, &contract_id);

        let user = Address::random(&env);

        let result = client.identity(&user);

        let logs = env.logs().all();

        std::println!("{}", logs.join("\n"));

        assert_eq!(
            result,
            Identity {
                owner: user,
                delegates: map![&env],
                attributes: map![&env],
            }
        );
    }

    #[test]
    fn change_owner() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, DIDStellarRegistry);

        let client = DIDStellarRegistryClient::new(&env, &contract_id);
        let id = Address::random(&env);
        let to = Address::random(&env);

        log!(&env, "Hello {}", id);

        client.transfer(&id, &id, &to);

        let result = client.identity(&id);

        assert_eq!(
            result,
            Identity {
                owner: to,
                delegates: map![&env],
                attributes: map![&env],
            }
        );
    }

    #[test]
    fn change_owner_auth() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, DIDStellarRegistry);
        let client = DIDStellarRegistryClient::new(&env, &contract_id);
        let id = Address::random(&env);
        let first_address = Address::random(&env);
        let second_address = Address::random(&env);

        client.transfer(&id, &id, &first_address);

        assert_eq!(
            env.auths(),
            [(
                // The owner of the identity is the only one who can transfer ownership
                id.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        contract_id.clone(),
                        symbol_short!("transfer"),
                        (id.clone(), id.clone(), first_address.clone()).into_val(&env)
                    )),
                    sub_invocations: std::vec![]
                }
            )]
        );
        let result = client.identity(&id);

        assert_eq!(
            result,
            Identity {
                owner: first_address.clone(),
                delegates: map![&env],
                attributes: map![&env],
            }
        );

        client.transfer(&id, &first_address, &second_address);

        assert_eq!(
            env.auths(),
            [(
                first_address.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        contract_id.clone(),
                        symbol_short!("transfer"),
                        (id.clone(), first_address.clone(), second_address.clone()).into_val(&env)
                    )),
                    sub_invocations: std::vec![]
                }
            )]
        );

        let result = client.identity(&id);

        assert_eq!(
            result,
            Identity {
                owner: second_address.clone(),
                delegates: map![&env],
                attributes: map![&env],
            }
        );
    }
    // Delegates do not affect the identity owner, they are only used for the DIDDocument Delegates fields
    #[test]
    fn add_delegate() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, DIDStellarRegistry);

        let client = DIDStellarRegistryClient::new(&env, &contract_id);
        let user = Address::random(&env);
        let delegate = Address::random(&env);

        client.delegate(&user, &delegate, &Symbol::new(&env, "test"), &200);

        let result = client.identity(&user);
        assert_eq!(
            result,
            Identity {
                owner: user,
                delegates: map![
                    &env,
                    (
                        delegate,
                        Delegation {
                            delegate_type: Symbol::new(&env, "test"),
                            valid_to: 200,
                        }
                    )
                ],
                attributes: map![&env],
            }
        );
    }

    #[test]
    fn remove_delegate() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, DIDStellarRegistry);

        let client = DIDStellarRegistryClient::new(&env, &contract_id);
        let user = Address::random(&env);
        let delegate = Address::random(&env);

        client.delegate(&user, &delegate, &Symbol::new(&env, "test"), &200);

        let result = client.identity(&user);
        assert_eq!(
            result,
            Identity {
                owner: user.clone(),
                delegates: map![
                    &env,
                    (
                        delegate.clone(),
                        Delegation {
                            delegate_type: Symbol::new(&env, "test"),
                            valid_to: 200,
                        }
                    )
                ],
                attributes: map![&env],
            }
        );

        client.remove_delegate(&user, &delegate);

        let result = client.identity(&user);

        assert_eq!(
            result,
            Identity {
                owner: user.clone(),
                delegates: map![&env],
                attributes: map![&env],
            }
        );
    }

    #[test]
    fn set_attribute() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, DIDStellarRegistry);

        let client = DIDStellarRegistryClient::new(&env, &contract_id);
        let user = Address::random(&env);
        let name = String::from_slice(&env, "test_name");
        let attribute = Attribute {
            value: String::from_slice(&env, "test_value"),
            valid_to: 200,
        };

        client.set_attribute(&user, &name, &attribute.value, &attribute.valid_to);

        let result = client.identity(&user);
        assert_eq!(
            result,
            Identity {
                owner: user,
                delegates: map![&env],
                attributes: map![&env, (name, attribute)],
            }
        );
    }

    #[test]
    fn modify_attribute() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, DIDStellarRegistry);

        let client = DIDStellarRegistryClient::new(&env, &contract_id);
        let user = Address::random(&env);
        let name = String::from_slice(&env, "test_name");
        let attribute = Attribute {
            value: String::from_slice(&env, "test_value"),
            valid_to: 200,
        };

        client.set_attribute(&user, &name, &attribute.value, &attribute.valid_to);
        let result = client.identity(&user);

        assert_eq!(
            result,
            Identity {
                owner: user.clone(),
                delegates: map![&env],
                attributes: map![&env, (name.clone(), attribute.clone())],
            }
        );
        client.set_attribute(&user, &name, &attribute.value, &300);

        let result = client.identity(&user);

        assert_eq!(
            result,
            Identity {
                owner: user.clone(),
                delegates: map![&env],
                attributes: map![
                    &env,
                    (
                        name.clone(),
                        Attribute {
                            value: attribute.value,
                            valid_to: 300,
                        }
                    )
                ],
            }
        );
    }

    #[test]
    fn remove_attribute() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, DIDStellarRegistry);

        let client = DIDStellarRegistryClient::new(&env, &contract_id);
        let user = Address::random(&env);
        let name = String::from_slice(&env, "test_name");
        let attribute = Attribute {
            value: String::from_slice(&env, "test_value"),
            valid_to: 200,
        };

        client.set_attribute(&user, &name, &attribute.value, &attribute.valid_to);

        let result = client.identity(&user);
        assert_eq!(
            result,
            Identity {
                owner: user.clone(),
                delegates: map![&env],
                attributes: map![&env, (name.clone(), attribute.clone())],
            }
        );

        client.remove_attribute(&user, &name);

        let result = client.identity(&user);
        assert_eq!(
            result,
            Identity {
                owner: user.clone(),
                delegates: map![&env],
                attributes: map![&env],
            }
        );
    }
}
