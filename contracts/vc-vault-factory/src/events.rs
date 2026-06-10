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

#[contractevent]
pub struct AdminNominated {
    pub current: Address,
    pub nominee: Address,
}

#[contractevent]
pub struct AdminTransferred {
    pub old_admin: Address,
    pub new_admin: Address,
}

pub fn admin_nominated(e: &Env, current: &Address, nominee: &Address) {
    AdminNominated { current: current.clone(), nominee: nominee.clone() }.publish(e);
}

pub fn admin_transferred(e: &Env, old_admin: &Address, new_admin: &Address) {
    AdminTransferred { old_admin: old_admin.clone(), new_admin: new_admin.clone() }.publish(e);
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
