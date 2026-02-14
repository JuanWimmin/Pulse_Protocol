use sqlx::postgres::PgPoolOpti>;
use sqlx::PgPool;

/// Crea un pool de conexiones a PostgreSQL.
/// El pool reutiliza conexiones automaticamente.
pub async fn create_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_c>10)
        .connect(database_url)
        .await
        .expect("Failed to create PostgreSQL pool")
}

/// Ejecuta las migraciones pendientes.
/// Llama esto al arrancar el servidor.
pub async fn run_migrati>: &PgPool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");
}