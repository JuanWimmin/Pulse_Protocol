use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Verification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub score: i32,
    pub source: String,
    pub perceptron_output: Option<i32>,
    pub on_chain_tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Verification {
    /// Get verification history for a user.
    pub async fn find_by_user(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM verifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    /// Get latest verification for a user.
    pub async fn latest(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM verifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    /// Count total verifications for a user.
    pub async fn count(pool: &PgPool, user_id: Uuid) -> sqlx::Result<i64> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM verifications WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(pool)
                .await?;
        Ok(row.0)
    }

    /// Create a new verification.
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        score: i32,
        source: &str,
        perceptron_output: Option<i32>,
        on_chain_tx_hash: Option<&str>,
    ) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            r#"INSERT INTO verifications (user_id, score, source, perceptron_output, on_chain_tx_hash)
               VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
        )
        .bind(user_id)
        .bind(score)
        .bind(source)
        .bind(perceptron_output)
        .bind(on_chain_tx_hash)
        .fetch_one(pool)
        .await
    }
}
