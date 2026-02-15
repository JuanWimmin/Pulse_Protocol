use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vault {
    pub id: Uuid,
    pub contract_id: Option<String>,
    pub owner_id: Uuid,
    pub status: String,
    pub escrow_contract_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_synced_at: DateTime<Utc>,
}

impl Vault {
    /// Find vault by ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>("SELECT * FROM vaults WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// Get all vaults for a user.
    pub async fn find_by_owner(pool: &PgPool, owner_id: Uuid) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM vaults WHERE owner_id = $1 ORDER BY created_at DESC",
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await
    }

    /// Create a new vault.
    pub async fn create(pool: &PgPool, owner_id: Uuid) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO vaults (owner_id) VALUES ($1) RETURNING *",
        )
        .bind(owner_id)
        .fetch_one(pool)
        .await
    }

    /// Update vault status.
    pub async fn update_status(pool: &PgPool, id: Uuid, status: &str) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            "UPDATE vaults SET status = $1, last_synced_at = NOW() WHERE id = $2 RETURNING *",
        )
        .bind(status)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// Set contract ID after on-chain deployment.
    pub async fn set_contract_id(pool: &PgPool, id: Uuid, contract_id: &str) -> sqlx::Result<()> {
        sqlx::query(
            "UPDATE vaults SET contract_id = $1, last_synced_at = NOW() WHERE id = $2",
        )
        .bind(contract_id)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Set escrow contract ID.
    pub async fn set_escrow(
        pool: &PgPool,
        id: Uuid,
        escrow_contract_id: &str,
    ) -> sqlx::Result<()> {
        sqlx::query(
            "UPDATE vaults SET escrow_contract_id = $1, last_synced_at = NOW() WHERE id = $2",
        )
        .bind(escrow_contract_id)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }
}
