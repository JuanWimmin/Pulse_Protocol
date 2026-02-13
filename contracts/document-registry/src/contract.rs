use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, String, Vec};

use crate::errors::DocumentError;
use crate::storage;
use crate::types::{DocId, DocumentAccess, DocumentInfo, DocumentProof, DocumentType};

#[contract]
pub struct DocumentRegistryContract;

#[allow(deprecated)]
#[contractimpl]
impl DocumentRegistryContract {
    /// Initialize the contract.
    pub fn initialize(env: Env, admin: Address) -> Result<(), DocumentError> {
        if storage::is_initialized(&env) {
            return Err(DocumentError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        storage::set_initialized(&env);
        Ok(())
    }

    /// Register a new document. Returns DocId.
    pub fn register_document(
        env: Env,
        owner: Address,
        ipfs_cid: String,
        doc_hash: BytesN<32>,
        doc_type: DocumentType,
        is_encrypted: bool,
    ) -> Result<DocId, DocumentError> {
        owner.require_auth();

        let doc_id = storage::get_doc_count(&env);
        let doc = DocumentInfo {
            id: doc_id,
            owner,
            ipfs_cid,
            doc_hash,
            doc_type,
            is_encrypted,
            registered_at: env.ledger().timestamp(),
            vault_id: 0,
            linked: false,
        };

        storage::set_document(&env, doc_id, &doc);
        storage::set_doc_count(&env, doc_id + 1);

        env.events()
            .publish((symbol_short!("doc"), symbol_short!("register")), doc_id);

        Ok(doc_id)
    }

    /// Link a document to a vault.
    pub fn link_to_vault(env: Env, doc_id: DocId, vault_id: u64) -> Result<(), DocumentError> {
        let mut doc = storage::get_document(&env, doc_id)
            .ok_or(DocumentError::DocumentNotFound)?;

        doc.owner.require_auth();

        doc.vault_id = vault_id;
        doc.linked = true;
        storage::set_document(&env, doc_id, &doc);
        storage::add_vault_document(&env, vault_id, doc_id);

        Ok(())
    }

    /// Store an encrypted AES key for a beneficiary.
    pub fn store_encrypted_key(
        env: Env,
        doc_id: DocId,
        beneficiary: Address,
        encrypted_key: Bytes,
    ) -> Result<(), DocumentError> {
        let doc = storage::get_document(&env, doc_id)
            .ok_or(DocumentError::DocumentNotFound)?;
        doc.owner.require_auth();
        storage::set_encrypted_key(&env, doc_id, &beneficiary, &encrypted_key);
        Ok(())
    }

    /// Grant access to a document for a beneficiary.
    pub fn grant_access(
        env: Env,
        doc_id: DocId,
        beneficiary: Address,
    ) -> Result<DocumentAccess, DocumentError> {
        let admin = storage::get_admin(&env);
        admin.require_auth();

        let doc = storage::get_document(&env, doc_id)
            .ok_or(DocumentError::DocumentNotFound)?;

        storage::grant_access(&env, doc_id, &beneficiary);

        let encrypted_key = storage::get_encrypted_key(&env, doc_id, &beneficiary)
            .unwrap_or(Bytes::new(&env));

        Ok(DocumentAccess {
            ipfs_cid: doc.ipfs_cid,
            encrypted_key,
            doc_type: doc.doc_type,
            is_encrypted: doc.is_encrypted,
        })
    }

    /// Verify document existence and integrity.
    pub fn verify_document(env: Env, doc_id: DocId) -> Result<DocumentProof, DocumentError> {
        let doc = storage::get_document(&env, doc_id)
            .ok_or(DocumentError::DocumentNotFound)?;

        Ok(DocumentProof {
            exists: true,
            doc_hash: doc.doc_hash,
            registered_at: doc.registered_at,
            ipfs_cid: doc.ipfs_cid,
        })
    }

    /// Get all document IDs for a vault.
    pub fn get_vault_documents(env: Env, vault_id: u64) -> Vec<DocId> {
        storage::get_vault_documents(&env, vault_id)
    }

    /// Get document info.
    pub fn get_document(env: Env, doc_id: DocId) -> Result<DocumentInfo, DocumentError> {
        storage::get_document(&env, doc_id).ok_or(DocumentError::DocumentNotFound)
    }
}
