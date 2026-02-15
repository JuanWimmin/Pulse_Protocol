# PLAN DE DESARROLLO MVP - Pulse Protocol

## Equipo de 5 Desarrolladores

**Documento:** Plan de trabajo MVP para demo/hackathon
**Fecha:** Febrero 2026
**Base:** `Pulse_Protocol_Propuesta.pdf` + `PLAN_DESARROLLO.md` (plan completo) + estado actual del repositorio
**Objetivo:** Demostrar el ciclo completo de herencia criptografica end-to-end en testnet

---

## 1. Estado Actual del Proyecto

### Lo que YA existe (commits `f56b508` + `e632f17`):

| Componente | Estado | Detalle |
|---|---|---|
| **Vault Contract** | Completo | `create_vault`, `deposit`, `withdraw`, `set_beneficiaries`, `link_proof_of_life`, `transition_status` — 13 tests |
| **ProofOfLife Contract** | Completo | `register_model`, `submit_verification`, `update_model`, `emergency_checkin`, `link_vault` — 13 tests |
| **Beneficiary Contract** | Completo | `set_beneficiaries`, `can_claim`, `record_claim` — 11 tests |
| **DocumentRegistry Contract** | Completo (NO se usa en MVP) | `register_document`, `link_to_vault`, `store_encrypted_key`, `grant_access`, `verify_document` — 12 tests |
| **Integration Tests** | Completo | 3 tests: ciclo completo de herencia, recuperacion de emergencia, flujo documental |
| **CI/CD** | Completo | GitHub Actions: lint + test + build WASM |
| **Configuracion** | Completo | `environments.toml` (dev, testnet, mainnet), `rust-toolchain.toml` |

### Lo que FALTA para el MVP:

- **Capa 2:** Backend/Oraculo simplificado (Rust + Actix-web + async-graphql + PostgreSQL)
- **Capa 1:** App movil reducida (React Native + Perceptron TS + Wallet basica)
- **Integracion C2C** con Trustless Work (Escrow) — modificacion del Vault Contract
- **Deploy** de 3 contratos a testnet de Stellar
- **Flujo de demo** end-to-end funcional

### Lo que se EXCLUYE del MVP (queda para version futura):

- Sistema de documentos (DocumentRegistry no se deploya)
- IPFS y cifrado AES-256-GCM
- Redis (cache, jobs, rate limiting)
- Event Store (auditoria)
- Jobs en background (timeouts, sync, notifications, model_sync)
- Push notifications (FCM)
- GraphQL Subscriptions (WebSocket)
- Vision artificial real (YOLO, InsightFace, Liveness detection)
- Patrones de comportamiento (typing, movement, app usage)
- Calibracion del perceptron (2-4 semanas)
- Scripts de ML en Python
- Monitoreo (Prometheus, Grafana)

---

## 2. Asignacion de Roles

```
BACKEND (3 personas)
  B1 — Servidor Actix-web, API GraphQL, PostgreSQL, autenticacion
  B2 — Oraculo: publicador on-chain, integracion Soroban, Trustless Work C2C
  B2s (ex-B3) — Apoyo de B2: wrappers tipados de contratos, integracion claim

FRONTEND (2 personas)
  F1 — App movil: navegacion, pantallas, wallet Stellar, cliente GraphQL
  F2 — Perceptron TS, huella biometrica, flujo de verificacion simplificado
```

### Justificacion de la redistribucion:

- **B1** monta la infraestructura del servidor y la API. Con el scope reducido (4 tablas, sin Redis, sin subscriptions), termina rapido y puede apoyar a B2 con wrappers de contratos.
- **B2** tiene la tarea mas critica y riesgosa: integracion C2C con Trustless Work + publicador on-chain. Es el cuello de botella del proyecto.
- **B2s (ex-B3)** pierde su rol original (sin Event Store, sin Redis, sin Jobs, sin IPFS) y se convierte en apoyo dedicado de B2: implementa los wrappers tipados por contrato y ayuda con la integracion de Trustless Work.
- **F1** construye la experiencia de usuario: 9 pantallas con datos reales del backend.
- **F2** implementa el perceptron en TypeScript y el flujo de verificacion simplificado. Cuando termina, ayuda a F1 con pantallas.

---

## 3. Arquitectura MVP

```
┌─────────────────────────────────────────────────────────────────┐
│                    CAPA 1: CLIENTE MOVIL                        │
│                     (React Native)                              │
│                                                                 │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐                │
│  │  Huella    │  │ Perceptron │  │  Wallet    │                │
│  │ BiometricAPI│  │ TS Local  │  │  Stellar   │                │
│  │ (score)    │  │ σ(w·x+b)  │  │ (keypair)  │                │
│  └─────┬──────┘  └─────┬──────┘  └────────────┘                │
│        └───────────────┤                                        │
│                        ▼                                        │
│  ┌────────────────────────────────────────────────────────┐     │
│  │         HTTP Client (queries + mutations)              │     │
│  └────────────────────────────────────────────────────────┘     │
└───────────────────────────┬─────────────────────────────────────┘
                            │ HTTPS (GraphQL)
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                CAPA 2: BACKEND / ORACULO                        │
│                 (Rust + Actix-web)                               │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ async-graphql│  │  Agregador   │  │ Publicador   │          │
│  │  API (HTTP)  │  │   Simple     │  │  On-Chain    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐                                               │
│  │ PostgreSQL   │                                               │
│  │ (4 tablas)   │                                               │
│  └──────────────┘                                               │
└───────────────────────────┬─────────────────────────────────────┘
                            │ Soroban RPC
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│              CAPA 3: SMART CONTRACTS (Soroban)                  │
│                                                                 │
│  ┌──────────┐  ┌───────────┐  ┌────────────┐                   │
│  │  Vault   │  │ProofOfLife│  │Beneficiary │                   │
│  │ Contract │  │ Contract  │  │ Contract   │                   │
│  └────┬─────┘  └───────────┘  └────────────┘                   │
│       │                                                         │
│       │ Cross-Contract Invocation (C2C)                         │
│       ▼                                                         │
│  ┌────────────────────┐                                         │
│  │  Trustless Work    │                                         │
│  │  Escrow Contract   │                                         │
│  └────────────────────┘                                         │
└───────────────────────────┬─────────────────────────────────────┘
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                   CAPA 5: STELLAR NETWORK                       │
│                    Soroban Runtime                               │
│                    Testnet                                       │
└─────────────────────────────────────────────────────────────────┘
```

### Diferencias clave vs arquitectura completa:

- Sin Redis, sin Event Store, sin IPFS
- Sin GraphQL Subscriptions (sin WebSocket)
- Sin DocumentRegistry Contract (no se deploya)
- Vision artificial y patrones de comportamiento mockeados
- Perceptron con pesos hardcodeados (sin calibracion)
- Transiciones de estado manuales via mutation de admin (sin jobs de timeout)

---

## 4. Base de Datos (PostgreSQL)

### 4 tablas:

```sql
-- Usuarios
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stellar_address VARCHAR(56) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Vaults (cache local del estado on-chain)
CREATE TABLE vaults (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id VARCHAR(56) UNIQUE NOT NULL,
    owner_id UUID NOT NULL REFERENCES users(id),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    escrow_contract_id VARCHAR(56),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Beneficiarios (cache local)
CREATE TABLE beneficiaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id UUID NOT NULL REFERENCES vaults(id),
    stellar_address VARCHAR(56) NOT NULL,
    percentage INT NOT NULL CHECK (percentage > 0 AND percentage <= 10000),
    claimed BOOLEAN NOT NULL DEFAULT FALSE,
    claimed_at TIMESTAMPTZ,
    UNIQUE(vault_id, stellar_address)
);

-- Verificaciones (historial simplificado)
CREATE TABLE verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    score INT NOT NULL CHECK (score >= 0 AND score <= 10000),
    source VARCHAR(30) NOT NULL,
    perceptron_output INT,
    on_chain_tx_hash VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indices
CREATE INDEX idx_verifications_user_created ON verifications(user_id, created_at DESC);
CREATE INDEX idx_vaults_owner ON vaults(owner_id);
CREATE INDEX idx_beneficiaries_vault ON beneficiaries(vault_id);
```

---

## 5. Schema GraphQL MVP

### Queries (4):

```graphql
type Query {
    vault(id: ID!): Vault
    myVaults: [Vault!]!
    livenessScore(userId: ID!): LivenessData
    beneficiaries(vaultId: ID!): [Beneficiary!]!
}
```

### Mutations (7):

```graphql
type Mutation {
    # Vaults
    createVault(input: CreateVaultInput!): Vault!
    deposit(vaultId: ID!, amount: String!, token: String!): TransactionResult!

    # Beneficiarios
    setBeneficiaries(vaultId: ID!, beneficiaries: [BeneficiaryInput!]!): [Beneficiary!]!

    # Prueba de vida
    submitVerification(input: VerificationInput!): VerificationResult!
    emergencyCheckin: CheckinResult!

    # Claims
    claimInheritance(vaultId: ID!): ClaimResult!

    # Admin/Demo — forzar transicion de estado sin esperar timeouts
    forceTransition(vaultId: ID!, newStatus: VaultStatus!): Vault!
}
```

### Tipos principales:

```graphql
type Vault {
    id: ID!
    owner: String!
    status: VaultStatus!
    beneficiaries: [Beneficiary!]!
    balance: [TokenBalance!]!
    escrowContract: String
    createdAt: DateTime!
}

type Beneficiary {
    address: String!
    percentage: Int!
    claimed: Boolean!
    claimedAt: DateTime
}

type LivenessData {
    score: Int!
    lastVerified: DateTime!
    totalVerifications: Int!
}

type TokenBalance {
    token: String!
    amount: String!
}

enum VaultStatus {
    ACTIVE
    ALERT
    GRACE_PERIOD
    TRIGGERED
    DISTRIBUTED
}

input CreateVaultInput {
    token: String!
    initialDeposit: String
}

input BeneficiaryInput {
    address: String!
    percentage: Int!
}

input VerificationInput {
    perceptronOutput: Int!
    source: String!
}
```

### Lo que NO tiene el schema MVP:

- Subscriptions (ninguna)
- Queries de documentos
- Mutations de documentos, withdraw, updateModelWeights
- Tipos de documentos (Document, DocumentType, etc.)
- VerificationInput simplificado (solo perceptronOutput + source, sin las 10 features individuales)

---

## 6. Autenticacion

**Mecanismo:** Firma Stellar simple.

**Flujo:**
1. La app genera un mensaje con timestamp: `"pulse-auth:{timestamp}"`
2. Firma el mensaje con la private key Stellar del usuario
3. Envia al backend: `{ stellar_address, message, signature }`
4. El backend verifica la firma con la public key
5. Crea una sesion con un token random y lo retorna
6. La app envia el token en el header `Authorization: Bearer {token}` en cada request

**Sin JWT, sin refresh tokens, sin expiracion compleja.** Sesion en memoria del servidor. Si el servidor reinicia, el usuario se re-autentica (aceptable para un demo).

---

## 7. Modificaciones al Vault Contract (C2C Trustless Work)

El Vault Contract existente necesita funciones adicionales para invocacion C2C con Trustless Work:

### Funciones nuevas:

```rust
/// Crear escrow en Trustless Work al crear el vault
fn create_escrow(
    env: Env,
    vault_id: VaultId,
    trustless_work_factory: Address,
) -> Address; // Retorna escrow contract ID

/// Fondear el escrow con los depositos del vault
fn fund_escrow(
    env: Env,
    vault_id: VaultId,
    amount: i128,
    token: Address,
);

/// Aprobar milestones cuando el vault pasa a TRIGGERED
fn approve_milestones(
    env: Env,
    vault_id: VaultId,
);

/// Liberar fondos a un beneficiario via C2C
fn release_to_beneficiary(
    env: Env,
    vault_id: VaultId,
    beneficiary: Address,
) -> i128; // Retorna monto liberado
```

### Contratos que se deployan a testnet:

| Contrato | Estado | Accion |
|---|---|---|
| Vault | Modificado (+ funciones C2C) | **Deploy** |
| ProofOfLife | Sin cambios | **Deploy** |
| Beneficiary | Sin cambios | **Deploy** |
| DocumentRegistry | Sin cambios | **NO se deploya** (queda en repo para futuro) |

---

## 8. App Movil — Pantallas MVP

### 9 pantallas en 3 navigators:

```
AuthNavigator
  ├── WelcomeScreen            — Pantalla de bienvenida
  └── CreateWalletScreen       — Genera keypair Stellar, guarda en keychain

SimpleOnboardingNavigator
  └── BiometricSetupScreen     — Registra huella via BiometricPrompt/LocalAuth

MainNavigator
  ├── DashboardScreen          — Score actual + estado del vault + resumen
  ├── CreateVaultScreen        — Crear vault + deposito inicial
  ├── VaultDetailScreen        — Balance, estado, lista de beneficiarios
  ├── ManageBeneficiariesScreen — Agregar/editar beneficiarios con porcentajes
  ├── VerifyScreen             — Boton "Verificar vida" → huella + foto mock → score
  └── EmergencyCheckinScreen   — Verificacion estricta para resetear a ACTIVE

ClaimNavigator
  └── ClaimScreen              — Beneficiario reclama herencia + ve resultado
```

### Pantallas eliminadas del plan original:

- ImportWalletScreen (no se importan wallets en MVP)
- FaceRegistrationScreen (sin onboarding facial real)
- CalibrationInfoScreen (sin calibracion)
- VaultListScreen (1 vault por usuario en MVP, se ve en Dashboard)
- DocumentListScreen, RegisterDocumentScreen (sin documentos)
- BeneficiaryListScreen (fusionada en VaultDetail)
- SettingsScreen (no aporta al demo)
- ClaimResultScreen (fusionada en ClaimScreen)

### Dependencias mobile:

```json
{
  "@react-navigation/native": "latest",
  "@react-navigation/stack": "latest",
  "@apollo/client": "latest",
  "graphql": "latest",
  "zustand": "latest",
  "react-native-keychain": "latest",
  "react-native-biometrics": "latest",
  "@stellar/stellar-sdk": "latest"
}
```

### Lo que NO se instala:

- `react-native-tflite` (sin modelos de ML)
- `react-native-camera` (foto mock, no necesita camara real)
- Cualquier dependencia de TensorFlow

---

## 9. Perceptron MVP

### Implementacion en TypeScript:

```typescript
// mobile/src/services/perceptron/model.ts

// Pesos default para demo — producen score alto (~0.85) con features normales
// y score bajo (~0.15) con features de inactividad
const DEFAULT_WEIGHTS = [0.25, 0.20, 0.10, 0.10, 0.08, 0.07, 0.05, 0.05, 0.05, 0.05];
const DEFAULT_BIAS = -0.5;

class Perceptron {
    weights: number[];
    bias: number;

    constructor(weights = DEFAULT_WEIGHTS, bias = DEFAULT_BIAS) {
        this.weights = weights;
        this.bias = bias;
    }

    predict(features: number[]): number {
        const z = this.weights.reduce((sum, w, i) => sum + w * features[i], 0) + this.bias;
        return this.sigmoid(z);
    }

    private sigmoid(z: number): number {
        return 1 / (1 + Math.exp(-z));
    }
}
```

### Vector de features MVP (10 dimensiones):

| Feature | Fuente en MVP | Valor |
|---|---|---|
| x1: face_match_score | **Mock** (siempre 0.85) | Simulado |
| x2: face_liveness_score | **Mock** (siempre 0.80) | Simulado |
| x3: fingerprint_frequency | **Real** — BiometricPrompt | Evento de huella |
| x4: fingerprint_consistency | **Mock** (0.70) | Simulado |
| x5: time_of_day_normality | **Mock** (0.75) | Simulado |
| x6: typing_pattern_match | **Mock** (0.50) | Simulado |
| x7: app_usage_match | **Mock** (0.50) | Simulado |
| x8: movement_pattern_match | **Mock** (0.50) | Simulado |
| x9: days_since_last_verify | **Real** — calculado | Desde ultima verificacion |
| x10: session_behavior | **Mock** (0.60) | Simulado |

Para el demo, 2 features son reales (huella + dias desde ultima verificacion) y 8 son mock con valores razonables. El perceptron produce un score realista.

---

## 10. Oraculo MVP

### Agregador simplificado:

```
Recibe del movil:
├── perceptron_output: u32 (0-10000)
└── source: String

Validacion:
├── Verifica que perceptron_output este en rango [0, 10000]
└── Verifica que el usuario exista

Resultado:
└── Score validado listo para publicar
```

Sin rate limiting, sin deteccion de anomalias, sin consistencia temporal, sin historial.

### Publicador on-chain:

```
Score validado
│
▼
Firma score con clave ed25519 del oraculo
│
▼
Construye transaccion Soroban:
└── submit_verification(user, score, source, oracle_sig)
│
▼
Envia a Stellar Testnet via Soroban RPC
│
▼
Espera confirmacion → retorna tx_hash
│
▼
Persiste en tabla verifications
```

El publicador se mantiene completo — es el core del oraculo.

---

## 11. Infraestructura

### Docker Compose:

```yaml
# docker-compose.yml
services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_DB: pulse_protocol
      POSTGRES_USER: pulse
      POSTGRES_PASSWORD: pulse_dev
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data

  oracle:
    build: ./oracle
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://pulse:pulse_dev@postgres/pulse_protocol
      STELLAR_RPC_URL: https://soroban-testnet.stellar.org
      ORACLE_SECRET_KEY: ${ORACLE_SECRET_KEY}
    depends_on:
      - postgres

volumes:
  pgdata:
```

### Modo de trabajo por rol:

| Rol | Como corre el entorno |
|---|---|
| B1, B2, B2s (backend) | `docker compose up postgres` + `cargo run` nativo (mas rapido para iterar) |
| F1, F2 (frontend) | `docker compose up` completo (postgres + oracle como caja negra) |

### Acceso remoto:

```bash
# Si se necesita acceso remoto al backend
ngrok http 8080
```

### Variables de entorno (.env.example):

```
DATABASE_URL=postgres://pulse:pulse_dev@localhost/pulse_protocol
STELLAR_RPC_URL=https://soroban-testnet.stellar.org
ORACLE_SECRET_KEY=S...  # Secret key del oraculo (generar con stellar keys generate)
SERVER_PORT=8080
```

---

## 12. Plan por Sprints

---

### SPRINT 1 — Scaffolding + Contratos en Testnet + App Navegable (7 dias)

**Objetivo:** Backend responde, contratos en testnet, app se compila y navega entre pantallas.

---

#### B1 — Servidor + Base de Datos + API Base

**Directorio:** `oracle/`

**Tareas:**

1. **Inicializar proyecto Rust del backend**
   - Crear `oracle/Cargo.toml` con dependencias:
     - `actix-web` (HTTP server)
     - `async-graphql` + `async-graphql-actix-web` (GraphQL)
     - `sqlx` con feature `postgres` + `runtime-tokio` (DB async)
     - `serde` + `serde_json` (serialization)
     - `dotenv` (config)
     - `tokio` (async runtime)
     - `tracing` + `tracing-subscriber` (logging)
   - Crear `oracle/src/main.rs` con servidor Actix-web basico
   - Crear `oracle/src/config.rs` con lectura de variables de entorno

2. **Base de datos PostgreSQL**
   - Crear directorio `oracle/src/db/migrations/`
   - Implementar las 4 tablas: `users`, `vaults`, `beneficiaries`, `verifications`
   - Crear indices
   - Crear `oracle/src/db/postgres.rs` con pool de conexiones sqlx

3. **Schema GraphQL base**
   - Crear `oracle/src/graphql/schema.rs` con schema raiz
   - Crear `oracle/src/graphql/types.rs` con tipos: `Vault`, `Beneficiary`, `LivenessData`, `TokenBalance`, enums y inputs
   - Montar endpoint GraphQL en `/graphql` y playground en `/graphql/playground`
   - Los resolvers retornan datos mock por ahora (Sprint 2 los conecta a la DB)

4. **Autenticacion simple**
   - Crear `oracle/src/auth.rs` con:
     - Endpoint `POST /auth` que recibe `{ stellar_address, message, signature }`
     - Verifica firma ed25519 con la public key
     - Genera token random, lo almacena en memoria (HashMap)
     - Retorna el token
   - Middleware que extrae `Authorization: Bearer {token}` y lo valida

5. **Health check + Docker Compose**
   - Endpoint `GET /health` que verifica conexion a PostgreSQL
   - Crear `docker-compose.yml` con postgres + oracle
   - Crear `.env.example`

**Entregable:** `cargo run` levanta servidor en `localhost:8080` con GraphQL playground funcional y autenticacion basica.

---

#### B2 — Cliente Soroban + Trustless Work Research + Deploy

**Directorio:** `oracle/src/services/` + `contracts/` + `scripts/`

**Tareas:**

1. **Investigar Trustless Work (DIAS 1-3 — CRITICO)**
   - Leer documentacion del SDK de Trustless Work
   - Identificar la interfaz del contrato de escrow en Soroban
   - **Validar que C2C funciona:** hacer una invocacion de prueba desde un contrato de test
   - Documentar el mapping de roles (approver, service_provider, release_signer, etc.)
   - **DEADLINE DIA 3:** Si C2C no funciona, se cambia a backend-orquesta (el backend llama a TW directamente via Soroban RPC, sin C2C)

2. **Modificar Vault Contract para C2C**
   - Agregar funciones: `create_escrow`, `fund_escrow`, `approve_milestones`, `release_to_beneficiary`
   - Escribir tests para las funciones nuevas
   - Asegurarse de que los tests existentes siguen pasando

3. **Modulo de interaccion con Soroban**
   - Crear `oracle/src/services/soroban.rs`:
     - Conexion a Soroban RPC
     - Gestion de keypair del oraculo
     - Funcion generica `invoke_contract(contract_id, function_name, args) -> Result<Value>`

4. **Deploy a testnet**
   - Crear script `scripts/deploy_testnet.sh` que:
     - Genera identidades con `stellar keys generate`
     - Compila los 3 contratos
     - Deploya cada contrato a testnet
     - Inicializa y vincula contratos entre si
     - Crea 2 cuentas de prueba con friendbot
     - Guarda los contract IDs en `deployed_contracts.json`

**Entregable:** Trustless Work validado (C2C o fallback). 3 contratos en testnet. Modulo basico de interaccion con Soroban.

---

#### B2s (ex-B3) — Wrappers Tipados de Contratos

**Directorio:** `oracle/src/services/contracts/`

**Tareas:**

1. **Wrappers tipados por contrato** (usa el modulo generico de B2):
   - Crear `oracle/src/services/contracts/vault.rs`:
     - `create_vault(owner, token) -> VaultId`
     - `deposit(vault_id, amount, token)`
     - `get_vault(vault_id) -> VaultInfo`
     - `get_status(vault_id) -> VaultStatus`
     - `get_balance(vault_id) -> i128`
     - `transition_status(vault_id, new_status)`
     - `create_escrow(vault_id, tw_factory) -> Address`
     - `fund_escrow(vault_id, amount, token)`
     - `approve_milestones(vault_id)`
     - `release_to_beneficiary(vault_id, beneficiary) -> i128`
   - Crear `oracle/src/services/contracts/proof_of_life.rs`:
     - `submit_verification(user, score, source, signature)`
     - `get_liveness_score(user) -> u32`
     - `emergency_checkin(user)`
   - Crear `oracle/src/services/contracts/beneficiary.rs`:
     - `set_beneficiaries(vault_id, beneficiaries)`
     - `can_claim(vault_id, claimer) -> bool`
     - `record_claim(vault_id, claimer) -> u32`

2. **Modelos de datos (structs Rust)**
   - Crear `oracle/src/models/` con structs que mapean las 4 tablas SQL:
     - `user.rs`, `vault.rs`, `verification.rs`
   - Derivar `sqlx::FromRow` para cada struct

**Entregable:** Wrappers funcionales que invocan cualquier funcion de los 3 contratos. Modelos de datos listos para resolvers.

---

#### F1 — Scaffold de la App Movil

**Directorio:** `mobile/`

**Tareas:**

1. **Inicializar proyecto React Native**
   - `npx react-native init PulseProtocol --template react-native-template-typescript`
   - Instalar dependencias: `@react-navigation/native`, `@react-navigation/stack`, `@apollo/client`, `graphql`, `zustand`, `react-native-keychain`, `react-native-biometrics`, `@stellar/stellar-sdk`
   - Configurar TypeScript estricto

2. **Sistema de navegacion (9 pantallas)**
   - AuthNavigator: WelcomeScreen, CreateWalletScreen
   - SimpleOnboardingNavigator: BiometricSetupScreen
   - MainNavigator: DashboardScreen, CreateVaultScreen, VaultDetailScreen, ManageBeneficiariesScreen, VerifyScreen, EmergencyCheckinScreen
   - ClaimNavigator: ClaimScreen
   - Cada pantalla es placeholder con titulo y texto descriptivo

3. **Apollo Client setup**
   - Crear `mobile/src/services/graphql/client.ts` con HTTP link
   - Crear `mobile/src/services/graphql/queries.ts`: `VAULT_QUERY`, `MY_VAULTS_QUERY`, `LIVENESS_SCORE_QUERY`, `BENEFICIARIES_QUERY`
   - Crear `mobile/src/services/graphql/mutations.ts`: `CREATE_VAULT`, `DEPOSIT`, `SET_BENEFICIARIES`, `SUBMIT_VERIFICATION`, `EMERGENCY_CHECKIN`, `CLAIM_INHERITANCE`, `FORCE_TRANSITION`

4. **Tipos TypeScript**
   - Crear `mobile/src/types/` con interfaces que espejan el schema GraphQL:
     - `vault.ts`, `beneficiary.ts`, `verification.ts`

5. **Store global (Zustand)**
   - `authStore.ts` — stellar address, isAuthenticated, token de sesion
   - `vaultStore.ts` — vault del usuario, estado actual

**Entregable:** App React Native que se compila, navega entre 9 pantallas placeholder, tiene Apollo Client configurado y tipos TypeScript listos.

---

#### F2 — Perceptron + Huella + Flujo de Verificacion

**Directorio:** `mobile/src/services/`

**Tareas:**

1. **Perceptron en TypeScript**
   - Crear `mobile/src/services/perceptron/model.ts`:
     - Clase `Perceptron` con `predict(features)` y `sigmoid(z)`
     - Pesos default hardcodeados
   - Escribir 1-2 tests basicos: features altas → score alto, features bajas → score bajo

2. **Modulo de huella dactilar**
   - Crear `mobile/src/services/biometrics/fingerprint.ts` usando `react-native-biometrics`:
     - `checkBiometricAvailability() -> { available, type }`
     - `authenticateWithBiometric() -> { success, timestamp }`

3. **Feature extractor simplificado**
   - Crear `mobile/src/services/perceptron/features.ts`:
     - `extractFeatures() -> number[10]`
     - Retorna 2 features reales (huella + dias desde ultima verificacion) + 8 mock con valores razonables
     - Los valores mock varian ligeramente (+/- 0.05 random) para que el score no sea siempre identico

4. **Flujo de verificacion**
   - Crear `mobile/src/services/verification.ts`:
     - `runVerification()`:
       1. Solicita huella via BiometricPrompt
       2. Extrae features (reales + mock)
       3. Ejecuta perceptron local → score
       4. Retorna score listo para enviar al backend

**Entregable:** Perceptron funcional en TS. Huella captura eventos reales. El flujo de verificacion produce un score coherente.

---

### SPRINT 2 — Todo Conectado End-to-End + Demo Funcional (8 dias)

**Objetivo:** El flujo completo funciona. Listo para demo.

---

#### B1 — Resolvers Reales + forceTransition

**Tareas:**

1. **Query resolvers** — Crear `oracle/src/graphql/queries.rs`:
   - `vault(id)` — Lee de PostgreSQL, con datos del contrato on-chain via wrapper
   - `myVaults` — Filtra vaults por `owner_id` del usuario autenticado
   - `livenessScore(userId)` — Lee ultima verificacion de tabla `verifications`
   - `beneficiaries(vaultId)` — Lee de tabla `beneficiaries`

2. **Mutation resolvers** — Crear `oracle/src/graphql/mutations.rs`:
   - `createVault(input)`:
     1. Invoca `create_vault()` on-chain
     2. Invoca `create_escrow()` on-chain (C2C con Trustless Work)
     3. Persiste en tabla `vaults`
     4. Retorna vault creado
   - `deposit(vaultId, amount, token)`:
     1. Invoca `deposit()` on-chain
     2. Invoca `fund_escrow()` on-chain
   - `setBeneficiaries(vaultId, beneficiaries)`:
     1. Valida que porcentajes sumen 10000
     2. Invoca `set_beneficiaries()` on-chain
     3. Persiste en tabla `beneficiaries`
   - `submitVerification(input)`:
     1. Valida score en rango [0, 10000]
     2. Firma con clave del oraculo
     3. Invoca `submit_verification()` on-chain
     4. Persiste en tabla `verifications`
     5. Retorna resultado con tx_hash
   - `emergencyCheckin`:
     1. Invoca `emergency_checkin()` on-chain
     2. Persiste verificacion con score 10000
   - `claimInheritance(vaultId)`:
     1. Verifica estado TRIGGERED
     2. Invoca `release_to_beneficiary()` on-chain (C2C con TW)
     3. Invoca `record_claim()` on-chain
     4. Actualiza tabla `beneficiaries`
   - `forceTransition(vaultId, newStatus)`:
     1. Invoca `transition_status()` on-chain
     2. Actualiza tabla `vaults`
     3. Si newStatus es TRIGGERED: invoca `approve_milestones()` on-chain

**Entregable:** Todas las queries y mutations responden con datos reales de PostgreSQL + interaccion on-chain.

---

#### B2 — Agregador + Publicador + Trustless Work Completo

**Tareas:**

1. **Agregador simple** — Crear `oracle/src/services/aggregator.rs`:
   - Recibe `perceptron_output` del movil
   - Valida rango [0, 10000]
   - Retorna score validado

2. **Publicador on-chain** — Crear `oracle/src/services/publisher.rs`:
   - Firma score con clave ed25519 del oraculo
   - Construye transaccion Soroban: `submit_verification(user, score, source, oracle_sig)`
   - Envia a testnet via Soroban RPC
   - Espera confirmacion
   - Retorna tx_hash

3. **Integracion Trustless Work completa**
   - Asegurar que el flujo C2C funciona end-to-end:
     - createVault → create_escrow en TW
     - deposit → fund_escrow en TW
     - TRIGGERED → approve_milestones en TW
     - claim → release_funds en TW
   - Probar con fondos de testnet reales

**Entregable:** El oraculo publica scores on-chain. Trustless Work funciona end-to-end via C2C.

---

#### B2s (ex-B3) — Integracion Claim + Apoyo B2

**Tareas:**

1. **Flujo de claim end-to-end**
   - Integrar: claim request → verificar beneficiario → release_funds via TW → actualizar DB
   - Probar con multiples beneficiarios (60/40 split)

2. **Apoyo general a B2**
   - Debugging de transacciones Soroban fallidas
   - Probar flujos edge-case: claim doble, claim sin TRIGGERED, etc.

3. **Script de datos demo**
   - Actualizar `scripts/deploy_testnet.sh` para incluir:
     - Cuentas pre-fondeadas para demo rapido
     - Vault de ejemplo ya creado con beneficiarios asignados (backup para demo)

**Entregable:** Claim funciona end-to-end con Trustless Work. Datos de demo listos.

---

#### F1 — Pantallas Funcionales con Datos Reales

**Tareas:**

1. **WelcomeScreen + CreateWalletScreen**
   - Genera keypair Stellar via `@stellar/stellar-sdk`
   - Guarda secret en `react-native-keychain`
   - Llama al endpoint de auth del backend

2. **DashboardScreen**
   - Muestra liveness score con indicador visual (verde > 70%, amarillo 30-70%, rojo < 30%)
   - Muestra estado del vault con badge de color
   - Boton "Verificar vida" que navega a VerifyScreen
   - Usa queries `MY_VAULTS_QUERY` + `LIVENESS_SCORE_QUERY`

3. **CreateVaultScreen**
   - Formulario: seleccion de token + monto de deposito
   - Llama mutations `CREATE_VAULT` + `DEPOSIT`

4. **VaultDetailScreen**
   - Muestra: balance, estado, lista de beneficiarios con porcentajes
   - Boton para ir a ManageBeneficiariesScreen

5. **ManageBeneficiariesScreen**
   - Input de direccion Stellar + porcentaje
   - Validacion: suma debe ser exactamente 100%
   - Llama mutation `SET_BENEFICIARIES`

6. **ClaimScreen**
   - Lista vaults donde el usuario es beneficiario en estado TRIGGERED
   - Boton "Reclamar herencia"
   - Muestra resultado: monto recibido + tx hash
   - Llama mutation `CLAIM_INHERITANCE`

**Entregable:** La app muestra datos reales del backend. Se puede crear vault, depositar, gestionar beneficiarios y reclamar herencia.

---

#### F2 — VerifyScreen + EmergencyCheckin + Apoyo F1

**Tareas:**

1. **VerifyScreen funcional**
   - Boton "Verificar vida":
     1. Solicita huella (BiometricPrompt)
     2. Muestra animacion de "procesando"
     3. Ejecuta `runVerification()` (perceptron local)
     4. Envia score al backend via mutation `SUBMIT_VERIFICATION`
     5. Muestra resultado: score + tx hash + estado del vault

2. **EmergencyCheckinScreen**
   - Solicita huella biometrica
   - Llama mutation `EMERGENCY_CHECKIN`
   - Muestra resultado: score reseteado a 10000, estado vuelve a ACTIVE
   - Solo accesible cuando el vault esta en ALERT o GRACE

3. **Apoyo a F1**
   - Ayudar con pantallas que F1 no haya terminado
   - Integrar el flujo de verificacion en DashboardScreen

**Entregable:** Verificacion de vida funciona end-to-end. Emergency check-in funciona.

---

## 13. Flujo del Demo

```
PREPARACION (antes del demo):
├── Contratos deployados en testnet
├── Backend corriendo (local o ngrok)
└── App instalada en 2 dispositivos (owner + beneficiario)

DEMO EN VIVO (~10 minutos):

1. CREAR CUENTA
   └── Abrir app → Crear wallet → Se genera direccion Stellar

2. REGISTRAR BIOMETRIA
   └── Registrar huella dactilar

3. CREAR VAULT
   ├── Crear vault con XLM como token
   ├── Se crea escrow en Trustless Work (C2C)
   └── Depositar fondos (ej: 100 XLM)

4. ASIGNAR BENEFICIARIOS
   └── Alice: 60%, Bob: 40%

5. VERIFICAR VIDA
   ├── Tocar "Verificar" → huella + perceptron
   ├── Score: 8523 (85.23%) → ACTIVE
   └── Mostrar tx en Stellar Explorer

6. SIMULAR FALLECIMIENTO
   ├── forceTransition → ALERT (score bajo)
   ├── forceTransition → GRACE_PERIOD (sin actividad)
   └── forceTransition → TRIGGERED (timeout 30 dias simulado)

7. BENEFICIARIO RECLAMA
   ├── Abrir app en otro dispositivo como Alice
   ├── Ver: vault TRIGGERED, puede reclamar 60%
   ├── Tocar "Reclamar herencia"
   ├── Fondos transferidos via Trustless Work escrow
   └── Mostrar tx en Stellar Explorer

8. (BONUS) EMERGENCY CHECK-IN
   ├── Mostrar que si el usuario ESTUVIERA vivo en GRACE
   ├── Puede hacer emergency check-in con huella
   └── Score se resetea a 10000, vault vuelve a ACTIVE
```

---

## 14. Testing

### Tests automatizados:

| Capa | Tests | Cobertura |
|---|---|---|
| **Contratos Soroban** | Tests existentes (49) + tests nuevos para C2C TW | Todas las funciones |
| **Perceptron TS** | 1-2 tests basicos | `predict()` con features altas → score alto, features bajas → score bajo |

### Testing manual:

- Correr el flujo completo del demo 2-3 veces antes de presentar
- Probar en dispositivo real (no solo emulador)
- Tener datos de backup pre-creados por si algo falla en vivo

---

## 15. Riesgos y Mitigaciones

| Riesgo | Prob. | Impacto | Mitigacion | Deadline |
|---|---|---|---|---|
| **Trustless Work C2C no funciona** | Media-Alta | Critico | Cambiar a backend-orquesta (no C2C) | **Dia 3 Sprint 1** |
| **Soroban RPC testnet inestable** | Media | Alto | Reintentos + video de respaldo del demo | — |
| **React Native + Stellar SDK problemas** | Media | Medio | Generar keypair en backend como fallback | Sprint 1 |
| **Demo tarda demasiado en vivo** | Baja | Medio | `forceTransition` + cuentas pre-creadas | Sprint 2 |
| **Miembro del equipo bloqueado** | Media | Medio | Roles con overlap: B2s cubre B2, F2 cubre F1 | — |

---

## 16. Diagrama de Dependencias

```
Sprint 1 (mayormente en paralelo):
  B1: Servidor + DB + API base
  B2: Soroban client + TW research + Deploy testnet + Modificar Vault Contract
  B2s: Wrappers tipados (depende de B2 para el cliente generico)
  F1: Scaffold mobile + Navegacion + Apollo + Types
  F2: Perceptron TS + Huella + Feature extractor

Sprint 2 (dependencias):
  B1 depende de: B2s (wrappers para conectar resolvers con on-chain)
  B2 no tiene dependencia (trabaja en agregador/publicador/TW)
  B2s depende de: B2 (para flujo TW completo)
  F1 depende de: B1 (para datos reales via GraphQL)
  F2 depende de: Sprint 1 F2 completo. Apoya a F1.
```

---

## 17. Definicion de "Hecho" por Sprint

| Sprint | Criterio de aceptacion |
|---|---|
| **Sprint 1** | Backend levanta con GraphQL playground. App navega entre pantallas. 3 contratos en testnet. Perceptron TS produce scores. TW validado (C2C o fallback decidido). |
| **Sprint 2** | El flujo completo del demo (seccion 13) funciona end-to-end sin errores. Se puede ejecutar frente a una audiencia. |

---

## 18. Convenciones del Equipo

### Git

- **Branching:** `feature/mvp-{dev}-{descripcion}` (ej: `feature/mvp-b1-graphql-schema`)
- **Commits:** Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`)
- **PRs:** Requerido para merge a `main`. Minimo 1 review.
- **Nunca** commitear `.env`, credenciales, API keys, o secret keys

### Codigo

- **Rust:** `snake_case`, documentar funciones publicas con `///`, `cargo fmt` + `cargo clippy` antes de cada commit
- **TypeScript:** `camelCase` para funciones/variables, `PascalCase` para componentes/tipos. `strict: true`. No usar `any`.
- **Scores:** `u32` en rango 0-10000 (representa 0.00%-100.00%)
- **Decimales Soroban:** Fixed-point con 6 decimales (multiplicar por 1_000_000)
- **Biometria:** NUNCA almacenar datos raw. Solo scores normalizados.

---

*Plan MVP generado: Febrero 2026*
*Base: Pulse_Protocol_Propuesta.pdf v1.0 + PLAN_DESARROLLO.md v1.0*
*Objetivo: Demo/Hackathon — ciclo completo de herencia criptografica en testnet*
