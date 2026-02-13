#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, String};
use contract::{DocumentRegistryContract, DocumentRegistryContractClient};
use types::DocumentType;

// ---------------------------------------------------------------------------
// Helper: set up env + initialized client
// ---------------------------------------------------------------------------
fn setup() -> (Env, DocumentRegistryContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(DocumentRegistryContract, ());
    let client = DocumentRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);

    client.initialize(&admin);

    (env, client, admin, owner)
}

// ---------------------------------------------------------------------------
// 1. test_initialize
// ---------------------------------------------------------------------------
#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(DocumentRegistryContract, ());
    let client = DocumentRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
}

// ---------------------------------------------------------------------------
// 2. test_initialize_twice_fails
// ---------------------------------------------------------------------------
#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    let contract_id = env.register(DocumentRegistryContract, ());
    let client = DocumentRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    let result = client.try_initialize(&admin);
    assert!(result.is_err() || result.unwrap().is_err());
}

// ---------------------------------------------------------------------------
// 3. test_register_document
// ---------------------------------------------------------------------------
#[test]
fn test_register_document() {
    let (env, client, _admin, owner) = setup();

    let cid = String::from_str(&env, "QmTest123");
    let hash = BytesN::from_array(&env, &[1u8; 32]);

    let doc_id = client.register_document(
        &owner,
        &cid,
        &hash,
        &DocumentType::PublicManifest,
        &false,
    );

    assert_eq!(doc_id, 0);

    let doc = client.get_document(&doc_id);
    assert_eq!(doc.owner, owner);
    assert_eq!(doc.is_encrypted, false);
}

// ---------------------------------------------------------------------------
// 4. test_register_multiple_documents
// ---------------------------------------------------------------------------
#[test]
fn test_register_multiple_documents() {
    let (env, client, _admin, owner) = setup();

    let cid1 = String::from_str(&env, "QmFirst");
    let hash1 = BytesN::from_array(&env, &[1u8; 32]);
    let id1 = client.register_document(
        &owner,
        &cid1,
        &hash1,
        &DocumentType::Will,
        &false,
    );
    assert_eq!(id1, 0);

    let cid2 = String::from_str(&env, "QmSecond");
    let hash2 = BytesN::from_array(&env, &[2u8; 32]);
    let id2 = client.register_document(
        &owner,
        &cid2,
        &hash2,
        &DocumentType::Deed,
        &true,
    );
    assert_eq!(id2, 1);

    let cid3 = String::from_str(&env, "QmThird");
    let hash3 = BytesN::from_array(&env, &[3u8; 32]);
    let id3 = client.register_document(
        &owner,
        &cid3,
        &hash3,
        &DocumentType::Certificate,
        &false,
    );
    assert_eq!(id3, 2);
}

// ---------------------------------------------------------------------------
// 5. test_verify_document
// ---------------------------------------------------------------------------
#[test]
fn test_verify_document() {
    let (env, client, _admin, owner) = setup();

    let cid = String::from_str(&env, "QmTest456");
    let hash = BytesN::from_array(&env, &[2u8; 32]);

    let doc_id = client.register_document(
        &owner,
        &cid,
        &hash,
        &DocumentType::Deed,
        &false,
    );

    let proof = client.verify_document(&doc_id);
    assert_eq!(proof.exists, true);
    assert_eq!(proof.doc_hash, hash);
}

// ---------------------------------------------------------------------------
// 6. test_verify_nonexistent
// ---------------------------------------------------------------------------
#[test]
fn test_verify_nonexistent() {
    let (_env, client, _admin, _owner) = setup();

    let result = client.try_verify_document(&999u64);
    assert!(result.is_err() || result.unwrap().is_err());
}

// ---------------------------------------------------------------------------
// 7. test_link_to_vault
// ---------------------------------------------------------------------------
#[test]
fn test_link_to_vault() {
    let (env, client, _admin, owner) = setup();

    let cid = String::from_str(&env, "QmLinkMe");
    let hash = BytesN::from_array(&env, &[5u8; 32]);

    let doc_id = client.register_document(
        &owner,
        &cid,
        &hash,
        &DocumentType::LegalDocument,
        &true,
    );

    let vault_id: u64 = 42;
    client.link_to_vault(&doc_id, &vault_id);

    // Verify the document is now linked
    let doc = client.get_document(&doc_id);
    assert_eq!(doc.linked, true);
    assert_eq!(doc.vault_id, vault_id);

    // Verify vault documents list contains the doc
    let vault_docs = client.get_vault_documents(&vault_id);
    assert_eq!(vault_docs.len(), 1);
    assert_eq!(vault_docs.get(0).unwrap(), doc_id);
}

// ---------------------------------------------------------------------------
// 8. test_store_and_get_encrypted_key
// ---------------------------------------------------------------------------
#[test]
fn test_store_and_get_encrypted_key() {
    let (env, client, _admin, owner) = setup();

    let beneficiary = Address::generate(&env);

    let cid = String::from_str(&env, "QmEncrypted");
    let hash = BytesN::from_array(&env, &[6u8; 32]);

    let doc_id = client.register_document(
        &owner,
        &cid,
        &hash,
        &DocumentType::PersonalLetter,
        &true,
    );

    let encrypted_key = Bytes::from_slice(&env, &[10u8, 20u8, 30u8, 40u8, 50u8]);
    client.store_encrypted_key(&doc_id, &beneficiary, &encrypted_key);

    // Grant access to verify the encrypted key is returned correctly
    let access = client.grant_access(&doc_id, &beneficiary);
    assert_eq!(access.encrypted_key, encrypted_key);
    assert_eq!(access.is_encrypted, true);
}

// ---------------------------------------------------------------------------
// 9. test_grant_access
// ---------------------------------------------------------------------------
#[test]
fn test_grant_access() {
    let (env, client, _admin, owner) = setup();

    let beneficiary = Address::generate(&env);

    let cid = String::from_str(&env, "QmAccess");
    let hash = BytesN::from_array(&env, &[7u8; 32]);

    let doc_id = client.register_document(
        &owner,
        &cid,
        &hash,
        &DocumentType::Will,
        &false,
    );

    let access = client.grant_access(&doc_id, &beneficiary);

    // Verify DocumentAccess fields
    assert_eq!(access.ipfs_cid, String::from_str(&env, "QmAccess"));
    assert_eq!(access.doc_type, DocumentType::Will);
    assert_eq!(access.is_encrypted, false);
    // No key stored, so encrypted_key should be empty
    assert_eq!(access.encrypted_key, Bytes::new(&env));
}

// ---------------------------------------------------------------------------
// 10. test_get_vault_documents
// ---------------------------------------------------------------------------
#[test]
fn test_get_vault_documents() {
    let (env, client, _admin, owner) = setup();

    let vault_id: u64 = 100;

    // Register and link three documents to the same vault
    let cid1 = String::from_str(&env, "QmVault1");
    let hash1 = BytesN::from_array(&env, &[11u8; 32]);
    let id1 = client.register_document(
        &owner,
        &cid1,
        &hash1,
        &DocumentType::Will,
        &false,
    );
    client.link_to_vault(&id1, &vault_id);

    let cid2 = String::from_str(&env, "QmVault2");
    let hash2 = BytesN::from_array(&env, &[12u8; 32]);
    let id2 = client.register_document(
        &owner,
        &cid2,
        &hash2,
        &DocumentType::Deed,
        &true,
    );
    client.link_to_vault(&id2, &vault_id);

    let cid3 = String::from_str(&env, "QmVault3");
    let hash3 = BytesN::from_array(&env, &[13u8; 32]);
    let id3 = client.register_document(
        &owner,
        &cid3,
        &hash3,
        &DocumentType::Certificate,
        &false,
    );
    client.link_to_vault(&id3, &vault_id);

    let vault_docs = client.get_vault_documents(&vault_id);
    assert_eq!(vault_docs.len(), 3);
    assert_eq!(vault_docs.get(0).unwrap(), id1);
    assert_eq!(vault_docs.get(1).unwrap(), id2);
    assert_eq!(vault_docs.get(2).unwrap(), id3);
}
