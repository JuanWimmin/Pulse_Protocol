use async_graphql::{Enum, InputObject, Object, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ========== ENUMS ==========

#[derive(Enum, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum VaultStatus {
    Active,
    Triggered,
    Distributed,
    Cancelled,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum DocumentType {
    Will,
    Letter,
    Credentials,
    Legal,
    Financial,
    Medical,
    Other,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum VerificationSource {
    Manual,
    Biometric,
    Behavioral,
    Perceptron,
    Oracle,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

// ========== TYPES ==========

#[derive(SimpleObject, Clone, Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub stellar_address: String,
    pub created_at: DateTime<Utc>,
    pub calibration_complete: bool,
    pub calibration_started_at: Option<DateTime<Utc>>,
}

#[derive(SimpleObject, Clone, Debug, FromRow)]
pub struct Vault {
    pub id: Uuid,
    pub contract_vault_id: Option<i64>,
    pub owner_id: Uuid,
    pub token_address: String,
    pub status: String,
    pub balance: i64,
    pub escrow_contract: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_synced_at: DateTime<Utc>,
}

#[derive(SimpleObject, Clone, Debug, FromRow)]
pub struct Beneficiary {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub stellar_address: String,
    pub percentage: i32,
    pub claimed: bool,
    pub claimed_at: Option<DateTime<Utc>>,
}

#[derive(SimpleObject, Clone, Debug, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub vault_id: Option<Uuid>,
    pub ipfs_cid: String,
    pub doc_hash: String,
    pub doc_type: String,
    pub is_encrypted: bool,
    pub metadata: Option<serde_json::Value>,
    pub contract_doc_id: Option<i64>,
    pub registered_at: DateTime<Utc>,
}

#[derive(SimpleObject, Clone, Debug, FromRow)]
pub struct VerificationRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub score: i32,
    pub source: String,
    pub face_match_score: Option<i32>,
    pub face_liveness_score: Option<i32>,
    pub fingerprint_frequency: Option<i32>,
    pub fingerprint_count: Option<i32>,
    pub time_of_day_normality: Option<i32>,
    pub typing_pattern_match: Option<i32>,
    pub app_usage_match: Option<i32>,
    pub movement_pattern_match: Option<i32>,
    pub days_since_last_verify: Option<i32>,
    pub session_duration: Option<i32>,
    pub perceptron_score: Option<i32>,
    pub hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct LivenessData {
    pub user_id: Uuid,
    pub current_score: i32,
    pub last_verification: DateTime<Utc>,
    pub verification_count: i32,
    pub calibration_complete: bool,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct TokenBalance {
    pub token_address: String,
    pub balance: String,
    pub decimals: i32,
}

// ========== INPUT TYPES ==========

#[derive(InputObject, Debug)]
pub struct CreateVaultInput {
    pub stellar_address: String,
    pub token_address: String,
}

#[derive(InputObject, Debug)]
pub struct BeneficiaryInput {
    pub stellar_address: String,
    pub percentage: i32,
}

#[derive(InputObject, Debug)]
pub struct RegisterDocumentInput {
    pub vault_id: Option<Uuid>,
    pub ipfs_cid: String,
    pub doc_hash: String,
    pub doc_type: String,
    pub is_encrypted: bool,
    pub metadata: Option<serde_json::Value>,
}

#[derive(InputObject, Debug)]
pub struct VerificationInput {
    pub stellar_address: String,
    pub face_match_score: Option<i32>,
    pub face_liveness_score: Option<i32>,
    pub fingerprint_frequency: Option<i32>,
    pub typing_pattern_match: Option<i32>,
    pub app_usage_match: Option<i32>,
    pub movement_pattern_match: Option<i32>,
}

#[derive(InputObject, Debug)]
pub struct EncryptedKeyInput {
    pub document_id: Uuid,
    pub beneficiary_address: String,
    pub encrypted_key: String,
}