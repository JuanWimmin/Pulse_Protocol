use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Error as SqlxError};

/// Crea un pool de conexiones a PostgreSQL.
/// El pool reutiliza conexiones automaticamente.
pub async fn create_pool(database_url: &str) -> Result<PgPool, SqlxError> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

/// Hint when Postgres returns non-UTF-8 error (common on Windows with Docker).
pub fn postgres_utf8_hint(err: &SqlxError) -> Option<String> {
    let msg = err.to_string();
    if msg.contains("non-UTF-8") || msg.contains("lc_messages") {
        Some(
            "Tip: Run from the 'oracle' folder so .env is loaded. \
             If the error persists, recreate Postgres with UTF-8 locale: \
             docker compose down -v && docker compose up -d (then run migrations).".into(),
        )
    } else {
        None
    }
}

/// Ejecuta las migraciones pendientes.
/// Llama esto al arrancar el servidor.
pub async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");
}