use actix_cors::Cors;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use async_graphql::{EmptySubscription, Schema};
use async_graphql_actix_web::GraphQLRequest;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod auth;
mod config;
mod db;
mod graphql;
mod models;

use auth::SessionStore;
use config::Config;
use graphql::schema::{AppSchema, MutationRoot, QueryRoot};

async fn health(db_pool: web::Data<sqlx::PgPool>) -> impl Responder {
    let pg_ok = sqlx::query("SELECT 1")
        .fetch_one(db_pool.get_ref())
        .await
        .is_ok();

    if pg_ok {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "postgres": "connected"
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "unhealthy",
            "postgres": "disconnected"
        }))
    }
}

async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(async_graphql::http::playground_source(
            async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
        ))
}

async fn graphql_handler(
    schema: web::Data<AppSchema>,
    sessions: web::Data<SessionStore>,
    http_req: HttpRequest,
    gql_req: GraphQLRequest,
) -> HttpResponse {
    let auth_user = auth::extract_auth(&http_req, sessions.get_ref()).await;

    match auth_user {
        Some(user) => {
            let request = gql_req.into_inner().data(user);
            let response = schema.execute(request).await;
            HttpResponse::Ok()
                .content_type("application/json")
                .json(response)
        }
        None => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing or invalid Authorization header. Use POST /auth first."
        })),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env: cwd first, then oracle/.env
    dotenvy::dotenv().ok();
    dotenvy::from_path("oracle/.env").ok();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    let config = Config::from_env();
    info!("Configuration loaded successfully");

    // Initialize database pool
    let db_pool = db::postgres::create_pool(&config.database_url)
        .await
        .unwrap_or_else(|e| {
            let hint = db::postgres::postgres_utf8_hint(&e);
            eprintln!(
                "Failed to create PostgreSQL pool: {}.\n{}",
                e,
                hint.as_deref().unwrap_or("")
            );
            std::process::exit(1);
        });
    info!("PostgreSQL connection pool created");

    // Run migrations
    db::postgres::run_migrations(&db_pool).await;
    info!("Database migrations applied");

    // Session store (in-memory)
    let sessions: SessionStore = Arc::new(RwLock::new(HashMap::new()));
    info!("Session store initialized");

    // Build GraphQL schema
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(db_pool.clone())
        .finish();
    info!("GraphQL schema built");

    let host = config.host.clone();
    let port = config.port;

    info!("Starting server at {}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(schema.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(sessions.clone()))
            .route("/health", web::get().to(health))
            .route("/auth", web::post().to(auth::auth_handler))
            .route("/graphql", web::post().to(graphql_handler))
            .route("/graphql/playground", web::get().to(graphql_playground))
    })
    .bind((host, port))?
    .run()
    .await
}
