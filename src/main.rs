use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use async_graphql::{EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod db;
mod graphql;

use config::Config;
use graphql::schema::{QueryRoot, MutationRoot, AppSchema};

async fn health(
    db_pool: web::Data<sqlx::PgPool>,
    redis_client: web::Data<redis::Client>,
) -> impl Responder {
    // Check PostgreSQL
    let pg_ok = sqlx::query("SELECT 1")
        .fetch_one(db_pool.get_ref())
        .await
        .is_ok();

    // Check Redis
    let redis_ok = redis_client
        .get_multiplexed_async_connection()
        .await
        .is_ok();

    if pg_ok && redis_ok {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "postgres": "connected",
            "redis": "connected"
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "unhealthy",
            "postgres": if pg_ok { "connected" } else { "disconnected" },
            "redis": if redis_ok { "connected" } else { "disconnected" }
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
    req: GraphQLRequest,
) -> GraphQLResponse {
    GraphQLResponse(async_graphql::BatchResponse::Single(
        schema.execute(req.into_inner()).await,
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env: cwd first, then oracle/.env (so "cargo run" from repo root still gets oracle config)
    dotenvy::dotenv().ok();
    dotenvy::from_path("oracle/.env").ok();

    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    // Load configuration
    let config = Config::from_env();
    info!("Configuration loaded successfully");

    // Initialize database pool
    let db_pool = db::postgres::create_pool(&config.database_url).await.unwrap_or_else(|e| {
        let hint = db::postgres::postgres_utf8_hint(&e);
        eprintln!(
            "Failed to create PostgreSQL pool: {}.\n{}",
            e,
            hint.as_deref().unwrap_or("")
        );
        std::process::exit(1);
    });
    info!("PostgreSQL connection pool created");

    // Initialize Redis client
    let redis_client = db::redis::create_client(&config.redis_url);
    info!("Redis client created");

    // Build GraphQL schema
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(db_pool.clone())
        .data(redis_client.clone())
        .finish();
    info!("GraphQL schema built");

    let host = config.host.clone();
    let port = config.port;

    info!("Starting server at {}:{}", host, port);

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(schema.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(redis_client.clone()))
            .route("/health", web::get().to(health))
            .route("/graphql", web::post().to(graphql_handler))
            .route("/graphql/playground", web::get().to(graphql_playground))
    })
    .bind((host, port))?
    .run()
    .await
}
