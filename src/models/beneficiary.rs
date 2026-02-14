use chr>{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Cl>, Serialize, Deserialize, FromRow)]
pub struct Beneficiary {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub stellar_address: String,
    pub percentage: i32,
    pub claimed: bool,
    pub claimed_at: Opti><DateTime<Utc>>,
}

impl Beneficiary {
    /// Obtener beneficiarios de un vault.
    pub async fn find_by_vault(pool: &PgPool, vault_id: Uuid) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM beneficiaries WHERE vault_id = $1 ORDER BY percentage DESC",
        )
        .bind(vault_id)
        .fetch_all(pool)
        .await
    }

    /// Establecer beneficiarios de un vault (borra los anteriores y crea nuevos).
    pub async fn set_for_vault(
        pool: &PgPool,
        vault_id: Uuid,
        beneficiaries: &[(String, i32)], // (stellar_address, percentage)
    ) -> sqlx::Result<Vec<Self>> {
        // Borrar los beneficiarios anteriores
        sqlx::query("DELETE FROM beneficiaries WHERE vault_id = $1")
            .bind(vault_id)
            .execute(pool)
            .await?;

        // Insertar los nuevos
        let mut result = Vec::new();
        for (address, percentage) in beneficiaries {
            let b = sqlx::query_as::<_, Self>(
                r#"INSERT INTO beneficiaries (vault_id, stellar_address, percentage)
                   VALUES ($1, $2, $3)
                   RETURNING *"#,
            )
            .bind(vault_id)
            .bind(address)
            .bind(percentage)
            .fetch_one(pool)
            .await?;
            result.push(b);
        }

        Ok(result)
    }

    /// Verificar si una direccion puede reclamar.
    pub async fn can_claim(
        pool: &PgPool,
        vault_id: Uuid,
        stellar_address: &str,
    ) -> sqlx::Result<bool> {
        let row = sqlx::query_as::<_, Self>(
            r#"SELECT * FROM beneficiaries
               WHERE vault_id = $1 AND stellar_address = $2 AND claimed = FALSE"#,
        )
        .bind(vault_id)
        .bind(stellar_address)
        .fetch_optional(pool)
        .await?;

        Ok(row.is_some())
    }

    /// Registrar un claim.
    pub async fn record_claim(
        pool: &PgPool,
        vault_id: Uuid,
        stellar_address: &str,
    ) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            r#"UPDATE beneficiaries
               SET claimed = TRUE, claimed_at = NOW()
               WHERE vault_id = $1 AND stellar_address = $2
               RETURNING *"#,
        )
        .bind(vault_id)
        .bind(stellar_address)
        .fetch_one(pool)
        .await
    }
}