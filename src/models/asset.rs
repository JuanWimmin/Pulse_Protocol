use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Asset {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub symbol: String,
    pub amount: f64,
    pub value_usd: f64,
    pub custody: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl Asset {
    pub async fn find_by_owner(pool: &PgPool, owner_id: Uuid) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM assets WHERE owner_id = $1 ORDER BY created_at DESC",
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        owner_id: Uuid,
        name: &str,
        symbol: &str,
        amount: f64,
        value_usd: f64,
        custody: bool,
        status: &str,
    ) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO assets (owner_id, name, symbol, amount, value_usd, custody, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING *",
        )
        .bind(owner_id)
        .bind(name)
        .bind(symbol)
        .bind(amount)
        .bind(value_usd)
        .bind(custody)
        .bind(status)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid, owner_id: Uuid) -> sqlx::Result<bool> {
        let result = sqlx::query("DELETE FROM assets WHERE id = $1 AND owner_id = $2")
            .bind(id)
            .bind(owner_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
