use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::types::Beneficiary;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    VaultContract,
    VaultBeneficiaries(u64),
    ClaimRecord(u64, Address),
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

pub fn get_vault_contract(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::VaultContract)
}

pub fn set_vault_contract(env: &Env, vault_contract: &Address) {
    env.storage().instance().set(&DataKey::VaultContract, vault_contract);
}

pub fn get_beneficiaries(env: &Env, vault_id: u64) -> Vec<Beneficiary> {
    env.storage()
        .persistent()
        .get(&DataKey::VaultBeneficiaries(vault_id))
        .unwrap_or(Vec::new(env))
}

pub fn set_beneficiaries(env: &Env, vault_id: u64, beneficiaries: &Vec<Beneficiary>) {
    env.storage()
        .persistent()
        .set(&DataKey::VaultBeneficiaries(vault_id), beneficiaries);
}

pub fn has_claimed(env: &Env, vault_id: u64, claimer: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::ClaimRecord(vault_id, claimer.clone()))
}

pub fn set_claimed(env: &Env, vault_id: u64, claimer: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::ClaimRecord(vault_id, claimer.clone()), &true);
}
