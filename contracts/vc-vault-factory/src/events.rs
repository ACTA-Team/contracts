use soroban_sdk::{contractevent, Address, Env};

#[contractevent]
pub struct VaultDeployed {
    pub owner: Address,
    pub vault_address: Address,
}

#[contractevent]
pub struct SponsoredVaultDeployed {
    pub deployer: Address,
    pub owner: Address,
    pub vault_address: Address,
}

pub fn vault_deployed(e: &Env, owner: &Address, vault_address: &Address) {
    VaultDeployed {
        owner: owner.clone(),
        vault_address: vault_address.clone(),
    }
    .publish(e);
}

pub fn sponsored_vault_deployed(e: &Env, deployer: &Address, owner: &Address, vault_address: &Address) {
    SponsoredVaultDeployed {
        deployer: deployer.clone(),
        owner: owner.clone(),
        vault_address: vault_address.clone(),
    }
    .publish(e);
}
