use soroban_sdk::{contracttype, Address, Bytes, Env, Vec};

use crate::types::{DocId, DocumentInfo};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    DocCount,
    Document(DocId),
    VaultDocuments(u64),
    EncryptedKey(DocId, Address),
    AccessGranted(DocId, Address),
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

pub fn get_doc_count(env: &Env) -> DocId {
    env.storage().instance().get(&DataKey::DocCount).unwrap_or(0)
}

pub fn set_doc_count(env: &Env, count: DocId) {
    env.storage().instance().set(&DataKey::DocCount, &count);
}

pub fn get_document(env: &Env, doc_id: DocId) -> Option<DocumentInfo> {
    env.storage().persistent().get(&DataKey::Document(doc_id))
}

pub fn set_document(env: &Env, doc_id: DocId, doc: &DocumentInfo) {
    env.storage().persistent().set(&DataKey::Document(doc_id), doc);
}

pub fn get_vault_documents(env: &Env, vault_id: u64) -> Vec<DocId> {
    env.storage()
        .persistent()
        .get(&DataKey::VaultDocuments(vault_id))
        .unwrap_or(Vec::new(env))
}

pub fn add_vault_document(env: &Env, vault_id: u64, doc_id: DocId) {
    let mut docs = get_vault_documents(env, vault_id);
    docs.push_back(doc_id);
    env.storage()
        .persistent()
        .set(&DataKey::VaultDocuments(vault_id), &docs);
}

pub fn get_encrypted_key(env: &Env, doc_id: DocId, beneficiary: &Address) -> Option<Bytes> {
    env.storage()
        .persistent()
        .get(&DataKey::EncryptedKey(doc_id, beneficiary.clone()))
}

pub fn set_encrypted_key(env: &Env, doc_id: DocId, beneficiary: &Address, key: &Bytes) {
    env.storage()
        .persistent()
        .set(&DataKey::EncryptedKey(doc_id, beneficiary.clone()), key);
}

pub fn is_access_granted(env: &Env, doc_id: DocId, beneficiary: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::AccessGranted(doc_id, beneficiary.clone()))
}

pub fn grant_access(env: &Env, doc_id: DocId, beneficiary: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::AccessGranted(doc_id, beneficiary.clone()), &true);
}
