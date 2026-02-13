use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::types::{LifeModel, VerificationRecord};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Oracle,
    UserModel(Address),
    LastScore(Address),
    Verifications(Address),
    LinkedVault(Address),
    LinkedVaultId(Address),
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

pub fn get_oracle(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Oracle).unwrap()
}

pub fn set_oracle(env: &Env, oracle: &Address) {
    env.storage().instance().set(&DataKey::Oracle, oracle);
}

pub fn get_model(env: &Env, user: &Address) -> Option<LifeModel> {
    env.storage().persistent().get(&DataKey::UserModel(user.clone()))
}

pub fn set_model(env: &Env, user: &Address, model: &LifeModel) {
    env.storage().persistent().set(&DataKey::UserModel(user.clone()), model);
}

pub fn get_last_score(env: &Env, user: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::LastScore(user.clone()))
        .unwrap_or(10_000)
}

pub fn set_last_score(env: &Env, user: &Address, score: u32) {
    env.storage().persistent().set(&DataKey::LastScore(user.clone()), &score);
}

pub fn get_verifications(env: &Env, user: &Address) -> Vec<VerificationRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Verifications(user.clone()))
        .unwrap_or(Vec::new(env))
}

pub fn set_verifications(env: &Env, user: &Address, records: &Vec<VerificationRecord>) {
    env.storage()
        .persistent()
        .set(&DataKey::Verifications(user.clone()), records);
}

pub fn get_linked_vault(env: &Env, user: &Address) -> Option<Address> {
    env.storage().persistent().get(&DataKey::LinkedVault(user.clone()))
}

pub fn set_linked_vault(env: &Env, user: &Address, vault_contract: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::LinkedVault(user.clone()), vault_contract);
}

pub fn get_linked_vault_id(env: &Env, user: &Address) -> Option<u64> {
    env.storage().persistent().get(&DataKey::LinkedVaultId(user.clone()))
}

pub fn set_linked_vault_id(env: &Env, user: &Address, vault_id: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::LinkedVaultId(user.clone()), &vault_id);
}
