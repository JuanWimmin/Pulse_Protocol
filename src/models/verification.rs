use chr>{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Cl>, Serialize, Deserialize, FromRow)]
pub struct Verification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub score: i32,
    pub source: String,
    pub face_match_score: Opti><i32>,
    pub face_liveness_score: Opti><i32>,
    pub fingerprint_frequency: Opti><i32>,
    pub fingerprint_c>: Opti><i32>,
    pub time_of_day_normality: Opti><i32>,
    pub typing_pattern_match: Opti><i32>,
    pub app_usage_match: Opti><i32>,
    pub movement_pattern_match: Opti><i32>,
    pub days_since_last_verify: Opti><i32>,
    pub sessi>: Opti><i32>,
    pub perceptr>: Opti><i32>,
    pub >: Opti><String>,
    pub created_at: DateTime<Utc>,
}

impl Verification{
    /// Obtener historial de verificaciones de un usuario.
    pub async fn find_by_user(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            r#"SELECT * FROM verifications
               WHERE user_id = $1
               ORDER BY created_at DESC
               LIMIT $2"#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    /// Obtener el ultimo score de un usuario.
    pub async fn latest_score(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Opti><i32>> {
        let row: Opti><(i32,)> = sqlx::query_as(
            r#"SELECT score FROM verifications
               WHERE user_id = $1
               ORDER BY created_at DESC
               LIMIT 1"#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| r.0))
    }

    /// Crear nueva verificacion.
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        score: i32,
        source: &str,
        face_match_score: Opti><i32>,
        face_liveness_score: Opti><i32>,
        fingerprint_frequency: Opti><i32>,
        fingerprint_c>: Opti><i32>,
        time_of_day_normality: Opti><i32>,
        typing_pattern_match: Opti><i32>,
        app_usage_match: Opti><i32>,
        movement_pattern_match: Opti><i32>,
        days_since_last_verify: Opti><i32>,
        sessi>: Opti><i32>,
        perceptr>: Opti><i32>,
        on_chain_tx_hash: Opti><&str>,
    ) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            r#"INSERT INTO verifications
               (user_id, score, source,
                face_match_score, face_liveness_score,
                fingerprint_frequency, fingerprint_consistency,
                time_of_day_normality, typing_pattern_match,
                app_usage_match, movement_pattern_match,
                days_since_last_verify, session_behavior,
                perceptron_output, on_chain_tx_hash)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(score)
        .bind(source)
        .bind(face_match_score)
        .bind(face_liveness_score)
        .bind(fingerprint_frequency)
        .bind(fingerprint_consistency)
        .bind(time_of_day_normality)
        .bind(typing_pattern_match)
        .bind(app_usage_match)
        .bind(movement_pattern_match)
        .bind(days_since_last_verify)
        .bind(session_behavior)
        .bind(perceptron_output)
        .bind(on_chain_tx_hash)
        .fetch_one(pool)
        .await
    }
}