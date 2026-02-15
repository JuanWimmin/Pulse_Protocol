use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub stellar_address: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    /// Find user by ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// Find user by Stellar address.
    pub async fn find_by_address(pool: &PgPool, address: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE stellar_address = $1")
            .bind(address)
            .fetch_optional(pool)
            .await
    }

    /// Create a new user.
    pub async fn create(pool: &PgPool, stellar_address: &str) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO users (stellar_address) VALUES ($1) RETURNING *",
        )
        .bind(stellar_address)
        .fetch_one(pool)
        .await
    }

    /// Find or create user by Stellar address.
    pub async fn find_or_create(pool: &PgPool, stellar_address: &str) -> sqlx::Result<Self> {
        if let Some(user) = Self::find_by_address(pool, stellar_address).await? {
            Ok(user)
        } else {
            Self::create(pool, stellar_address).await
        }
    }
}
