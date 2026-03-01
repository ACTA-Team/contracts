//! Shared helpers for all fuzz targets.

use soroban_sdk::{testutils::Address as _, Address, Env, String as SStr};
use vc_vault_contract::contract::{VcVaultContract, VcVaultContractClient};

/// Truncate a Rust string to a safe length and convert to a Soroban String.
/// Soroban Strings are limited; 256 bytes is well within bounds.
pub fn s(env: &Env, input: &str) -> SStr {
    let safe = &input[..input.len().min(256)];
    SStr::from_str(env, safe)
}

/// Create a fresh environment with all auths mocked, register the contract,
/// initialize it, and return (env, contract_id, admin, owner, issuer, client).
pub fn setup(
    env: &Env,
) -> (Address, Address, Address, Address, VcVaultContractClient<'_>) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let issuer = Address::generate(env);
    let owner = Address::generate(env);
    let cid = env.register(VcVaultContract, ());
    let client = VcVaultContractClient::new(env, &cid);
    client.initialize(&admin);
    client.create_vault(&owner, &s(env, "did:fuzz:owner"));
    client.authorize_issuer(&owner, &issuer);
    (admin, issuer, owner, cid, client)
}
