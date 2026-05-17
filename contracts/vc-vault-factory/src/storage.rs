use soroban_sdk::{contracttype, Address, BytesN, Env, Symbol};

const ONE_DAY_LEDGERS: u32 = 17_280; // ~5s per ledger

const LEDGER_THRESHOLD_INSTANCE: u32 = ONE_DAY_LEDGERS * 30;
const LEDGER_BUMP_INSTANCE: u32 = LEDGER_THRESHOLD_INSTANCE + ONE_DAY_LEDGERS;

const LEDGER_THRESHOLD_CONTRACTS: u32 = ONE_DAY_LEDGERS * 100;
const LEDGER_BUMP_CONTRACTS: u32 = LEDGER_THRESHOLD_CONTRACTS + ONE_DAY_LEDGERS * 20;

#[derive(Clone)]
#[contracttype]
pub enum VaultFactoryDataKey {
    Contracts(Address),
}

#[derive(Clone)]
#[contracttype]
pub struct VaultInitMeta {
    pub vault_hash: BytesN<32>,
    pub contract_admin: Address,
}

pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_INSTANCE, LEDGER_BUMP_INSTANCE);
}

pub fn set_vault_init_meta(e: &Env, meta: &VaultInitMeta) {
    e.storage()
        .instance()
        .set::<Symbol, VaultInitMeta>(&Symbol::new(e, "VaultMeta"), meta);
}

pub fn get_vault_init_meta(e: &Env) -> VaultInitMeta {
    e.storage()
        .instance()
        .get::<Symbol, VaultInitMeta>(&Symbol::new(e, "VaultMeta"))
        .unwrap()
}

pub fn set_deployed(e: &Env, vault_address: &Address) {
    let key = VaultFactoryDataKey::Contracts(vault_address.clone());
    e.storage()
        .persistent()
        .set::<VaultFactoryDataKey, bool>(&key, &true);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD_CONTRACTS, LEDGER_BUMP_CONTRACTS);
}

pub fn is_deployed(e: &Env, vault_address: &Address) -> bool {
    let key = VaultFactoryDataKey::Contracts(vault_address.clone());
    if let Some(result) = e
        .storage()
        .persistent()
        .get::<VaultFactoryDataKey, bool>(&key)
    {
        e.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD_CONTRACTS, LEDGER_BUMP_CONTRACTS);
        result
    } else {
        false
    }
}
