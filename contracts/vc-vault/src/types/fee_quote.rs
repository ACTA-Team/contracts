//! Mirror of the factory's FeeQuote, used to decode the cross-contract
//! `quote_fee` response. Field names and types MUST match
//! `vc-vault-factory::storage::FeeQuote` exactly for ScVal decoding.

use soroban_sdk::{contracttype, Address};

#[derive(Clone)]
#[contracttype]
pub struct FeeQuote {
    pub enabled: bool,
    pub amount: i128,
    pub token: Option<Address>,
    pub dest: Option<Address>,
}
