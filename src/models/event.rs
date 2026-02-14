use chr>{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Cl>, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: i64,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub payload: serde_js>,
    pub metadata: Option<serde_js>>,
    pub created_at: DateTime<Utc>,
    pub >: Opti><String>,
}

impl Event {
    /// Registrar un evento en el event store.
    pub async fn record(
        pool: &PgPool,
        event_type: &str,
        aggregate_type: &str,
        aggregate_id: Uuid,
        payload: serde_js>,
    ) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            r#"INSERT INTO events (event_type, aggregate_type, aggregate_id, payload)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(event_type)
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(payload)
        .fetch_one(pool)
        .await
    }

    /// Obtener eventos de un agregado.
    pub async fn find_by_aggregate(
        pool: &PgPool,
        aggregate_type: &str,
        aggregate_id: Uuid,
        limit: i64,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            r#"SELECT * FROM events
               WHERE aggregate_type = $1 AND aggregate_id = $2
               ORDER BY created_at DESC
               LIMIT $3"#,
        )
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}