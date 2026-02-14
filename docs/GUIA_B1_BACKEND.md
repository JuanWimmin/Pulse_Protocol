# GUIA B1 — Backend / Infraestructura / API GraphQL / Base de Datos

## Guia paso a paso para el perfil B1 de Pulse Protocol

**Importante:** Esta guia esta disenada para que B1 pueda desarrollar de forma independiente, sin necesitar el trabajo de B2 (Soroban) ni B3 (Jobs/IPFS). Las dependencias con esos perfiles se resuelven con interfaces mock que se reemplazan despues.

---

## PARTE 0: PREREQUISITOS

### 0.1 Instalar Docker Desktop

PostgreSQL y Redis corren en contenedores Docker. Es la forma mas simple.

1. Descargar Docker Desktop desde https://www.docker.com/products/docker-desktop/
2. Instalar y reiniciar Windows
3. Verificar:

```bash
docker --version
docker compose version
```

### 0.2 Instalar Rust (ya lo tienes)

Verificar:

```bash
rustc --version
cargo --version
```

### 0.3 Instalar sqlx-cli

Herramienta de linea de comandos para manejar migraciones de PostgreSQL:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Verificar:

```bash
sqlx --version
```

### 0.4 (Opcional) Instalar psql

Para conectarte a PostgreSQL desde terminal. Si instalaste Docker, puedes usar el psql dentro del contenedor sin instalar nada:

```bash
docker exec -it pulse_postgres psql -U pulse -d pulse_protocol
```

---

## PARTE 1: LEVANTAR POSTGRESQL Y REDIS

### 1.1 Crear docker-compose.yml

Este archivo va en la RAIZ del proyecto (`Pulse_Protocol/docker-compose.yml`):

```yaml
services:
  postgres:
    image: postgres:16
    container_name: pulse_postgres
    environment:
      POSTGRES_USER: pulse
      POSTGRES_PASSWORD: pulse_secret
      POSTGRES_DB: pulse_protocol
    ports:
      - "5432:5432"
    volumes:
      - pulse_pgdata:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    container_name: pulse_redis
    ports:
      - "6379:6379"
    volumes:
      - pulse_redisdata:/data

volumes:
  pulse_pgdata:
  pulse_redisdata:
```

### 1.2 Levantar los servicios

```bash
# Desde la raiz del proyecto (Pulse_Protocol/)
docker compose up -d
```

Verificar que estan corriendo:

```bash
docker ps
```

Deberias ver `pulse_postgres` y `pulse_redis` en estado "Up".

### 1.3 Conectarte a PostgreSQL (verificar que funciona)

```bash
docker exec -it pulse_postgres psql -U pulse -d pulse_protocol
```

Ahora estas dentro de psql. Comandos basicos:

```sql
-- Ver tablas (no hay ninguna todavia)
\dt

-- Salir
\q
```

### 1.4 Comandos de Docker que vas a usar seguido

```bash
# Levantar servicios
docker compose up -d

# Apagar servicios (sin borrar datos)
docker compose down

# Apagar Y BORRAR datos (reset completo de la DB)
docker compose down -v

# Ver logs de postgres
docker logs pulse_postgres

# Ver logs de redis
docker logs pulse_redis
```

---

## PARTE 2: CREAR EL PROYECTO oracle/

### 2.1 Estructura de archivos

Crea esta estructura dentro de `Pulse_Protocol/oracle/`:

```
oracle/
├── Cargo.toml
├── .env
├── migrations/
│   ├── 20260213_001_create_users.sql
│   ├── 20260213_002_create_vaults.sql
│   ├── 20260213_003_create_beneficiaries.sql
│   ├── 20260213_004_create_documents.sql
│   ├── 20260213_005_create_document_encrypted_keys.sql
│   ├── 20260213_006_create_verifications.sql
│   ├── 20260213_007_create_user_models.sql
│   ├── 20260213_008_create_events.sql
│   └── 20260213_009_create_push_tokens.sql
└── src/
    ├── main.rs
    ├── config.rs
    ├── db/
    │   ├── mod.rs
    │   ├── postgres.rs
    │   └── redis.rs
    ├── models/
    │   ├── mod.rs
    │   ├── user.rs
    │   ├── vault.rs
    │   ├── beneficiary.rs
    │   ├── document.rs
    │   ├── verification.rs
    │   ├── user_model.rs
    │   └── event.rs
    ├── graphql/
    │   ├── mod.rs
    │   ├── schema.rs
    │   ├── types.rs
    │   ├── queries.rs
    │   ├── mutations.rs
    │   └── subscriptions.rs
    └── services/
        ├── mod.rs
        ├── auth.rs
        └── blockchain.rs   ← mock de lo que B2 implementa despues
```

### 2.2 Cargo.toml

```toml
[package]
name = "pulse-oracle"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web server
actix-web = "4"
actix-cors = "0.7"
actix-ws = "0.3"

# GraphQL
async-graphql = { version = "7", features = ["uuid", "chrono"] }
async-graphql-actix-web = "7"

# Database
sqlx = { version = "0.8", features = [
    "runtime-tokio",
    "tls-rustls",
    "postgres",
    "uuid",
    "chrono",
    "json",
    "migrate",
] }

# Redis
redis = { version = "0.25", features = ["tokio-comp", "aio"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Async runtime
tokio = { version = "1", features = ["full"] }

# Config
dotenvy = "0.15"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utils
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
jsonwebtoken = "9"
sha2 = "0.10"
hex = "0.4"
async-trait = "0.1"
futures = "0.3"
```

### 2.3 Archivo .env

Crea `oracle/.env`:

```env
DATABASE_URL=postgres://pulse:pulse_secret@localhost:5432/pulse_protocol
REDIS_URL=redis://127.0.0.1:6379
HOST=127.0.0.1
PORT=8080
JWT_SECRET=pulse_protocol_dev_secret_change_in_production
RUST_LOG=pulse_oracle=debug,actix_web=info
```

---

## PARTE 3: MIGRACIONES DE POSTGRESQL

Las migraciones son archivos SQL que crean las tablas. Se ejecutan en orden.

### 3.1 Crear las migraciones

Cada archivo SQL va en `oracle/migrations/`.

**`20260213_001_create_users.sql`:**

```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stellar_address VARCHAR(56) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    calibration_complete BOOLEAN NOT NULL DEFAULT FALSE,
    calibration_started_at TIMESTAMPTZ
);
```

**`20260213_002_create_vaults.sql`:**

```sql
CREATE TABLE vaults (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_vault_id BIGINT,
    owner_id UUID NOT NULL REFERENCES users(id),
    token_address VARCHAR(56) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'Active',
    balance BIGINT NOT NULL DEFAULT 0,
    escrow_contract_id VARCHAR(56),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vaults_owner ON vaults(owner_id);
```

**`20260213_003_create_beneficiaries.sql`:**

```sql
CREATE TABLE beneficiaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id UUID NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
    stellar_address VARCHAR(56) NOT NULL,
    percentage INT NOT NULL CHECK (percentage > 0 AND percentage <= 10000),
    claimed BOOLEAN NOT NULL DEFAULT FALSE,
    claimed_at TIMESTAMPTZ,
    UNIQUE(vault_id, stellar_address)
);

CREATE INDEX idx_beneficiaries_vault ON beneficiaries(vault_id);
```

**`20260213_004_create_documents.sql`:**

```sql
CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id),
    vault_id UUID REFERENCES vaults(id),
    ipfs_cid VARCHAR(100) NOT NULL,
    doc_hash VARCHAR(64) NOT NULL,
    doc_type VARCHAR(30) NOT NULL,
    is_encrypted BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB,
    contract_doc_id BIGINT,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_documents_vault ON documents(vault_id);
CREATE INDEX idx_documents_owner ON documents(owner_id);
```

**`20260213_005_create_document_encrypted_keys.sql`:**

```sql
CREATE TABLE document_encrypted_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    beneficiary_address VARCHAR(56) NOT NULL,
    encrypted_key TEXT NOT NULL,
    revealed BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(document_id, beneficiary_address)
);
```

**`20260213_006_create_verifications.sql`:**

```sql
CREATE TABLE verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    score INT NOT NULL CHECK (score >= 0 AND score <= 10000),
    source VARCHAR(30) NOT NULL,
    face_match_score INT,
    face_liveness_score INT,
    fingerprint_frequency INT,
    fingerprint_consistency INT,
    time_of_day_normality INT,
    typing_pattern_match INT,
    app_usage_match INT,
    movement_pattern_match INT,
    days_since_last_verify INT,
    session_behavior INT,
    perceptron_output INT,
    on_chain_tx_hash VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_verifications_user_created
    ON verifications(user_id, created_at DESC);
```

**`20260213_007_create_user_models.sql`:**

```sql
CREATE TABLE user_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id),
    weights JSONB NOT NULL,
    bias VARCHAR(30) NOT NULL,
    version INT NOT NULL DEFAULT 1,
    calibration_complete BOOLEAN NOT NULL DEFAULT FALSE,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**`20260213_008_create_events.sql`:**

```sql
CREATE TABLE events (
    id BIGSERIAL PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    aggregate_type VARCHAR(30) NOT NULL,
    aggregate_id UUID NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    on_chain_tx_hash VARCHAR(64)
);

CREATE INDEX idx_events_aggregate
    ON events(aggregate_type, aggregate_id, created_at);
CREATE INDEX idx_events_type
    ON events(event_type, created_at);
```

**`20260213_009_create_push_tokens.sql`:**

```sql
CREATE TABLE push_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    device_token TEXT NOT NULL,
    platform VARCHAR(10) NOT NULL CHECK (platform IN ('android', 'ios')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, device_token)
);
```

### 3.2 Ejecutar las migraciones

Desde la carpeta `oracle/`:

```bash
cd oracle
sqlx migrate run --source migrations
```

Si necesitas verificar que se crearon las tablas:

```bash
docker exec -it pulse_postgres psql -U pulse -d pulse_protocol -c "\dt"
```

Resultado esperado:

```
              List of relations
 Schema |          Name                | Type  | Owner
--------+------------------------------+-------+-------
 public | beneficiaries                | table | pulse
 public | document_encrypted_keys      | table | pulse
 public | documents                    | table | pulse
 public | events                       | table | pulse
 public | push_tokens                  | table | pulse
 public | user_models                  | table | pulse
 public | users                        | table | pulse
 public | vaults                       | table | pulse
 public | verifications                | table | pulse
```

### 3.3 Comandos de PostgreSQL que necesitaras

```bash
# Entrar a psql
docker exec -it pulse_postgres psql -U pulse -d pulse_protocol

# Ya dentro de psql:

\dt                          -- listar todas las tablas
\d users                     -- ver estructura de la tabla users
\d vaults                    -- ver estructura de la tabla vaults

SELECT * FROM users;         -- ver todos los usuarios
SELECT * FROM vaults;        -- ver todos los vaults
SELECT COUNT(*) FROM events; -- contar eventos

-- Borrar TODOS los datos (util para reset en desarrollo)
TRUNCATE users CASCADE;

-- Insertar un usuario de prueba
INSERT INTO users (stellar_address)
VALUES ('GBTEST1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ123456');

\q                           -- salir de psql
```

### 3.4 Si algo sale mal con las migraciones

```bash
# Reset completo: borra la DB y vuelve a crearla
docker compose down -v
docker compose up -d
# Espera 2-3 segundos a que postgres inicie
sqlx migrate run --source migrations
```

---

## PARTE 4: CODIGO FUENTE — CONFIGURACION

### 4.1 `src/config.rs`

```rust
use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            host: env::var("HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a number"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
        }
    }
}
```

---

## PARTE 5: CODIGO FUENTE — CONEXION A BASE DE DATOS

### 5.1 `src/db/mod.rs`

```rust
pub mod postgres;
pub mod redis;
```

### 5.2 `src/db/postgres.rs`

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Crea un pool de conexiones a PostgreSQL.
/// El pool reutiliza conexiones automaticamente.
pub async fn create_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Failed to create PostgreSQL pool")
}

/// Ejecuta las migraciones pendientes.
/// Llama esto al arrancar el servidor.
pub async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");
}
```

### 5.3 `src/db/redis.rs`

```rust
use redis::Client;

/// Crea un cliente Redis.
pub fn create_client(redis_url: &str) -> Client {
    Client::open(redis_url).expect("Failed to create Redis client")
}

/// Helper: guardar un valor con TTL (en segundos).
pub async fn set_with_ttl(
    client: &Client,
    key: &str,
    value: &str,
    ttl_seconds: u64,
) -> redis::RedisResult<()> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("EX")
        .arg(ttl_seconds)
        .query_async(&mut conn)
        .await
}

/// Helper: leer un valor.
pub async fn get(client: &Client, key: &str) -> redis::RedisResult<Option<String>> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    redis::cmd("GET").arg(key).query_async(&mut conn).await
}

/// Helper: incrementar un counter (para rate limiting).
pub async fn incr(client: &Client, key: &str, ttl_seconds: u64) -> redis::RedisResult<i64> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    let count: i64 = redis::cmd("INCR").arg(key).query_async(&mut conn).await?;
    if count == 1 {
        redis::cmd("EXPIRE")
            .arg(key)
            .arg(ttl_seconds)
            .query_async(&mut conn)
            .await?;
    }
    Ok(count)
}

/// Helper: borrar una clave.
pub async fn del(client: &Client, key: &str) -> redis::RedisResult<()> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    redis::cmd("DEL").arg(key).query_async(&mut conn).await
}
```

---

## PARTE 6: CODIGO FUENTE — MODELOS

Estos structs representan las filas de las tablas de PostgreSQL.

### 6.1 `src/models/mod.rs`

```rust
pub mod user;
pub mod vault;
pub mod beneficiary;
pub mod document;
pub mod verification;
pub mod user_model;
pub mod event;
```

### 6.2 `src/models/user.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub stellar_address: String,
    pub created_at: DateTime<Utc>,
    pub calibration_complete: bool,
    pub calibration_started_at: Option<DateTime<Utc>>,
}

impl User {
    /// Buscar usuario por ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// Buscar usuario por direccion Stellar.
    pub async fn find_by_address(pool: &PgPool, address: &str) -> sqlx::Result<Option<Self>> {
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
    pub async fn complete_calibration(pool: &PgPool, id: Uuid) -> sqlx::Result<()> {
        sqlx::query("UPDATE users SET calibration_complete = TRUE WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
```

### 6.3 `src/models/vault.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vault {
    pub id: Uuid,
    pub contract_vault_id: Option<i64>,
    pub owner_id: Uuid,
    pub token_address: String,
    pub status: String,
    pub balance: i64,
    pub escrow_contract_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_synced_at: DateTime<Utc>,
}

impl Vault {
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Self>> {
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
        contract_vault_id: Option<i64>,
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
```

### 6.4 `src/models/beneficiary.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Beneficiary {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub stellar_address: String,
    pub percentage: i32,
    pub claimed: bool,
    pub claimed_at: Option<DateTime<Utc>>,
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
```

### 6.5 `src/models/document.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub vault_id: Option<Uuid>,
    pub ipfs_cid: String,
    pub doc_hash: String,
    pub doc_type: String,
    pub is_encrypted: bool,
    pub metadata: Option<serde_json::Value>,
    pub contract_doc_id: Option<i64>,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentEncryptedKey {
    pub id: Uuid,
    pub document_id: Uuid,
    pub beneficiary_address: String,
    pub encrypted_key: String,
    pub revealed: bool,
}

impl Document {
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_vault(pool: &PgPool, vault_id: Uuid) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM documents WHERE vault_id = $1 ORDER BY registered_at DESC",
        )
        .bind(vault_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_owner(pool: &PgPool, owner_id: Uuid) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM documents WHERE owner_id = $1 ORDER BY registered_at DESC",
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        owner_id: Uuid,
        ipfs_cid: &str,
        doc_hash: &str,
        doc_type: &str,
        is_encrypted: bool,
        metadata: Option<serde_json::Value>,
    ) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            r#"INSERT INTO documents
               (owner_id, ipfs_cid, doc_hash, doc_type, is_encrypted, metadata)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(owner_id)
        .bind(ipfs_cid)
        .bind(doc_hash)
        .bind(doc_type)
        .bind(is_encrypted)
        .bind(metadata)
        .fetch_one(pool)
        .await
    }

    pub async fn link_to_vault(pool: &PgPool, doc_id: Uuid, vault_id: Uuid) -> sqlx::Result<()> {
        sqlx::query("UPDATE documents SET vault_id = $1 WHERE id = $2")
            .bind(vault_id)
            .bind(doc_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl DocumentEncryptedKey {
    pub async fn store(
        pool: &PgPool,
        document_id: Uuid,
        beneficiary_address: &str,
        encrypted_key: &str,
    ) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            r#"INSERT INTO document_encrypted_keys
               (document_id, beneficiary_address, encrypted_key)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(document_id)
        .bind(beneficiary_address)
        .bind(encrypted_key)
        .fetch_one(pool)
        .await
    }

    pub async fn find_for_beneficiary(
        pool: &PgPool,
        document_id: Uuid,
        beneficiary_address: &str,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>(
            r#"SELECT * FROM document_encrypted_keys
               WHERE document_id = $1 AND beneficiary_address = $2"#,
        )
        .bind(document_id)
        .bind(beneficiary_address)
        .fetch_optional(pool)
        .await
    }
}
```

### 6.6 `src/models/verification.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Verification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub score: i32,
    pub source: String,
    pub face_match_score: Option<i32>,
    pub face_liveness_score: Option<i32>,
    pub fingerprint_frequency: Option<i32>,
    pub fingerprint_consistency: Option<i32>,
    pub time_of_day_normality: Option<i32>,
    pub typing_pattern_match: Option<i32>,
    pub app_usage_match: Option<i32>,
    pub movement_pattern_match: Option<i32>,
    pub days_since_last_verify: Option<i32>,
    pub session_behavior: Option<i32>,
    pub perceptron_output: Option<i32>,
    pub on_chain_tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Verification {
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
    pub async fn latest_score(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Option<i32>> {
        let row: Option<(i32,)> = sqlx::query_as(
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
        face_match_score: Option<i32>,
        face_liveness_score: Option<i32>,
        fingerprint_frequency: Option<i32>,
        fingerprint_consistency: Option<i32>,
        time_of_day_normality: Option<i32>,
        typing_pattern_match: Option<i32>,
        app_usage_match: Option<i32>,
        movement_pattern_match: Option<i32>,
        days_since_last_verify: Option<i32>,
        session_behavior: Option<i32>,
        perceptron_output: Option<i32>,
        on_chain_tx_hash: Option<&str>,
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
```

### 6.7 `src/models/user_model.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub weights: serde_json::Value,
    pub bias: String,
    pub version: i32,
    pub calibration_complete: bool,
    pub last_updated: DateTime<Utc>,
}

impl UserModel {
    pub async fn find_by_user(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>("SELECT * FROM user_models WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
    }

    pub async fn upsert(
        pool: &PgPool,
        user_id: Uuid,
        weights: serde_json::Value,
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
```

### 6.8 `src/models/event.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: i64,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub payload: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub on_chain_tx_hash: Option<String>,
}

impl Event {
    /// Registrar un evento en el event store.
    pub async fn record(
        pool: &PgPool,
        event_type: &str,
        aggregate_type: &str,
        aggregate_id: Uuid,
        payload: serde_json::Value,
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
```

---

(Continuado en GUIA_B1_BACKEND_P2.md)
