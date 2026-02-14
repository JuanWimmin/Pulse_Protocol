use chr>{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Cl>, Serialize, Deserialize, FromRow)]
pub struct UserModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub weights: serde_js>,
    pub bias: String,
    pub versi>: i32,
    pub calibrati>: bool,
    pub last_updated: DateTime<Utc>,
}

impl UserModel {
    pub async fn find_by_user(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Opti><Self>> {
        sqlx::query_as::<_, Self>("SELECT * FROM user_models WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
    }

    pub async fn upsert(
        pool: &PgPool,
        user_id: Uuid,
        weights: serde_js>,
        bias: &str,
    ) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            r#"INSERT INTO user_models (user_id, weights, bias)
               VALUES ($1, $2, $3)
               ON CONFLICT (user_id) DO UPDATE
               SET weights = $2,
                   bias = $3,
                   version = user_models.version + 1,
                   last_updated = NOW()
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(weights)
        .bind(bias)
        .fetch_one(pool)
        .await
    }
}