# Guia B1 — Servidor + API GraphQL + PostgreSQL + Autenticacion

**Rol:** B1 — Backend principal
**Directorio de trabajo:** `oracle/` y `src/`
**Fecha:** Febrero 2026
**Referencia:** `PLAN_DESARROLLO_MVP.md` secciones 4, 5, 6, Sprint 1 B1, Sprint 2 B1

---

## Resumen del Rol

B1 monta la infraestructura del servidor y la API GraphQL. Con el scope reducido del MVP (4 tablas, sin Redis, sin subscriptions), debe terminar rapido y puede apoyar a B2 con wrappers de contratos.

---

## Estado Actual vs MVP — Que hay que hacer

### Leyenda

- HECHO = ya existe y coincide con el MVP, no tocar
- MODIFICAR = existe pero no coincide con el spec MVP, hay que cambiarlo
- CREAR = no existe, hay que hacerlo desde cero
- BORRAR = existe pero NO es parte del MVP, hay que eliminarlo

---

## SPRINT 1

### Tarea 1: Proyecto Rust del backend — HECHO

Los archivos base ya existen y funcionan. No hay que crear nada nuevo aqui.

| Archivo | Estado | Notas |
|---|---|---|
| `oracle/Cargo.toml` | HECHO | Tiene todas las deps necesarias. Las extras (redis, actix-ws) no molestan por ahora |
| `src/main.rs` | MODIFICAR | Funciona, pero hay que quitar refs a Redis y agregar auth middleware (ver Tarea 4) |
| `src/config.rs` | HECHO | Lee DATABASE_URL, REDIS_URL, HOST, PORT, JWT_SECRET desde env |
| `oracle/.env` | HECHO | Configurado correctamente (puerto 5433 para evitar conflicto con Postgres local) |

**Accion:** No crear archivos nuevos. Solo modificar `main.rs` cuando se implemente auth.

---

### Tarea 2: Base de Datos PostgreSQL — MODIFICAR

El MVP especifica **4 tablas** con un schema simplificado. Actualmente hay **9 migraciones** con un schema del plan completo. Hay que reemplazarlas.

#### Que hay que cambiar:

**Migraciones a BORRAR** (no son parte del MVP):

| Migracion | Tabla | Razon |
|---|---|---|
| `20260213_004_create_documents.sql` | documents | Sin sistema de documentos en MVP |
| `20260213_005_create_document_encrypted_keys.sql` | document_encrypted_keys | Sin sistema de documentos en MVP |
| `20260213_007_create_user_models.sql` | user_models | Sin calibracion de perceptron en MVP |
| `20260213_008_create_events.sql` | events | Sin Event Store en MVP |
| `20260213_009_create_push_tokens.sql` | push_tokens | Sin push notifications en MVP |

**Migraciones a MODIFICAR** (existen pero el schema no coincide con MVP):

#### `001_create_users.sql` — Simplificar

```
ACTUAL (plan completo):                     MVP (lo que debe quedar):
─────────────────────────                   ─────────────────────────
id UUID PK                                  id UUID PK
stellar_address VARCHAR(56) UNIQUE          stellar_address VARCHAR(56) UNIQUE
created_at TIMESTAMPTZ                      created_at TIMESTAMPTZ
calibration_complete BOOLEAN        ← BORRAR
calibration_started_at TIMESTAMPTZ  ← BORRAR
```

#### `002_create_vaults.sql` — Reestructurar

```
ACTUAL (plan completo):                     MVP (lo que debe quedar):
─────────────────────────                   ─────────────────────────
id UUID PK                                  id UUID PK
contract_vault_id BIGINT            ← CAMBIAR a: contract_id VARCHAR(56) UNIQUE NOT NULL
owner_id UUID FK                            owner_id UUID FK
token_address VARCHAR(56)           ← BORRAR
status VARCHAR(20) DEFAULT 'Active'         status VARCHAR(20) DEFAULT 'active'
balance BIGINT DEFAULT 0            ← BORRAR
escrow_contract VARCHAR(56)         ← RENOMBRAR a: escrow_contract_id VARCHAR(56)
created_at TIMESTAMPTZ                      created_at TIMESTAMPTZ
last_synced_at TIMESTAMPTZ                  last_synced_at TIMESTAMPTZ
```

#### `003_create_beneficiaries.sql` — HECHO (sin cambios)

La tabla actual coincide con el MVP. No tocar.

#### `006_create_verifications.sql` — Simplificar drasticamente

```
ACTUAL (plan completo):                     MVP (lo que debe quedar):
─────────────────────────                   ─────────────────────────
id UUID PK                                  id UUID PK
user_id UUID FK                             user_id UUID FK
score INT CHECK(0-10000)                    score INT CHECK(0-10000)
source VARCHAR(30)                          source VARCHAR(30)
face_match_score INT                ← BORRAR
face_liveness_score INT             ← BORRAR
fingerprint_frequency INT           ← BORRAR
fingerprint_count INT               ← BORRAR
time_of_day_normality INT           ← BORRAR
typing_pattern_match INT            ← BORRAR
app_usage_match INT                 ← BORRAR
movement_pattern_match INT          ← BORRAR
days_since_last_verify INT          ← BORRAR
session_duration INT                ← BORRAR
perceptron_score INT                ← RENOMBRAR a: perceptron_output INT
hash VARCHAR(64)                    ← RENOMBRAR a: on_chain_tx_hash VARCHAR(64)
created_at TIMESTAMPTZ                      created_at TIMESTAMPTZ
```

#### Schema final MVP (referencia):

```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stellar_address VARCHAR(56) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE vaults (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id VARCHAR(56) UNIQUE NOT NULL,
    owner_id UUID NOT NULL REFERENCES users(id),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    escrow_contract_id VARCHAR(56),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE beneficiaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id UUID NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
    stellar_address VARCHAR(56) NOT NULL,
    percentage INT NOT NULL CHECK (percentage > 0 AND percentage <= 10000),
    claimed BOOLEAN NOT NULL DEFAULT FALSE,
    claimed_at TIMESTAMPTZ,
    UNIQUE(vault_id, stellar_address)
);

CREATE TABLE verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    score INT NOT NULL CHECK (score >= 0 AND score <= 10000),
    source VARCHAR(30) NOT NULL,
    perceptron_output INT,
    on_chain_tx_hash VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vaults_owner ON vaults(owner_id);
CREATE INDEX idx_beneficiaries_vault ON beneficiaries(vault_id);
CREATE INDEX idx_verifications_user_created ON verifications(user_id, created_at DESC);
```

#### Pasos concretos:

1. Borrar los 9 archivos de `oracle/migrations/`
2. Crear 1 migracion nueva con el schema MVP completo (o 4 migraciones separadas, una por tabla)
3. Recrear la base de datos: `docker compose down -v && docker compose up -d`
4. Verificar que `cargo run` levanta y las migraciones corren

---

### Tarea 3: Schema GraphQL base — MODIFICAR

El schema GraphQL actual no coincide con el MVP. Hay queries/mutations de mas y faltan las del MVP.

#### `src/graphql/types.rs` — Reestructurar

**BORRAR** estos tipos (no MVP):
- `Document` struct
- `DocumentType` enum
- `VerificationSource` enum (el MVP usa String para source)
- `AlertLevel` enum
- `TokenBalance` struct (se agrega diferente en MVP)
- `RegisterDocumentInput` struct
- `EncryptedKeyInput` struct

**MODIFICAR** estos tipos:

| Tipo | Cambio |
|---|---|
| `User` | Quitar `calibration_complete` y `calibration_started_at` |
| `Vault` | Quitar `contract_vault_id`, `token_address`, `balance`. Agregar `contract_id: String`, `owner: String` (stellar address, no UUID). Agregar campo `beneficiaries` resuelto y `balance: Vec<TokenBalance>` |
| `VerificationRecord` | Quitar todos los scores individuales. Dejar solo: `id`, `user_id`, `score`, `source`, `perceptron_output`, `on_chain_tx_hash`, `created_at` |
| `LivenessData` | Cambiar a: `score: Int!`, `lastVerified: DateTime!`, `totalVerifications: Int!` |
| `VerificationInput` | Simplificar a: `perceptronOutput: Int!`, `source: String!` (quitar todas las features individuales) |

**CREAR** estos tipos nuevos:

```rust
// Resultado de transacciones on-chain
pub struct TransactionResult {
    pub success: bool,
    pub tx_hash: Option<String>,
    pub message: String,
}

// Resultado de verificacion
pub struct VerificationResult {
    pub score: i32,
    pub tx_hash: Option<String>,
    pub vault_status: Option<String>,
}

// Resultado de checkin de emergencia
pub struct CheckinResult {
    pub success: bool,
    pub new_score: i32,
    pub tx_hash: Option<String>,
}

// Resultado de claim
pub struct ClaimResult {
    pub success: bool,
    pub amount_received: String,
    pub tx_hash: Option<String>,
}

// Input para crear vault (MVP: solo token)
pub struct CreateVaultInput {
    pub token: String,
    pub initial_deposit: Option<String>,
}

// Enum de estados del vault
pub enum VaultStatus {
    Active,
    Alert,
    GracePeriod,
    Triggered,
    Distributed,
}
```

#### `src/graphql/schema.rs` — Reescribir resolvers

**Queries actuales vs MVP:**

| Actual | MVP | Accion |
|---|---|---|
| `user(stellar_address)` | — | BORRAR |
| `vault(id)` | `vault(id)` | MODIFICAR (usar wrappers de B2s para datos on-chain) |
| `user_vaults(stellar_address)` | `myVaults` | MODIFICAR — ya no recibe address, usa auth context |
| `vault_beneficiaries(vault_id)` | `beneficiaries(vaultId)` | RENOMBRAR |
| `user_documents(stellar_address)` | — | BORRAR |
| `user_verifications(stellar_address)` | — | BORRAR |
| `liveness_status(stellar_address)` | `livenessScore(userId)` | MODIFICAR — cambia parametro y tipo de retorno |

**Mutations actuales vs MVP:**

| Actual | MVP | Accion |
|---|---|---|
| `create_user(stellar_address)` | — | BORRAR (el usuario se crea al autenticarse) |
| `create_vault(input)` | `createVault(input)` | MODIFICAR — invocar contrato on-chain + crear escrow C2C |
| — | `deposit(vaultId, amount, token)` | CREAR |
| `add_beneficiary(vault_id, input)` | `setBeneficiaries(vaultId, beneficiaries)` | REEMPLAZAR — es batch, no individual |
| `register_document(...)` | — | BORRAR |
| `submit_verification(input)` | `submitVerification(input)` | MODIFICAR — simplificar input, publicar on-chain |
| — | `emergencyCheckin` | CREAR |
| — | `claimInheritance(vaultId)` | CREAR |
| — | `forceTransition(vaultId, newStatus)` | CREAR (admin/demo) |

**Para Sprint 1:** Los resolvers pueden retornar datos mock / solo persistir en DB sin llamar on-chain. Sprint 2 los conecta con los wrappers de B2s.

---

### Tarea 4: Autenticacion simple — CREAR (no existe)

Este es el componente mas grande que falta. No hay nada implementado.

#### Crear `src/auth.rs`:

**Flujo segun el plan:**
1. La app genera mensaje: `"pulse-auth:{timestamp}"`
2. Firma con private key Stellar
3. Envia: `{ stellar_address, message, signature }`
4. Backend verifica firma ed25519 con la public key
5. Genera token random, lo almacena en `HashMap` en memoria
6. Retorna el token
7. Cada request lleva `Authorization: Bearer {token}`

**Que implementar:**

```
src/auth.rs:
├── struct AuthPayload { stellar_address, message, signature }
├── struct AuthResponse { token, stellar_address }
├── struct SessionStore (HashMap<String, SessionData> envuelto en Arc<RwLock>)
├── fn verify_stellar_signature(address, message, signature) -> bool
├── fn auth_handler(payload) -> AuthResponse   // POST /auth
├── struct AuthMiddleware                       // Extrae Bearer token, valida sesion
└── struct AuthenticatedUser { stellar_address, user_id }  // Se inyecta en Context
```

**Dependencias necesarias** (ya estan en Cargo.toml):
- `sha2` — para hashing
- `hex` — para encoding
- `jsonwebtoken` — NO se usa, auth es con token random (pero ya esta instalado, no molesta)

**Dependencia que FALTA:**
- `ed25519-dalek` o `stellar-strkey` — para verificar firmas Stellar ed25519. Agregar a `oracle/Cargo.toml`

**Modificar `src/main.rs`:**
- Agregar `mod auth;`
- Crear instancia de `SessionStore` compartida
- Agregar ruta `POST /auth` → `auth::auth_handler`
- Aplicar middleware de auth a las rutas de `/graphql`
- Inyectar `AuthenticatedUser` en el GraphQL context

---

### Tarea 5: Health check + Docker Compose — HECHO

| Archivo | Estado | Notas |
|---|---|---|
| `GET /health` en `main.rs` | HECHO | Verifica PostgreSQL y Redis. Para MVP podria verificar solo Postgres, pero funciona asi |
| `docker-compose.yml` | HECHO | Tiene postgres (puerto 5433) + redis. Redis no se usa en MVP pero no molesta |
| `.env.example` | HECHO | Existe en la raiz |
| `database/init.sql` | HECHO | Crea extensiones pgcrypto y uuid-ossp |

---

### Modelos (`src/models/`) — MODIFICAR

Los modelos actuales mapean al schema del plan completo. Hay que simplificarlos para MVP.

| Archivo | Accion | Detalle |
|---|---|---|
| `src/models/mod.rs` | MODIFICAR | Quitar exports de document, user_model, event |
| `src/models/user.rs` | MODIFICAR | Quitar campos `calibration_complete`, `calibration_started_at` y metodo `complete_calibration`. Agregar metodo `find_or_create` simplificado |
| `src/models/vault.rs` (dentro de user.rs) | MODIFICAR | Cambiar `contract_vault_id: Option<i64>` a `contract_id: String`. Quitar `token_address`, `balance`. Renombrar `escrow_contract` a `escrow_contract_id` |
| `src/models/beneficiary.rs` | HECHO | El modelo coincide con MVP. Tiene `set_for_vault`, `can_claim`, `record_claim` — todos utiles |
| `src/models/documents.rs` | BORRAR | No existe en MVP |
| `src/models/verification.rs` | MODIFICAR | Quitar todos los scores individuales. Dejar solo `score`, `source`, `perceptron_output`, `on_chain_tx_hash`. Simplificar `create()` |
| `src/models/user_model.rs` | BORRAR | No existe en MVP |
| `src/models/event.rs` | BORRAR | No existe en MVP |

---

### Archivos a BORRAR de `src/`:

| Archivo | Razon |
|---|---|
| `src/db/redis.rs` | No se usa Redis en MVP |
| `src/models/documents.rs` | Sin sistema de documentos |
| `src/models/user_model.rs` | Sin calibracion de perceptron |
| `src/models/event.rs` | Sin Event Store |

Tambien quitar `pub mod redis;` de `src/db/mod.rs` y los imports de Redis de `src/main.rs`.

---

## SPRINT 2

### Tarea 1: Query resolvers reales

Conectar los resolvers con PostgreSQL + wrappers on-chain de B2s.

#### `vault(id: ID!) -> Vault`
1. Lee vault de PostgreSQL por ID
2. (Opcional) Enriquece con datos on-chain via wrapper de B2s: `get_vault(contract_id)`, `get_balance(contract_id)`
3. Resuelve campo `beneficiaries` con query a tabla beneficiaries
4. Retorna vault completo

#### `myVaults -> [Vault!]!`
1. Extrae `user_id` del `AuthenticatedUser` en el context (requiere auth middleware)
2. Lee vaults de PostgreSQL donde `owner_id = user_id`
3. Para cada vault, resuelve beneficiaries y balance
4. Retorna lista

#### `livenessScore(userId: ID!) -> LivenessData`
1. Lee ultima verificacion de tabla `verifications` para el usuario
2. Cuenta total de verificaciones
3. Retorna `{ score, lastVerified, totalVerifications }`

#### `beneficiaries(vaultId: ID!) -> [Beneficiary!]!`
1. Lee de tabla `beneficiaries` donde `vault_id = vaultId`
2. Retorna lista

---

### Tarea 2: Mutation resolvers reales

Cada mutation persiste en DB y (donde aplique) invoca contratos on-chain via wrappers de B2s.

#### `createVault(input: CreateVaultInput!) -> Vault!`
1. Extrae usuario autenticado del context
2. Invoca `create_vault()` on-chain via wrapper → obtiene `contract_id`
3. Invoca `create_escrow()` on-chain (C2C con Trustless Work) → obtiene `escrow_contract_id`
4. Persiste en tabla `vaults` con `contract_id`, `owner_id`, `escrow_contract_id`
5. Si hay `initialDeposit`, invoca `deposit()` y `fund_escrow()` on-chain
6. Retorna vault creado

#### `deposit(vaultId: ID!, amount: String!, token: String!) -> TransactionResult!`
1. Busca vault en DB, verifica que el owner sea el usuario autenticado
2. Invoca `deposit()` on-chain via wrapper
3. Invoca `fund_escrow()` on-chain via wrapper
4. Actualiza `last_synced_at` en DB
5. Retorna `TransactionResult` con tx_hash

#### `setBeneficiaries(vaultId: ID!, beneficiaries: [BeneficiaryInput!]!) -> [Beneficiary!]!`
1. Valida que los porcentajes sumen exactamente 10000
2. Verifica que el vault pertenece al usuario autenticado
3. Invoca `set_beneficiaries()` on-chain via wrapper
4. Usa `Beneficiary::set_for_vault()` para reemplazar beneficiarios en DB (ya implementado en modelo)
5. Retorna lista de beneficiarios creados

#### `submitVerification(input: VerificationInput!) -> VerificationResult!`
1. Valida que `perceptronOutput` este en rango [0, 10000]
2. Verifica que el usuario exista
3. Firma score con clave ed25519 del oraculo (B2 implementa la firma)
4. Invoca `submit_verification()` on-chain via wrapper → obtiene tx_hash
5. Persiste en tabla `verifications` con score, source, perceptron_output, on_chain_tx_hash
6. Retorna `VerificationResult` con score y tx_hash

#### `emergencyCheckin -> CheckinResult!`
1. Extrae usuario autenticado
2. Invoca `emergency_checkin()` on-chain via wrapper
3. Persiste verificacion con score = 10000 y source = 'emergency'
4. Retorna `CheckinResult`

#### `claimInheritance(vaultId: ID!) -> ClaimResult!`
1. Verifica que el vault esta en estado TRIGGERED
2. Verifica que el usuario autenticado es beneficiario del vault (usa `Beneficiary::can_claim()`)
3. Invoca `release_to_beneficiary()` on-chain (C2C con TW) → obtiene monto
4. Invoca `record_claim()` on-chain
5. Actualiza tabla `beneficiaries` con `claimed = TRUE` (usa `Beneficiary::record_claim()`)
6. Retorna `ClaimResult` con monto y tx_hash

#### `forceTransition(vaultId: ID!, newStatus: VaultStatus!) -> Vault!`
1. Verifica que el vault existe (esta mutation es de admin/demo, no necesita auth estricto)
2. Invoca `transition_status()` on-chain via wrapper
3. Si newStatus es TRIGGERED: invoca `approve_milestones()` on-chain
4. Actualiza tabla `vaults` con nuevo status
5. Retorna vault actualizado

---

## Dependencias con otros roles

```
Sprint 1: B1 trabaja mayormente independiente
  └── Solo necesita Docker corriendo (postgres)

Sprint 2: B1 depende de B2s
  └── B2s provee wrappers tipados en oracle/src/services/contracts/
      ├── vault.rs    → create_vault, deposit, get_vault, transition_status, create_escrow...
      ├── proof_of_life.rs → submit_verification, emergency_checkin
      └── beneficiary.rs   → set_beneficiaries, can_claim, record_claim
  └── B2 provee oracle/src/services/soroban.rs (cliente generico de Soroban RPC)
```

**Si B2s no tiene los wrappers listos al inicio de Sprint 2:**
- B1 puede crear stubs/mocks de los wrappers para no bloquearse
- Implementar resolvers con logica de DB completa + `todo!()` en las llamadas on-chain
- Reemplazar los stubs cuando B2s tenga los wrappers reales

---

## Orden recomendado de trabajo

### Sprint 1 (7 dias):

| Dia | Tarea | Detalle |
|---|---|---|
| 1 | Limpiar migraciones | Borrar las 9 migraciones, crear las 4 del MVP, recrear DB, verificar que compila |
| 1-2 | Simplificar modelos | Modificar structs en `src/models/` para que coincidan con el schema MVP |
| 2-3 | Simplificar GraphQL types | Reestructurar `src/graphql/types.rs` con tipos MVP |
| 3-4 | Reescribir resolvers | `src/graphql/schema.rs` con queries/mutations MVP (mock por ahora) |
| 4-5 | Implementar auth | Crear `src/auth.rs`, agregar middleware, ruta POST /auth |
| 5-6 | Integrar auth en main.rs | SessionStore, middleware, inyeccion en GraphQL context |
| 6-7 | Testing manual | Verificar todo desde GraphQL Playground con auth |

### Sprint 2 (8 dias):

| Dia | Tarea | Detalle |
|---|---|---|
| 1-2 | Conectar query resolvers | Reemplazar mocks con queries reales a DB |
| 2-4 | Conectar mutation resolvers | Integrar wrappers de B2s para llamadas on-chain |
| 4-5 | submitVerification + emergencyCheckin | Requiere publicador on-chain de B2 |
| 5-6 | claimInheritance + forceTransition | Requiere TW completo de B2 |
| 6-7 | Testing end-to-end | Probar flujo completo del demo |
| 7-8 | Bug fixes + pulido | Resolver issues encontrados en testing |

---

## Checklist final

### Sprint 1 — Entregable:
- [ ] `cargo run` levanta servidor en `localhost:8080`
- [ ] GraphQL Playground funcional en `/graphql/playground`
- [ ] 4 queries responden (datos mock o de DB)
- [ ] 7 mutations registradas en el schema (pueden retornar mock)
- [ ] `POST /auth` acepta firma Stellar y retorna token
- [ ] Requests a `/graphql` sin token retornan 401
- [ ] Requests con token valido pasan y resuelven queries/mutations
- [ ] Base de datos tiene 4 tablas con schema MVP
- [ ] Health check `GET /health` responde OK

### Sprint 2 — Entregable:
- [ ] Todas las queries retornan datos reales de PostgreSQL
- [ ] `createVault` crea vault on-chain + escrow C2C + persiste en DB
- [ ] `deposit` fondea on-chain + escrow
- [ ] `setBeneficiaries` valida porcentajes + persiste + on-chain
- [ ] `submitVerification` publica score on-chain + persiste con tx_hash
- [ ] `emergencyCheckin` resetea score a 10000 on-chain + persiste
- [ ] `claimInheritance` libera fondos via TW + registra claim
- [ ] `forceTransition` cambia estado on-chain + persiste
- [ ] El flujo completo del demo (seccion 13 del plan) funciona sin errores
