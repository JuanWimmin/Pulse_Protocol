use async_graphql::{Context, EmptySubscription, Object, Result, Schema};
use sqlx::PgPool;
use uuid::Uuid;

use super::types::*;

#[derive(sqlx::FromRow)]
struct UserIdRow { id: Uuid }

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get user by Stellar address
    async fn user(&self, ctx: &Context<'_>, stellar_address: String) -> Result<Option<User>> {
        let pool = ctx.data::<PgPool>()?;
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, stellar_address, created_at, calibration_complete, calibration_started_at
               FROM users WHERE stellar_address = $1"#,
        )
        .bind(&stellar_address)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    /// Get vault by ID
    async fn vault(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<Vault>> {
        let pool = ctx.data::<PgPool>()?;
        let vault = sqlx::query_as::<_, Vault>(
            r#"SELECT id, contract_vault_id, owner_id, token_address, status, balance,
                      escrow_contract, created_at, last_synced_at
               FROM vaults WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(vault)
    }

    /// Get all vaults for a user
    async fn user_vaults(&self, ctx: &Context<'_>, stellar_address: String) -> Result<Vec<Vault>> {
        let pool = ctx.data::<PgPool>()?;
        let vaults = sqlx::query_as::<_, Vault>(
            r#"SELECT v.id, v.contract_vault_id, v.owner_id, v.token_address, v.status,
                      v.balance, v.escrow_contract, v.created_at, v.last_synced_at
               FROM vaults v JOIN users u ON v.owner_id = u.id
               WHERE u.stellar_address = $1"#,
        )
        .bind(&stellar_address)
        .fetch_all(pool)
        .await?;
        Ok(vaults)
    }

    /// Get beneficiaries for a vault
    async fn vault_beneficiaries(&self, ctx: &Context<'_>, vault_id: Uuid) -> Result<Vec<Beneficiary>> {
        let pool = ctx.data::<PgPool>()?;
        let beneficiaries = sqlx::query_as::<_, Beneficiary>(
            r#"SELECT id, vault_id, stellar_address, percentage, claimed, claimed_at
               FROM beneficiaries WHERE vault_id = $1"#,
        )
        .bind(vault_id)
        .fetch_all(pool)
        .await?;
        Ok(beneficiaries)
    }

    /// Get documents for a user
    async fn user_documents(&self, ctx: &Context<'_>, stellar_address: String) -> Result<Vec<Document>> {
        let pool = ctx.data::<PgPool>()?;
        let documents = sqlx::query_as::<_, Document>(
            r#"SELECT d.id, d.owner_id, d.vault_id, d.ipfs_cid, d.doc_hash, d.doc_type,
                      d.is_encrypted, d.metadata, d.contract_doc_id, d.registered_at
               FROM documents d JOIN users u ON d.owner_id = u.id
               WHERE u.stellar_address = $1"#,
        )
        .bind(&stellar_address)
        .fetch_all(pool)
        .await?;
        Ok(documents)
    }

    /// Get verification history for a user
    async fn user_verifications(&self, ctx: &Context<'_>, stellar_address: String) -> Result<Vec<VerificationRecord>> {
        let pool = ctx.data::<PgPool>()?;
        let verifications = sqlx::query_as::<_, VerificationRecord>(
            r#"SELECT v.id, v.user_id, v.score, v.source, v.face_match_score, v.face_liveness_score,
                      v.fingerprint_frequency, v.fingerprint_count, v.time_of_day_normality,
                      v.typing_pattern_match, v.app_usage_match, v.movement_pattern_match,
                      v.days_since_last_verify, v.session_duration, v.perceptron_score, v.hash, v.created_at
               FROM verifications v JOIN users u ON v.user_id = u.id
               WHERE u.stellar_address = $1 ORDER BY v.created_at DESC"#,
        )
        .bind(&stellar_address)
        .fetch_all(pool)
        .await?;
        Ok(verifications)
    }

    /// Get current liveness data for a user
    async fn liveness_status(&self, ctx: &Context<'_>, stellar_address: String) -> Result<Option<LivenessData>> {
        let pool = ctx.data::<PgPool>()?;
        let user = sqlx::query_as::<_, User>(
            "SELECT id, stellar_address, created_at, calibration_complete, calibration_started_at FROM users WHERE stellar_address = $1",
        )
        .bind(&stellar_address)
        .fetch_optional(pool)
        .await?;
        let user = match user {
            Some(u) => u,
            None => return Ok(None),
        };
        #[derive(sqlx::FromRow)]
        struct LivenessRow { score: i32, created_at: chrono::DateTime<chrono::Utc>, total_count: Option<i64> }
        let latest = sqlx::query_as::<_, LivenessRow>(
            r#"SELECT score, created_at, COUNT(*) OVER() as total_count
               FROM verifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(user.id)
        .fetch_optional(pool)
        .await?;
        match latest {
            Some(v) => Ok(Some(LivenessData {
                user_id: user.id,
                current_score: v.score,
                last_verification: v.created_at,
                verification_count: v.total_count.unwrap_or(0) as i32,
                calibration_complete: user.calibration_complete,
            })),
            None => Ok(Some(LivenessData {
                user_id: user.id,
                current_score: 0,
                last_verification: user.created_at,
                verification_count: 0,
                calibration_complete: user.calibration_complete,
            })),
        }
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new user
    async fn create_user(&self, ctx: &Context<'_>, stellar_address: String) -> Result<User> {
        let pool = ctx.data::<PgPool>()?;
        let user = sqlx::query_as::<_, User>(
            r#"INSERT INTO users (stellar_address) VALUES ($1)
               RETURNING id, stellar_address, created_at, calibration_complete, calibration_started_at"#,
        )
        .bind(&stellar_address)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }

    /// Create a new vault
    async fn create_vault(&self, ctx: &Context<'_>, input: CreateVaultInput) -> Result<Vault> {
        let pool = ctx.data::<PgPool>()?;
        let user = sqlx::query_as::<_, UserIdRow>("SELECT id FROM users WHERE stellar_address = $1")
            .bind(&input.stellar_address)
            .fetch_one(pool)
            .await?;
        let vault = sqlx::query_as::<_, Vault>(
            r#"INSERT INTO vaults (owner_id, token_address) VALUES ($1, $2)
               RETURNING id, contract_vault_id, owner_id, token_address, status, balance,
                         escrow_contract, created_at, last_synced_at"#,
        )
        .bind(user.id)
        .bind(&input.token_address)
        .fetch_one(pool)
        .await?;
        Ok(vault)
    }

    /// Add beneficiary to vault
    async fn add_beneficiary(&self, ctx: &Context<'_>, vault_id: Uuid, beneficiary: BeneficiaryInput) -> Result<Beneficiary> {
        let pool = ctx.data::<PgPool>()?;
        let new_beneficiary = sqlx::query_as::<_, Beneficiary>(
            r#"INSERT INTO beneficiaries (vault_id, stellar_address, percentage)
               VALUES ($1, $2, $3)
               RETURNING id, vault_id, stellar_address, percentage, claimed, claimed_at"#,
        )
        .bind(vault_id)
        .bind(&beneficiary.stellar_address)
        .bind(beneficiary.percentage)
        .fetch_one(pool)
        .await?;
        Ok(new_beneficiary)
    }

    /// Register a document
    async fn register_document(&self, ctx: &Context<'_>, stellar_address: String, input: RegisterDocumentInput) -> Result<Document> {
        let pool = ctx.data::<PgPool>()?;
        let user = sqlx::query_as::<_, UserIdRow>("SELECT id FROM users WHERE stellar_address = $1")
            .bind(&stellar_address)
            .fetch_one(pool)
            .await?;
        let document = sqlx::query_as::<_, Document>(
            r#"INSERT INTO documents (owner_id, vault_id, ipfs_cid, doc_hash, doc_type, is_encrypted, metadata)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id, owner_id, vault_id, ipfs_cid, doc_hash, doc_type, is_encrypted,
                         metadata, contract_doc_id, registered_at"#,
        )
        .bind(user.id)
        .bind(input.vault_id)
        .bind(&input.ipfs_cid)
        .bind(&input.doc_hash)
        .bind(&input.doc_type)
        .bind(input.is_encrypted)
        .bind(&input.metadata)
        .fetch_one(pool)
        .await?;
        Ok(document)
    }

    /// Submit verification data
    async fn submit_verification(&self, ctx: &Context<'_>, input: VerificationInput) -> Result<VerificationRecord> {
        let pool = ctx.data::<PgPool>()?;
        let user = sqlx::query_as::<_, UserIdRow>("SELECT id FROM users WHERE stellar_address = $1")
            .bind(&input.stellar_address)
            .fetch_one(pool)
            .await?;
        let mut scores = Vec::new();
        if let Some(s) = input.face_match_score { scores.push(s); }
        if let Some(s) = input.face_liveness_score { scores.push(s); }
        if let Some(s) = input.fingerprint_frequency { scores.push(s); }
        if let Some(s) = input.typing_pattern_match { scores.push(s); }
        if let Some(s) = input.app_usage_match { scores.push(s); }
        if let Some(s) = input.movement_pattern_match { scores.push(s); }
        let score = if scores.is_empty() { 0 } else { scores.iter().sum::<i32>() / scores.len() as i32 };
        let verification = sqlx::query_as::<_, VerificationRecord>(
            r#"INSERT INTO verifications (user_id, score, source, face_match_score, face_liveness_score,
                                         fingerprint_frequency, typing_pattern_match, app_usage_match,
                                         movement_pattern_match)
               VALUES ($1, $2, 'behavioral', $3, $4, $5, $6, $7, $8)
               RETURNING id, user_id, score, source, face_match_score, face_liveness_score,
                         fingerprint_frequency, fingerprint_count, time_of_day_normality,
                         typing_pattern_match, app_usage_match, movement_pattern_match,
                         days_since_last_verify, session_duration, perceptron_score, hash, created_at"#,
        )
        .bind(user.id)
        .bind(score)
        .bind(input.face_match_score)
        .bind(input.face_liveness_score)
        .bind(input.fingerprint_frequency)
        .bind(input.typing_pattern_match)
        .bind(input.app_usage_match)
        .bind(input.movement_pattern_match)
        .fetch_one(pool)
        .await?;
        Ok(verification)
    }
}