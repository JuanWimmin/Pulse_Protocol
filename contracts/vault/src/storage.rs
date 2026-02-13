use soroban_sdk::{contracttype, Address, Env};

use crate::types::{VaultId, VaultInfo, Beneficiary};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    VaultCount,
    Vault(VaultId),
    VaultBeneficiaries(VaultId),
    ProofOfLifeLink(VaultId),
    Initialized,
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Initialized)
}

pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&DataKey::Initialized, &true);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_vault_count(env: &Env) -> VaultId {
    env.storage().instance().get(&DataKey::VaultCount).unwrap_or(0)
}

pub fn set_vault_count(env: &Env, count: VaultId) {
    env.storage().instance().set(&DataKey::VaultCount, &count);
}

pub fn get_vault(env: &Env, vault_id: VaultId) -> Option<VaultInfo> {
    env.storage().persistent().get(&DataKey::Vault(vault_id))
}

pub fn set_vault(env: &Env, vault_id: VaultId, vault: &VaultInfo) {
    env.storage().persistent().set(&DataKey::Vault(vault_id), vault);
}

pub fn get_beneficiaries(env: &Env, vault_id: VaultId) -> soroban_sdk::Vec<Beneficiary> {
    env.storage()
        .persistent()
        .get(&DataKey::VaultBeneficiaries(vault_id))
        .unwrap_or(soroban_sdk::Vec::new(env))
}

pub fn set_beneficiaries(env: &Env, vault_id: VaultId, beneficiaries: &soroban_sdk::Vec<Beneficiary>) {
    env.storage()
        .persistent()
        .set(&DataKey::VaultBeneficiaries(vault_id), beneficiaries);
}

pub fn get_pol_link(env: &Env, vault_id: VaultId) -> Option<Address> {
    env.storage().persistent().get(&DataKey::ProofOfLifeLink(vault_id))
}

pub fn set_pol_link(env: &Env, vault_id: VaultId, pol_contract: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::ProofOfLifeLink(vault_id), pol_contract);
}
