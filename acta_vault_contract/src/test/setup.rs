use crate::contract::{VaultContract, VaultContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

pub struct VaultContractTest<'a> {
    pub env: Env,
    pub admin: Address,
    pub issuer: Address,
    pub did_uri: String,
    pub contract: VaultContractClient<'a>,
}

impl<'a> VaultContractTest<'a> {
    pub fn setup() -> Self {
        let env: Env = Default::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let did_uri = String::from_str(
            &env,
            "did:pkh:stellar:testnet:GCUETKXJ2YNVADOF5SZBBZA6M3O6HEOXN4GRJZUW2MBRS2UKXZM37QDE",
        );

        let contract = VaultContractClient::new(&env, &env.register_contract(None, VaultContract));
        VaultContractTest {
            env,
            admin,
            issuer,
            did_uri,
            contract,
        }
    }
}

pub struct VCVaultContractTest {
    pub vc_id: String,
    pub vc_data: String,
    pub issuance_contract_address: Address,
    pub issuer_did: String,
}

pub fn get_vc_setup(env: &Env) -> VCVaultContractTest {
    let vc_id = String::from_str(env, "vc_id");
    let vc_data = String::from_str(env, "vc_data");
    let issuance_contract_address = Address::generate(env);
    let issuer_did = String::from_str(env, "did:pkh:stellar:testnet:GCUETKXJ2YNVADOF5SZBBZA6M3O6HEOXN4GRJZUW2MBRS2UKXZM37QDE");

    VCVaultContractTest {
        vc_id,
        vc_data,
        issuance_contract_address,
        issuer_did,
    }
}
