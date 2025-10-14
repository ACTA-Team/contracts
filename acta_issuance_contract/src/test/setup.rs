use crate::contract::{VCIssuanceContract, VCIssuanceContractClient};
use soroban_sdk::{map, testutils::Address as _, Address, Env, FromVal, Map, String};

mod vault_contract {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/acta_vault_contract.wasm"
    );
}

pub struct VCIssuanceContractTest<'a> {
    pub env: Env,
    pub admin: Address,
    pub vc_id: String,
    pub vc_data: String,
    pub issuer_did: String,
    pub contract: VCIssuanceContractClient<'a>,
}

impl<'a> VCIssuanceContractTest<'a> {
    pub fn setup() -> Self {
        let env: Env = Default::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract =
            VCIssuanceContractClient::new(&env, &env.register_contract(None, VCIssuanceContract));
        let vc_id = String::from_str(&env, "iwvkdjquj3fscmafrgeeqblw");
        let vc_data = String::from_str(&env, "eoZXggNeVDW2g5GeA0G2s0QJBn3SZWzWSE3fXM9V6IB5wWIfFJRxPrTLQRMHulCF62bVQNmZkj7zbSa39fVjAUTtfm6JMio75uMxoDlAN/Y");
        let issuer_did = String::from_str(&env, "did:chaincerts:7dotwpyzo2weqj6oto6liic6");

        VCIssuanceContractTest {
            env,
            admin,
            vc_id,
            vc_data,
            issuer_did,
            contract,
        }
    }
}

pub fn create_vc(
    env: &Env,
    admin: &Address,
    contract: &VCIssuanceContractClient,
    issuer_did: &String,
) -> Address {
    let vault_admin = Address::generate(env);
    let vault_contract_address = env.register_contract_wasm(None, vault_contract::WASM);
    let vault_client = vault_contract::Client::new(env, &vault_contract_address);

    let did_uri = String::from_str(
        env,
        "did:pkh:stellar:testnet:GCUETKXJ2YNVADOF5SZBBZA6M3O6HEOXN4GRJZUW2MBRS2UKXZM37QDE",
    );
    vault_client.initialize(&vault_admin, &did_uri);
    vault_client.authorize_issuer(admin);

    contract.initialize(admin, issuer_did);
    vault_contract_address
}

pub fn get_revoked_vc_map(env: &Env, date: String) -> Map<String, String> {
    let status_str = String::from_str(env, "status");
    let since_str = String::from_str(env, "since");
    let revoked_str = String::from_str(env, "revoked");

    map![env, (status_str, revoked_str), (since_str, date)]
}

pub fn get_valid_vc_map(env: &Env) -> Map<String, String> {
    let status_str = String::from_str(env, "status");
    let valid_str = String::from_str(env, "valid");

    map![env, (status_str, valid_str)]
}

// Se elimina la construcción del DID Document on-chain: el vault acepta `did_uri` directamente.
