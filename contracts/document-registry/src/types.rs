use soroban_sdk::{contracttype, Address, Bytes, BytesN, String};

pub type DocId = u64;

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DocumentType {
    PublicManifest,
    Deed,
    Will,
    Certificate,
    LegalDocument,
    PersonalLetter,
    Other,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct DocumentInfo {
    pub id: DocId,
    pub owner: Address,
    pub ipfs_cid: String,
    pub doc_hash: BytesN<32>,
    pub doc_type: DocumentType,
    pub is_encrypted: bool,
    pub registered_at: u64,
    pub vault_id: u64,
    pub linked: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct DocumentProof {
    pub exists: bool,
    pub doc_hash: BytesN<32>,
    pub registered_at: u64,
    pub ipfs_cid: String,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct DocumentAccess {
    pub ipfs_cid: String,
    pub encrypted_key: Bytes,
    pub doc_type: DocumentType,
    pub is_encrypted: bool,
}
