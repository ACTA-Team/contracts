use soroban_sdk::{contracterror, contracttype, Address, Map, String, Symbol};

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub enum DataKey {
    Identity(Address),
}

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub struct Identity {
    pub owner: Address,
    pub delegates: Map<Address, Delegation>,
    pub attributes: Map<String, Attribute>,
}

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub struct Delegation {
    pub delegate_type: Symbol, // uint validTo:
    pub valid_to: u64,
}

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub value: String,
    pub valid_to: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    DelegateAlreadyExists = 2,
    AttributeNotFound = 3,
}
