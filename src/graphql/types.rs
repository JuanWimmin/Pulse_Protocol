use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ========== ENUMS ==========

#[derive(Enum, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum VaultStatus {
    Active,
    Alert,
    GracePeriod,
    Triggered,
    Distributed,
}

impl VaultStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Alert => "alert",
            Self::GracePeriod => "grace_period",
            Self::Triggered => "triggered",
            Self::Distributed => "distributed",
        }
    }

    pub fn from_db(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "alert" => Self::Alert,
            "grace_period" => Self::GracePeriod,
            "triggered" => Self::Triggered,
            "distributed" => Self::Distributed,
            _ => Self::Active,
        }
    }
}

// ========== OUTPUT TYPES ==========

#[derive(SimpleObject, Clone, Debug)]
pub struct Vault {
    pub id: Uuid,
    pub contract_id: Option<String>,
    pub owner: String,
    pub status: VaultStatus,
    pub beneficiaries: Vec<Beneficiary>,
    pub balance: Vec<TokenBalance>,
    pub escrow_contract: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct Beneficiary {
    pub address: String,
    pub percentage: i32,
    pub claimed: bool,
    pub claimed_at: Option<DateTime<Utc>>,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct LivenessData {
    pub score: i32,
    pub last_verified: DateTime<Utc>,
    pub total_verifications: i32,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct TokenBalance {
    pub token: String,
    pub amount: String,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub symbol: String,
    pub amount: f64,
    pub value_usd: f64,
    pub custody: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct ActivityEvent {
    pub id: Uuid,
    pub title: String,
    pub detail: Option<String>,
    pub kind: String,
    pub created_at: DateTime<Utc>,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct TransactionResult {
    pub success: bool,
    pub tx_hash: Option<String>,
    pub message: String,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct VerificationResult {
    pub score: i32,
    pub tx_hash: Option<String>,
    pub vault_status: Option<String>,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct CheckinResult {
    pub success: bool,
    pub new_score: i32,
    pub tx_hash: Option<String>,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct ClaimResult {
    pub success: bool,
    pub amount_received: String,
    pub tx_hash: Option<String>,
}

// ========== INPUT TYPES ==========

#[derive(InputObject, Debug)]
pub struct CreateVaultInput {
    pub token: String,
    pub initial_deposit: Option<String>,
}

#[derive(InputObject, Debug)]
pub struct BeneficiaryInput {
    pub address: String,
    pub percentage: i32,
}

#[derive(InputObject, Debug)]
pub struct VerificationInput {
    pub perceptron_output: i32,
    pub source: String,
}

#[derive(InputObject, Debug)]
pub struct AddAssetInput {
    pub name: String,
    pub symbol: String,
    pub amount: f64,
    pub value_usd: f64,
    pub custody: bool,
    pub status: Option<String>,
}

#[derive(InputObject, Debug)]
pub struct ActivityInput {
    pub title: String,
    pub detail: Option<String>,
    pub kind: String,
}
