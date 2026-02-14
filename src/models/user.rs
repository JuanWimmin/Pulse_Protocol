use chr>{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Cl>, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub stellar_address: String,
    pub created_at: DateTime<Utc>,
    pub calibrati>: bool,
    pub calibrati>: Opti><DateTime<Utc>>,
}

impl User {
    /// Buscar usuario por ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Opti><Self>> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// Buscar usuario por direccion Stellar.
    pub async fn find_by_address(pool: &PgPool, address: &str) -> sqlx::Result<Opti><Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM users WHERE stellar_address = $1",
        )
        .bind(address)
        .fetch_optional(pool)
        .await
    }

    /// Crear un nuevo usuario. Retorna el usuario creado.
    pub async fn create(pool: &PgPool, stellar_address: &str) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO users (stellar_address) VALUES ($1) RETURNING *",
        )
        .bind(stellar_address)
        .fetch_one(pool)
        .await
    }

    /// Buscar o crear usuario por direccion Stellar.
    pub async fn find_or_create(pool: &PgPool, stellar_address: &str) -> sqlx::Result<Self> {
        if let Some(user) = Self::find_by_address(pool, stellar_address).await? {
            Ok(user)
        } else {
            Self::create(pool, stellar_address).await
        }
    }

    /// Marcar calibracion como completada.
    pub async fn complete_calibrati>: &PgPool, id: Uuid) -> sqlx::Result<()> {
        sqlx::query("UPDATE users SET calibration_complete = TRUE WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

6.3 src/models/vault.rs

use chr>{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Cl>, Serialize, Deserialize, FromRow)]
pub struct Vault {
    pub id: Uuid,
    pub c>: Opti><i64>,
    pub owner_id: Uuid,
    pub token_address: String,
    pub status: String,
    pub balance: i64,
    pub escrow_c>: Opti><String>,
    pub created_at: DateTime<Utc>,
    pub last_synced_at: DateTime<Utc>,
}

impl Vault {
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Opti><Self>> {
        sqlx::query_as::<_, Self>("SELECT * FROM vaults WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// Obtener todos los vaults de un usuario.
    pub async fn find_by_owner(pool: &PgPool, owner_id: Uuid) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM vaults WHERE owner_id = $1 ORDER BY created_at DESC",
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await
    }

    /// Crear un vault nuevo.
    pub async fn create(
        pool: &PgPool,
        owner_id: Uuid,
        token_address: &str,
        c>: Opti><i64>,
    ) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            r#"INSERT INTO vaults (owner_id, token_address, contract_vault_id)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(owner_id)
        .bind(token_address)
        .bind(contract_vault_id)
        .fetch_one(pool)
        .await
    }

    /// Actualizar estado del vault.
    pub async fn update_status(pool: &PgPool, id: Uuid, status: &str) -> sqlx::Result<()> {
        sqlx::query(
            "UPDATE vaults SET status = $1, last_synced_at = NOW() WHERE id = $2",
        )
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Actualizar balance.
    pub async fn update_balance(pool: &PgPool, id: Uuid, balance: i64) -> sqlx::Result<()> {
        sqlx::query("UPDATE vaults SET balance = $1, last_synced_at = NOW() WHERE id = $2")
            .bind(balance)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Obtener vaults en un estado especifico.
    pub async fn find_by_status(pool: &PgPool, status: &str) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>("SELECT * FROM vaults WHERE status = $1")
            .bind(status)
            .fetch_all(pool)
            .await
    }
}

