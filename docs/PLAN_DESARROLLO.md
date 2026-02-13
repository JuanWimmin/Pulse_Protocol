# PLAN DE DESARROLLO - Pulse Protocol

## Equipo de 5 Desarrolladores

**Documento:** Plan de trabajo detallado por sprints
**Fecha:** Febrero 2026
**Base:** `Pulse_Protocol_Propuesta.md` + estado actual del repositorio

---

## 1. Estado Actual del Proyecto

### Lo que YA existe (commits `f56b508` + `e632f17`):

| Componente | Estado | Detalle |
|---|---|---|
| **Vault Contract** | Completo | `create_vault`, `deposit`, `withdraw`, `set_beneficiaries`, `link_proof_of_life`, `transition_status` — 13 tests |
| **ProofOfLife Contract** | Completo | `register_model`, `submit_verification`, `update_model`, `emergency_checkin`, `link_vault` — 13 tests |
| **Beneficiary Contract** | Completo | `set_beneficiaries`, `can_claim`, `record_claim` — 11 tests |
| **DocumentRegistry Contract** | Completo | `register_document`, `link_to_vault`, `store_encrypted_key`, `grant_access`, `verify_document` — 12 tests |
| **Integration Tests** | Completo | 3 tests: ciclo completo de herencia, recuperacion de emergencia, flujo documental |
| **CI/CD** | Completo | GitHub Actions: lint + test + build WASM |
| **Configuracion** | Completo | `environments.toml` (dev, testnet, mainnet), `rust-toolchain.toml` |

### Lo que FALTA (Fases 2, 3 y 4 de la propuesta):

- **Capa 2:** Backend/Oraculo completo (Rust + Actix-web + async-graphql + PostgreSQL + Redis)
- **Capa 1:** App movil completa (React Native + TFLite + YOLO + InsightFace + Wallet)
- **Integracion C2C** con Trustless Work (Escrow)
- **Scripts ML** para entrenamiento/exportacion del perceptron
- **Deploy** a testnet de Stellar

---

## 2. Asignacion de Roles

```
BACKEND (3 personas)
  B1 — Infraestructura del servidor, API GraphQL, base de datos
  B2 — Oraculo: agregador de senales, publicador on-chain, integracion Soroban
  B3 — Jobs en background, Event Store, Redis, IPFS, notificaciones

FRONTEND (2 personas)
  F1 — App movil: navegacion, pantallas, wallet Stellar, cliente GraphQL
  F2 — Modulos de IA/biometria: vision artificial, huella, patrones, perceptron
```

### Justificacion de la division:

- **B1** se encarga de que el servidor exista, responda requests y persista datos. Sin esto, nadie mas puede probar nada.
- **B2** es el puente critico entre el backend y la blockchain. Es la logica mas delicada del sistema (firmas del oraculo, transacciones Soroban, validacion de scores).
- **B3** maneja todo lo que corre en background: workers de Redis, sincronizacion de estado on-chain, event sourcing y el cliente IPFS. Esto es independiente de la API que hace B1.
- **F1** construye toda la experiencia de usuario: onboarding, dashboard, vaults, documentos, beneficiarios, wallet. Es el "frontend clasico".
- **F2** trabaja exclusivamente en los modulos de IA y biometria que viven en el dispositivo. Esto es altamente especializado (TFLite, modelos pre-entrenados, sensores) y no se cruza con las pantallas de F1.

---

## 3. Plan por Sprints

---

### SPRINT 1 — Cimientos del Backend + Scaffold del Mobile

**Objetivo:** Levantar la estructura base del backend con un endpoint funcional y la app movil navegable con pantallas placeholder.

---

#### B1 — Servidor + Base de Datos

**Directorio:** `oracle/`

**Tareas:**

1. **Inicializar proyecto Rust del backend**
   - Crear `oracle/Cargo.toml` con dependencias:
     - `actix-web` (HTTP server)
     - `async-graphql` + `async-graphql-actix-web` (GraphQL)
     - `sqlx` con feature `postgres` + `runtime-tokio` (DB async)
     - `redis` (cache)
     - `serde` + `serde_json` (serialization)
     - `dotenv` (config)
     - `tokio` (async runtime)
     - `tracing` + `tracing-subscriber` (logging)
   - Crear `oracle/src/main.rs` con servidor Actix-web basico
   - Crear `oracle/src/config.rs` con lectura de variables de entorno (`DATABASE_URL`, `REDIS_URL`, `STELLAR_RPC_URL`, `ORACLE_SECRET_KEY`)

2. **Esquema de base de datos PostgreSQL**
   - Crear directorio `oracle/src/db/migrations/`
   - Implementar las 7 tablas definidas en la propuesta (seccion 18.1):
     - `users` (id UUID, stellar_address, calibration_complete, calibration_started_at, created_at)
     - `vaults` (id UUID, contract_id, owner_id FK, status, escrow_contract_id, created_at, last_synced_at)
     - `beneficiaries` (id UUID, vault_id FK, stellar_address, percentage, claimed, claimed_at)
     - `documents` (id UUID, owner_id FK, vault_id FK, ipfs_cid, doc_hash, doc_type, is_encrypted, metadata JSONB, contract_doc_id)
     - `document_encrypted_keys` (id UUID, document_id FK, beneficiary_address, encrypted_key, revealed)
     - `verifications` (id UUID, user_id FK, score, source, 10 campos de features individuales, perceptron_output, on_chain_tx_hash, created_at)
     - `user_models` (id UUID, user_id FK UNIQUE, weights JSONB, bias, version, calibration_complete, last_updated)
   - Crear indices: `idx_verifications_user_created`, `idx_vaults_owner`, `idx_documents_vault`, `idx_beneficiaries_vault`
   - Crear `oracle/src/db/postgres.rs` con pool de conexiones sqlx

3. **Conexion Redis**
   - Crear `oracle/src/db/redis.rs` con pool de conexiones
   - Implementar funciones helper para las claves definidas en la propuesta (seccion 18.2):
     - `user:{id}:score` (cache liveness score, TTL 5min)
     - `user:{id}:session` (datos de sesion, TTL 24h)
     - `vault:{id}:status` (cache estado, TTL 5min)
     - `rate_limit:verify:{user_id}` (counter, TTL 1h)

4. **Schema GraphQL base**
   - Crear `oracle/src/graphql/schema.rs` con el schema raiz (Query + Mutation + Subscription vacios)
   - Crear `oracle/src/graphql/types.rs` con los tipos GraphQL mapeados del schema de la propuesta (seccion 17.2):
     - `Vault`, `Beneficiary`, `Document`, `LivenessData`, `VerificationRecord`, `TokenBalance`
     - Enums: `VaultStatus`, `DocumentType`, `VerificationSource`, `AlertLevel`
     - Inputs: `CreateVaultInput`, `BeneficiaryInput`, `RegisterDocumentInput`, `VerificationInput`, `EncryptedKeyInput`
   - Montar endpoint GraphQL en `/graphql` y playground en `/graphql/playground`

5. **Health check + Docker Compose**
   - Endpoint `GET /health` que verifica conexion a PostgreSQL y Redis
   - Crear `docker-compose.yml` en la raiz del proyecto con servicios: `postgres:16`, `redis:7`, `oracle` (build local)
   - Crear `.env.example` con todas las variables necesarias

**Entregable:** `cargo run` desde `oracle/` levanta un servidor en `localhost:8080` con GraphQL playground funcional, conectado a PostgreSQL y Redis.

---

#### B2 — Cliente Soroban + Deploy a Testnet

**Directorio:** `oracle/src/services/` + `contracts/`

**Tareas:**

1. **Modulo de interaccion con Soroban**
   - Crear `oracle/src/services/soroban.rs` (o `stellar.rs`)
   - Agregar dependencias: `stellar-sdk` (Rust), `reqwest` (HTTP client para Soroban RPC), `ed25519-dalek` (firmas)
   - Implementar:
     - Conexion a Soroban RPC (leer URL de `environments.toml`)
     - Generacion/gestion de keypair del oraculo
     - Funcion generica `invoke_contract(contract_id, function_name, args) -> Result<Value>` que:
       - Construye la transaccion Soroban
       - Simula (preflight)
       - Firma con la clave del oraculo
       - Envia y espera confirmacion
     - Funcion `get_contract_data(contract_id, key) -> Result<Value>` para leer storage

2. **Deploy de contratos a Testnet**
   - Crear script `scripts/deploy_testnet.sh` que:
     - Genera identidades con `stellar keys generate`
     - Compila los 4 contratos (`cargo build --target wasm32-unknown-unknown --release`)
     - Deploya cada contrato a testnet con `stellar contract deploy`
     - Inicializa cada contrato (`initialize()`) con las direcciones correctas
     - Vincula contratos entre si (`link_proof_of_life`, `link_vault`, `set_vault_contract`)
     - Guarda los contract IDs en un archivo `deployed_contracts.json`
   - Documentar el proceso en `docs/DEPLOY.md`

3. **Wrappers tipados por contrato**
   - Crear `oracle/src/services/contracts/vault.rs` con funciones:
     - `create_vault(owner, token) -> VaultId`
     - `deposit(vault_id, amount, token)`
     - `get_vault(vault_id) -> VaultInfo`
     - `get_status(vault_id) -> VaultStatus`
     - `get_balance(vault_id) -> i128`
     - `transition_status(vault_id, new_status)`
   - Crear `oracle/src/services/contracts/proof_of_life.rs`:
     - `submit_verification(user, score, source, signature)`
     - `get_liveness_score(user) -> u32`
     - `register_model(user, weights, bias)`
     - `get_model(user) -> LifeModel`
   - Crear `oracle/src/services/contracts/beneficiary.rs`:
     - `set_beneficiaries(vault_id, beneficiaries)`
     - `can_claim(vault_id, claimer) -> bool`
     - `record_claim(vault_id, claimer) -> u32`
   - Crear `oracle/src/services/contracts/document_registry.rs`:
     - `register_document(owner, ipfs_cid, doc_hash, doc_type, is_encrypted) -> DocId`
     - `grant_access(doc_id, beneficiary) -> DocumentAccess`
     - `verify_document(doc_id) -> DocumentProof`

**Entregable:** Los 4 contratos deployados en testnet, un archivo `deployed_contracts.json` con sus IDs, y un modulo Rust que puede invocar cualquier funcion de cualquier contrato desde el backend.

---

#### B3 — Event Store + Estructura de Jobs

**Directorio:** `oracle/src/`

**Tareas:**

1. **Event Store (PostgreSQL)**
   - Crear migracion para la tabla `events` (seccion 18.3 de la propuesta):
     - `id BIGSERIAL PRIMARY KEY`
     - `event_type VARCHAR(50) NOT NULL`
     - `aggregate_type VARCHAR(30) NOT NULL` — valores: 'vault', 'user', 'document'
     - `aggregate_id UUID NOT NULL`
     - `payload JSONB NOT NULL`
     - `metadata JSONB` — IP, device info, etc.
     - `created_at TIMESTAMPTZ DEFAULT NOW()`
     - `on_chain_tx_hash VARCHAR(64)`
   - Indices: `idx_events_aggregate`, `idx_events_type`
   - Crear `oracle/src/services/event_store.rs` con:
     - `record_event(event_type, aggregate_type, aggregate_id, payload, metadata) -> EventId`
     - `get_events(aggregate_type, aggregate_id, limit) -> Vec<Event>`
     - `get_events_by_type(event_type, since, limit) -> Vec<Event>`
   - Definir todos los tipos de evento como enum (tabla de la seccion 18.3):
     - `VaultCreated`, `DepositMade`, `WithdrawalMade`, `BeneficiariesUpdated`, `StatusTransition`
     - `VerificationSubmitted`, `VerificationPublished`, `ModelUpdated`, `CalibrationCompleted`, `EmergencyCheckin`
     - `DocumentRegistered`, `DocumentLinkedToVault`
     - `GracePeriodStarted`, `HeritageTriggered`, `ClaimExecuted`, `DocumentAccessGranted`, `AllClaimsCompleted`

2. **Framework de Jobs con Redis**
   - Crear `oracle/src/jobs/worker.rs` con:
     - Trait `Job` con metodo `async fn execute(&self) -> Result<()>`
     - Worker loop que lee de colas Redis y ejecuta jobs
     - Soporte para jobs periodicos (cron-like) y event-driven
   - Crear stubs para los 6 jobs definidos en la propuesta (seccion 5.3.4):
     - `oracle/src/jobs/timeout_checker.rs` — `check_alert_timeouts` (cada 1h) + `check_grace_timeouts` (cada 30min)
     - `oracle/src/jobs/chain_sync.rs` — `sync_chain_state` (cada 5min)
     - `oracle/src/jobs/notification.rs` — `send_notifications` (event-driven)
     - `oracle/src/jobs/session_cleanup.rs` — `cleanup_expired_sessions` (cada 6h)
     - `oracle/src/jobs/model_sync.rs` — `model_weight_sync` (event-driven)
   - Cada stub solo logea "Job X ejecutado" por ahora; la implementacion real viene en Sprint 2.

3. **Modelos de datos (structs Rust)**
   - Crear `oracle/src/models/` con structs que mapean las tablas SQL:
     - `user.rs` — `User { id, stellar_address, created_at, calibration_complete }`
     - `vault.rs` — `Vault { id, contract_id, owner_id, status, escrow_contract_id, created_at, last_synced_at }`
     - `verification.rs` — `Verification { id, user_id, score, source, ... 10 features ..., on_chain_tx_hash, created_at }`
     - `document.rs` — `Document { id, owner_id, vault_id, ipfs_cid, doc_hash, doc_type, is_encrypted, metadata }`
     - `event.rs` — `Event { id, event_type, aggregate_type, aggregate_id, payload, metadata, created_at, on_chain_tx_hash }`
   - Implementar `From<sqlx::Row>` o derivar `sqlx::FromRow` para cada struct

**Entregable:** Event store funcional con persistencia en PostgreSQL. Framework de workers que lee de Redis y ejecuta jobs (stubs). Modelos de datos listos para usar en resolvers.

---

#### F1 — Scaffold de la App Movil

**Directorio:** `mobile/`

**Tareas:**

1. **Inicializar proyecto React Native**
   - `npx react-native init PulseProtocol --template react-native-template-typescript`
   - Instalar dependencias core:
     - `@react-navigation/native` + `@react-navigation/stack` + `@react-navigation/bottom-tabs` (navegacion)
     - `@apollo/client` + `graphql` (GraphQL client)
     - `zustand` (estado global)
     - `react-native-keychain` (secure storage)
     - `react-native-biometrics` (huella/face ID)
   - Configurar TypeScript estricto (`"strict": true`, `"noImplicitAny": true`)
   - Crear `.env.example` con `GRAPHQL_URL`, `STELLAR_NETWORK`

2. **Sistema de navegacion**
   - Implementar navegacion segun las pantallas de la propuesta (seccion 21):
   ```
   AuthNavigator (no autenticado)
     - WelcomeScreen
     - CreateWalletScreen
     - ImportWalletScreen

   OnboardingNavigator (primer uso)
     - BiometricSetupScreen (huella)
     - FaceRegistrationScreen (fotos para embedding)
     - CalibrationInfoScreen (explicacion de 2-4 semanas)

   MainNavigator (autenticado, tabs)
     - DashboardTab → DashboardScreen
     - VaultsTab → VaultListScreen → VaultDetailScreen → CreateVaultScreen
     - DocumentsTab → DocumentListScreen → RegisterDocumentScreen
     - BeneficiariesTab → BeneficiaryListScreen → ManageBeneficiariesScreen
     - SettingsTab → SettingsScreen

   ClaimNavigator (beneficiario reclamando)
     - ClaimScreen → ClaimResultScreen
   ```
   - Cada pantalla es un placeholder con titulo y texto descriptivo de lo que hara

3. **Apollo Client setup**
   - Crear `mobile/src/services/graphql/client.ts`:
     - HTTP link para queries/mutations
     - WebSocket link para subscriptions
     - Split link (HTTP para query/mutation, WS para subscription)
     - Cache con `InMemoryCache` configurada
   - Crear `mobile/src/services/graphql/queries.ts` con las queries del schema (seccion 17.2):
     - `VAULT_QUERY`, `MY_VAULTS_QUERY`, `LIVENESS_SCORE_QUERY`, `VERIFICATION_HISTORY_QUERY`
     - `BENEFICIARIES_QUERY`, `DOCUMENTS_QUERY`, `DOCUMENT_QUERY`, `CAN_CLAIM_QUERY`
   - Crear `mobile/src/services/graphql/mutations.ts`:
     - `CREATE_VAULT`, `DEPOSIT`, `WITHDRAW`, `SET_BENEFICIARIES`
     - `REGISTER_DOCUMENT`, `LINK_DOCUMENT_TO_VAULT`
     - `SUBMIT_VERIFICATION`, `EMERGENCY_CHECKIN`, `UPDATE_MODEL_WEIGHTS`, `CLAIM_INHERITANCE`
   - Crear `mobile/src/services/graphql/subscriptions.ts`:
     - `VAULT_STATUS_CHANGED`, `LIVENESS_ALERT`, `CLAIM_STARTED`, `VERIFICATION_COMPLETED`, `DOCUMENT_ACCESS_GRANTED`

4. **Tipos TypeScript**
   - Crear `mobile/src/types/` con interfaces que espejan el schema GraphQL:
     - `vault.ts` — `Vault`, `VaultStatus`, `TokenBalance`, `CreateVaultInput`
     - `beneficiary.ts` — `Beneficiary`, `BeneficiaryInput`
     - `document.ts` — `Document`, `DocumentType`, `RegisterDocumentInput`, `DocumentAccess`
     - `verification.ts` — `LivenessData`, `VerificationRecord`, `VerificationSource`, `VerificationInput`
     - `events.ts` — `VaultStatusEvent`, `LivenessAlertEvent`, `ClaimEvent`

5. **Store global (Zustand)**
   - Crear `mobile/src/store/`:
     - `authStore.ts` — stellar address, isAuthenticated, keypair (en secure storage)
     - `vaultStore.ts` — vaults del usuario, vault seleccionado
     - `livenessStore.ts` — score actual, ultimo verificado, estado de calibracion

**Entregable:** App React Native que se compila, navega entre todas las pantallas (placeholder), tiene Apollo Client configurado y tipos TypeScript listos. No depende del backend todavia.

---

#### F2 — Modulos de IA y Biometria (Investigacion + Prototipos)

**Directorio:** `mobile/src/services/` + `ml/`

**Tareas:**

1. **Investigacion y seleccion de modelos TFLite**
   - Evaluar y seleccionar modelos concretos para React Native:
     - YOLO v8-face TFLite (~6MB) — deteccion de rostros
     - MobileFaceNet / InsightFace TFLite (~1MB) — embedding facial 512-d
     - Modelo de liveness detection TFLite (~2MB) — anti-spoofing
   - Documentar en `docs/ML_MODELS.md`:
     - URLs de descarga de los modelos
     - Formatos de entrada/salida de cada modelo
     - Metricas de rendimiento esperadas
     - Dependencias nativas necesarias en React Native

2. **Setup de TensorFlow Lite en React Native**
   - Instalar y configurar `react-native-tflite` o equivalente
   - Crear `mobile/src/services/vision/tflite_runner.ts`:
     - Funcion generica `loadModel(modelPath) -> Model`
     - Funcion generica `runInference(model, inputTensor) -> outputTensor`
   - Validar que un modelo TFLite dummy carga y ejecuta correctamente en ambas plataformas (Android/iOS)

3. **Modulo de huella dactilar**
   - Crear `mobile/src/services/biometrics/fingerprint.ts` usando `react-native-biometrics`:
     - `checkBiometricAvailability() -> { available: boolean, type: 'fingerprint' | 'faceId' | 'both' }`
     - `authenticateWithBiometric() -> { success: boolean, timestamp: number }`
     - `registerBiometricListeners()` — registra eventos pasivos de uso del sensor
   - Crear `mobile/src/services/biometrics/fingerprint_tracker.ts`:
     - Lleva un registro local de eventos de huella (timestamps)
     - Calcula las 2 features derivadas de la propuesta (seccion 4.2.2):
       - `fingerprint_frequency`: `matches_hoy / promedio_historico` normalizado [0,1]
       - `fingerprint_consistency`: similitud histograma horario vs baseline [0,1]

4. **Perceptron en TypeScript (inferencia)**
   - Crear `mobile/src/services/perceptron/model.ts`:
     - Clase `Perceptron` con:
       - `weights: number[]` (10 pesos)
       - `bias: number`
       - `predict(features: number[]): number` — calcula `sigma(w^T * x + b)` donde sigma es la sigmoide
       - `sigmoid(z: number): number`
     - Cargar pesos desde secure storage o desde la blockchain (via API)
   - Crear `mobile/src/services/perceptron/calibration.ts`:
     - `CalibrationManager` que acumula datos durante 2-4 semanas
     - Almacena features etiquetadas como y=1 (vivo) en almacenamiento local
     - Metodo `isCalibrationComplete(): boolean` (verifica si hay suficientes datos)
     - No entrena en el dispositivo (envia datos al backend para entrenamiento)

5. **Scripts Python para entrenamiento (offline)**
   - Crear `ml/perceptron/train.py`:
     - Carga dataset de features (JSON o CSV)
     - Entrena un perceptron con descenso de gradiente (loss: binary cross-entropy)
     - Exporta pesos como JSON
   - Crear `ml/perceptron/inference.py`:
     - Carga pesos, ejecuta inferencia de prueba, valida output
   - Crear `ml/requirements.txt` con `numpy`, `scikit-learn` (solo para comparacion)

**Entregable:** Modulo de huella funcional, perceptron en TypeScript ejecutable, investigacion de modelos documentada, TFLite cargando al menos un modelo dummy.

---

### SPRINT 2 — API Funcional + Pantallas con Datos

**Objetivo:** Las queries y mutations principales del GraphQL funcionan end-to-end. La app movil muestra datos reales del backend.

---

#### B1 — Resolvers GraphQL (Queries + Mutations)

**Tareas:**

1. **Query resolvers** — Crear `oracle/src/graphql/queries.rs`:
   - `vault(id)` — Lee de PostgreSQL (cache local) con fallback a on-chain
   - `myVaults` — Filtra vaults por `owner_id` del usuario autenticado
   - `livenessScore(userId)` — Lee de Redis (cache) con fallback a PostgreSQL
   - `verificationHistory(userId, limit)` — Lee de tabla `verifications` ordenado por `created_at DESC`
   - `beneficiaries(vaultId)` — Lee de tabla `beneficiaries`
   - `documents(vaultId)` — Lee de tabla `documents` filtrado por `vault_id`
   - `document(docId)` — Lee documento individual
   - `canClaim(vaultId)` — Verifica si el usuario autenticado es beneficiario y no ha reclamado

2. **Mutation resolvers** — Crear `oracle/src/graphql/mutations.rs`:
   - `createVault(input)`:
     1. Invoca `create_vault()` on-chain (via modulo de B2)
     2. Persiste en tabla `vaults`
     3. Registra evento `VaultCreated` en event store (via modulo de B3)
     4. Retorna vault creado
   - `deposit(vaultId, amount, token)`:
     1. Invoca `deposit()` on-chain
     2. Actualiza balance en cache Redis
     3. Registra evento `DepositMade`
   - `withdraw(vaultId, amount, token)`:
     1. Verifica estado ACTIVE
     2. Invoca `withdraw()` on-chain
     3. Registra evento `WithdrawalMade`
   - `setBeneficiaries(vaultId, beneficiaries)`:
     1. Valida que porcentajes sumen 10000
     2. Invoca `set_beneficiaries()` on-chain
     3. Persiste en tabla `beneficiaries`
     4. Registra evento `BeneficiariesUpdated`
   - `registerDocument(input)`:
     1. Invoca `register_document()` on-chain
     2. Si privado: invoca `store_encrypted_key()` por cada beneficiario
     3. Persiste en tablas `documents` + `document_encrypted_keys`
     4. Registra evento `DocumentRegistered`
   - `emergencyCheckin`:
     1. Invoca `emergency_checkin()` on-chain
     2. Actualiza score en Redis y PostgreSQL
     3. Registra evento `EmergencyCheckin`
   - `claimInheritance(vaultId)`:
     1. Verifica estado TRIGGERED y que el usuario es beneficiario
     2. Invoca `record_claim()` on-chain
     3. Invoca `grant_access()` para documentos
     4. Persiste claim y registra eventos

3. **Autenticacion**
   - Implementar autenticacion por firma Stellar:
     - El cliente firma un challenge con su private key
     - El backend verifica la firma con la public key (direccion Stellar)
     - Emite un JWT con `stellar_address` + `user_id`
   - Middleware de Actix-web que extrae y valida JWT en cada request
   - Guard de async-graphql que inyecta el usuario autenticado en el contexto

**Entregable:** Todas las queries y mutations del GraphQL responden con datos reales de PostgreSQL + interaccion on-chain. Autenticacion funcional.

---

#### B2 — Agregador de Senales + Publicador On-Chain

**Tareas:**

1. **Agregador de senales** — Crear `oracle/src/services/aggregator.rs`:
   - Implementar el flujo de la seccion 5.3.2 de la propuesta:
     - Recibe: 10 features individuales + `perceptron_output` del movil
     - **Valida rangos:** cada score en [0, 10000]
     - **Verifica consistencia temporal:** delta entre verificacion anterior no puede ser menor a X minutos (rate limiting)
     - **Compara con historial:** si el score cambia > 30% entre verificaciones consecutivas, marca como anomalia
     - **Rate limiting:** maximo N verificaciones por hora (usando counter en Redis `rate_limit:verify:{user_id}`)
     - **Genera score final agregado:** por ahora, usa el `perceptron_output` directamente; en futuras versiones, pondera con confianza del historial
   - Retorna `AggregatedScore { score: u32, is_anomalous: bool, notes: Vec<String> }`

2. **Publicador on-chain** — Crear `oracle/src/services/publisher.rs`:
   - Implementar el flujo de la seccion 5.3.3 de la propuesta:
     - Recibe score validado del agregador
     - Firma el score con la clave ed25519 del oraculo generando `oracle_signature: [u8; 64]`
     - Construye transaccion Soroban que llama `submit_verification(user, score, source, oracle_sig)` en ProofOfLife
     - Envia a Stellar testnet via Soroban RPC
     - Espera confirmacion de la transaccion
     - Retorna `tx_hash`
   - Implementar deteccion de transicion de estado:
     - Despues de publicar, lee el score actual del contrato
     - Si el score cruza un threshold (7000 o 3000), invoca `transition_status()` en el Vault Contract
     - Registra evento `StatusTransition` en event store

3. **Mutation `submitVerification`** — Completar el resolver:
   - Recibe `VerificationInput` del cliente movil
   - Pasa por agregador -> publicador -> persiste en DB -> registra evento
   - Retorna `VerificationResult { score, status, txHash, timestamp }`

**Entregable:** El flujo completo funciona: app envia features -> backend agrega y valida -> publica score on-chain -> retorna resultado. Transiciones de estado se disparan automaticamente.

---

#### B3 — Jobs Funcionales + IPFS

**Tareas:**

1. **Implementar jobs de timeout** — `oracle/src/jobs/timeout_checker.rs`:
   - `check_alert_timeouts` (cada 1 hora):
     - Consulta vaults en estado ALERT
     - Para cada uno, lee la ultima verificacion
     - Si no hay verificacion exitosa (score > 7000) en las ultimas 72 horas: transicionar a GRACE_PERIOD
     - Registra evento `GracePeriodStarted`
   - `check_grace_timeouts` (cada 30 minutos):
     - Consulta vaults en estado GRACE_PERIOD
     - Lee timestamp de entrada a GRACE
     - Si han pasado `grace_period_days` dias (default 30): transicionar a TRIGGERED
     - Registra evento `HeritageTriggered`
     - Invoca `transition_status()` on-chain

2. **Implementar sync on-chain** — `oracle/src/jobs/chain_sync.rs`:
   - `sync_chain_state` (cada 5 minutos):
     - Lee estado on-chain de cada vault activo
     - Compara con estado en PostgreSQL
     - Si hay diferencia, actualiza DB local
     - Actualiza cache Redis con estados frescos

3. **Cliente IPFS** — Crear `oracle/src/services/ipfs.rs`:
   - Usar `reqwest` para comunicarse con gateway IPFS (Pinata o Infura)
   - Funciones:
     - `upload_file(bytes, filename) -> CID` — sube archivo a IPFS y lo pina
     - `download_file(cid) -> Bytes` — descarga archivo por CID
     - `pin(cid)` — asegura que el archivo persiste
     - `unpin(cid)` — remueve pin
   - Configuracion via variables de entorno: `IPFS_API_URL`, `IPFS_API_KEY`

4. **Cleanup de sesiones** — `oracle/src/jobs/session_cleanup.rs`:
   - Implementar limpieza de claves Redis expiradas
   - Limpiar registros de sesion inactivos

**Entregable:** Los jobs de timeout detectan y transicionan estados automaticamente. El cliente IPFS sube y descarga archivos. El sync mantiene la DB local actualizada con la blockchain.

---

#### F1 — Pantallas Funcionales con Datos Reales

**Tareas:**

1. **Dashboard** — `mobile/src/screens/Dashboard/DashboardScreen.tsx`:
   - Muestra resumen del usuario:
     - Liveness score actual (con indicador visual de color: verde > 70%, amarillo 30-70%, rojo < 30%)
     - Estado del vault principal (badge con color por estado)
     - Numero de documentos registrados
     - Lista de beneficiarios con porcentajes
   - Usa query `MY_VAULTS_QUERY` + `LIVENESS_SCORE_QUERY`
   - Subscription `VAULT_STATUS_CHANGED` para actualizaciones en tiempo real

2. **Vaults** — `mobile/src/screens/Vault/`:
   - `VaultListScreen.tsx` — Lista de vaults del usuario con estado y balance
   - `VaultDetailScreen.tsx` — Detalle de un vault: balance, estado, beneficiarios, documentos vinculados
   - `CreateVaultScreen.tsx` — Formulario:
     - Seleccion de token (XLM, USDC, u otro con trustline)
     - Monto de deposito inicial (opcional)
     - Mutation `CREATE_VAULT` + `DEPOSIT`

3. **Beneficiarios** — `mobile/src/screens/Beneficiaries/`:
   - `BeneficiaryListScreen.tsx` — Lista actual de beneficiarios con porcentajes
   - `ManageBeneficiariesScreen.tsx` — Formulario para agregar/editar:
     - Input de direccion Stellar del beneficiario
     - Slider o input numerico para porcentaje
     - Validacion visual: suma debe ser exactamente 100%
     - Mutation `SET_BENEFICIARIES`
     - Solo habilitado si vault esta en ACTIVE

4. **Documentos** — `mobile/src/screens/Documents/`:
   - `DocumentListScreen.tsx` — Lista de documentos con tipo, fecha, estado de cifrado
   - `RegisterDocumentScreen.tsx` — Formulario:
     - Selector de archivo (document picker)
     - Selector de tipo (`DocumentType`)
     - Toggle publico/privado
     - Si privado: cifrado con AES-256-GCM (delegado a F2) antes de subir
     - Upload a IPFS (via backend) y registrar on-chain

5. **Integracion Wallet**
   - Crear `mobile/src/services/wallet/stellar.ts`:
     - `createKeypair() -> { publicKey, secretKey }` — genera keypair Stellar
     - `importFromSecret(secret) -> { publicKey }` — importa wallet existente
     - `signTransaction(tx, secret) -> signedTx` — firma transacciones
     - `getBalance(publicKey) -> TokenBalance[]` — consulta balance via Horizon API
   - Almacenar secret key en `react-native-keychain` (Secure Enclave)
   - Crear `mobile/src/screens/Auth/CreateWalletScreen.tsx` y `ImportWalletScreen.tsx`

**Entregable:** La app muestra datos reales del backend. Se puede crear vault, depositar, gestionar beneficiarios, registrar documentos y ver el dashboard con liveness score.

---

#### F2 — Vision Artificial + Recoleccion de Patrones

**Tareas:**

1. **Modulo de deteccion facial (YOLO)** — `mobile/src/services/vision/yolo.ts`:
   - Cargar modelo YOLO v8-face TFLite
   - `detectFaces(frame: ImageData) -> FaceDetection[]`
     - Procesa frame de la camara frontal
     - Retorna bounding boxes de rostros detectados
   - Integrar con `react-native-camera` para captura de frames

2. **Modulo de verificacion facial (InsightFace)** — `mobile/src/services/vision/insightface.ts`:
   - Cargar modelo MobileFaceNet TFLite
   - `generateEmbedding(faceCrop: ImageData) -> number[]` — genera vector 512-d
   - `compareEmbeddings(embedding1, embedding2) -> number` — cosine similarity [0,1]
   - `setReferenceEmbedding(embedding)` — guarda embedding de referencia en secure storage
   - Flujo de onboarding: captura N fotos -> genera embedding promedio de referencia

3. **Modulo de liveness detection** — `mobile/src/services/vision/liveness.ts`:
   - Cargar modelo de liveness TFLite
   - Implementar deteccion multi-senal:
     - Deteccion de parpadeo (secuencia de frames)
     - Micro-movimientos involuntarios
     - Analisis de textura (pantalla vs piel real)
   - `checkLiveness(frames: ImageData[]) -> { score: number, isReal: boolean }`

4. **Modulo de patrones de comportamiento** — `mobile/src/services/patterns/`:
   - `behavior.ts`:
     - Recolecta eventos de uso en background
     - Registra: timestamps de uso, duracion de sesiones, apps activas (Usage Stats API)
     - Almacena baseline en storage local
   - `typing.ts`:
     - Listener de eventos de teclado (sin capturar contenido, solo timing)
     - Calcula velocidad promedio, ritmo, patrones
   - `movement.ts`:
     - Lee datos del acelerometro y giroscopio
     - Calcula signature de movimiento (como sostiene y mueve el dispositivo)
   - `feature_extractor.ts`:
     - Toma datos raw de los 3 modulos + vision + huella
     - Calcula las 10 features normalizadas [0,1] del vector del perceptron:
       - `x1: face_match_score`
       - `x2: face_liveness_score`
       - `x3: fingerprint_frequency`
       - `x4: fingerprint_consistency`
       - `x5: time_of_day_normality`
       - `x6: typing_pattern_match`
       - `x7: app_usage_match`
       - `x8: movement_pattern_match`
       - `x9: days_since_last_verify` (normalizado inverso)
       - `x10: session_behavior`
     - `extractFeatures() -> number[10]`

5. **Flujo de verificacion completo (en dispositivo)**
   - Crear `mobile/src/services/verification.ts`:
     - `runVerification()`:
       1. Captura frame de camara frontal
       2. Detecta rostro (YOLO)
       3. Genera embedding y compara (InsightFace) -> `face_match_score`
       4. Verifica liveness -> `face_liveness_score`
       5. Lee metricas de huella (tracker) -> `fingerprint_frequency`, `fingerprint_consistency`
       6. Lee metricas de patrones -> `time_of_day_normality`, `typing_pattern_match`, `app_usage_match`, `movement_pattern_match`, `session_behavior`
       7. Calcula `days_since_last_verify`
       8. Ejecuta perceptron local -> `perceptron_output`
       9. Envia todo al backend via mutation `submitVerification`
       10. Retorna resultado

**Entregable:** Los 3 pilares de verificacion implementados en el dispositivo. El flujo completo funciona: captura -> procesa -> features -> perceptron -> envia al backend.

---

### SPRINT 3 — Subscriptions en Tiempo Real + Onboarding + Escrow

**Objetivo:** El sistema funciona end-to-end con notificaciones en tiempo real, onboarding biometrico completo y la integracion con Trustless Work.

---

#### B1 — Subscriptions GraphQL + Notificaciones

**Tareas:**

1. **WebSocket server para Subscriptions**
   - Configurar `actix-web-actors` para WebSocket en Actix-web
   - Implementar los 5 subscriptions de la propuesta (seccion 17.2):
     - `vaultStatusChanged(vaultId)` — emite cuando cambia el estado de un vault
     - `livenessAlert(userId)` — emite cuando un score cruza un threshold
     - `claimStarted(vaultId)` — emite cuando un beneficiario inicia un claim
     - `verificationCompleted(userId)` — emite despues de cada verificacion publicada
     - `documentAccessGranted(beneficiaryId)` — emite cuando se revela acceso a documento
   - Usar Redis PubSub como broker de eventos entre instancias:
     - Canal `pubsub:vault:{id}` para eventos de vault
     - Canal `pubsub:user:{id}` para alertas de liveness
   - Los resolvers de mutations publican al canal Redis correspondiente despues de cada operacion exitosa

2. **Sistema de notificaciones push**
   - Crear `oracle/src/services/notification.rs`:
     - Tabla `push_tokens` (user_id, device_token, platform)
     - Integracion con Firebase Cloud Messaging (FCM) para push notifications
     - Mutation `registerPushToken(token, platform)` para que la app registre su token
   - Implementar logica de notificacion por estado (tabla seccion 11.4):
     - ACTIVE: ninguna notificacion push
     - ALERT: push semanal al owner ("Tu score de vida ha bajado")
     - GRACE: push diaria al owner + contactos de emergencia
     - TRIGGERED: push a todos los beneficiarios ("Herencia activada")

3. **Mutation `updateModelWeights`**
   - Recibe nuevos pesos del perceptron (entrenados en el dispositivo o via script Python)
   - Invoca `update_model()` on-chain
   - Actualiza tabla `user_models` en PostgreSQL
   - Registra evento `ModelUpdated`
   - Valida que el cambio de pesos no sea excesivo (factor de olvido lambda)

**Entregable:** La app recibe eventos en tiempo real via WebSocket. Las notificaciones push se envian segun el estado del vault. Los pesos del modelo se actualizan on-chain.

---

#### B2 — Integracion Trustless Work (Escrow C2C)

**Tareas:**

1. **Investigar API de Trustless Work**
   - Leer documentacion del SDK de Trustless Work (`@trustless-work/escrow`)
   - Identificar las funciones C2C disponibles en su contrato Soroban:
     - `initialize_escrow()`, `fund_escrow()`, `approve_milestone()`, `release_funds()`
   - Documentar el mapping de roles (tabla seccion 14.2 de la propuesta):
     - `service_provider` = Oraculo de Pulse
     - `approver` = ProofOfLife Contract
     - `release_signer` = Vault Contract
     - `dispute_resolver` = sistema de testigos (futuro)
     - `receiver` = beneficiario
     - `platform_address` = Pulse Protocol

2. **Modulo de escrow** — Crear `oracle/src/services/escrow.rs`:
   - `create_escrow(vault_id, beneficiaries, total_amount, token)`:
     - Invoca factory contract de Trustless Work para desplegar un escrow
     - Crea milestones: uno por beneficiario con su porcentaje
     - Retorna escrow contract ID
   - `fund_escrow(escrow_id, amount, token)`:
     - Transfiere fondos del vault al escrow
   - `approve_milestone(escrow_id, milestone_index)`:
     - Llamado cuando el vault pasa a TRIGGERED
     - Aprueba cada milestone para que los beneficiarios puedan reclamar
   - `release_funds(escrow_id, milestone_index, beneficiary)`:
     - Llamado cuando un beneficiario ejecuta claim
     - Libera fondos al beneficiario (menos fees)

3. **Integrar escrow en el flujo de herencia**
   - Modificar `createVault` mutation: despues de crear vault on-chain, crear escrow en Trustless Work
   - Modificar `deposit` mutation: despues de depositar on-chain, funded escrow
   - Modificar job `check_grace_timeouts`: cuando transiciona a TRIGGERED, llamar `approve_milestone()` para cada beneficiario
   - Modificar `claimInheritance` mutation: llamar `release_funds()` via C2C

**Entregable:** El flujo de herencia usa Trustless Work como escrow. Al crear un vault se crea un escrow. Al activar la herencia, los fondos se liberan via C2C.

---

#### B3 — Servicio de Entrenamiento del Perceptron + Mejoras en Jobs

**Tareas:**

1. **Servicio de entrenamiento** — Crear `oracle/src/services/training.rs`:
   - Recibe dataset de features de calibracion del usuario (acumulado durante 2-4 semanas)
   - Implementa entrenamiento del perceptron con descenso de gradiente:
     - Loss function: Binary Cross-Entropy
     - Learning rate: configurable (default 0.01)
     - Epochs: configurable (default 100)
   - Retorna pesos entrenados (10 weights + bias)
   - Publica pesos on-chain via modulo de B2

2. **Mutation `completeCalibration`**
   - Recibe: dataset de features de calibracion (array de 10-feature vectors)
   - Valida: suficientes muestras (al menos 14 dias de datos)
   - Ejecuta entrenamiento
   - Invoca `register_model()` on-chain
   - Actualiza `user_models` en PostgreSQL
   - Marca `calibration_complete = true` en tabla `users`
   - Registra eventos `CalibrationCompleted` + `ModelUpdated`

3. **Mejorar job de notificaciones** — `oracle/src/jobs/notification.rs`:
   - Implementar logica real de envio de push via FCM
   - Templates de mensajes por tipo de alerta:
     - ALERT: "Pulse Protocol: Tu score de vida ha bajado a {score}%. Verifica tu identidad."
     - GRACE: "URGENTE: Periodo de gracia activo. Quedan {days} dias. Realiza un check-in."
     - TRIGGERED: "Se ha activado la herencia del vault #{vault_id}. Puedes reclamar tu parte."

4. **Job de sincronizacion de pesos** — `oracle/src/jobs/model_sync.rs`:
   - Cuando un usuario actualiza pesos desde otro dispositivo, detectar el cambio on-chain
   - Sincronizar con la DB local
   - Notificar al dispositivo via subscription

**Entregable:** El backend puede entrenar perceptrones con datos de calibracion. Las notificaciones push se envian realmente. Los pesos se sincronizan entre dispositivos.

---

#### F1 — Onboarding Completo + Claim + Emergency Check-in

**Tareas:**

1. **Flujo de onboarding** — `mobile/src/screens/Onboarding/`:
   - `BiometricSetupScreen.tsx`:
     - Solicita permiso de biometria
     - Registra huella via BiometricPrompt/LocalAuth
     - Muestra confirmacion de exito
   - `FaceRegistrationScreen.tsx`:
     - Guia al usuario para capturar N fotos (frente, izquierda, derecha, arriba, abajo)
     - Usa modulo de vision de F2 para generar embedding de referencia
     - Muestra progreso y confirmacion
     - Almacena embedding en Secure Enclave
   - `CalibrationInfoScreen.tsx`:
     - Explica el periodo de calibracion (2-4 semanas)
     - Muestra progreso de la calibracion (dias completados / 14 minimo)
     - Cuando la calibracion es suficiente, boton "Completar calibracion" que envia datos al backend

2. **Pantalla de Emergency Check-in**
   - Crear `mobile/src/screens/Emergency/EmergencyCheckinScreen.tsx`:
     - Verificacion facial estricta (`face_match > 0.9` + `liveness > 0.8`)
     - Autenticacion biometrica del dispositivo (huella)
     - Si ambas pasan: llama mutation `emergencyCheckin`
     - Muestra resultado (exito: score reseteado a 10000, estado vuelve a ACTIVE)
   - Accesible desde DashboardScreen cuando el estado es ALERT o GRACE

3. **Pantalla de Claim (para beneficiarios)**
   - Crear `mobile/src/screens/Claim/`:
     - `ClaimScreen.tsx`:
       - Lista vaults donde el usuario es beneficiario y el estado es TRIGGERED
       - Para cada vault: muestra porcentaje asignado, monto estimado, documentos vinculados
       - Boton "Reclamar herencia"
       - Llama mutation `claimInheritance`
     - `ClaimResultScreen.tsx`:
       - Muestra resultado del claim: monto recibido, tx hash
       - Lista de documentos accesibles (publicos: link IPFS, privados: clave de descifrado revelada)

4. **Settings**
   - `mobile/src/screens/Settings/SettingsScreen.tsx`:
     - Ver direccion Stellar (con copy)
     - Ver estado de calibracion
     - Ver version del modelo y fecha de ultima actualizacion
     - Configurar thresholds personalizados (alert, critical, grace period)
     - Exportar/importar keypair (con advertencia de seguridad)
     - Registrar token push para notificaciones

**Entregable:** El onboarding guia al usuario desde la instalacion hasta la activacion completa. Los beneficiarios pueden reclamar herencia. Emergency check-in funciona.

---

#### F2 — Verificacion Automatica en Background + Calibracion

**Tareas:**

1. **Servicio en background para patrones**
   - Implementar background task que:
     - Corre 24/7 en background (React Native background service)
     - Registra eventos de uso: timestamps de foreground/background, duracion de sesiones
     - Registra datos de sensores: acelerometro, giroscopio (a intervalos)
     - Registra eventos de teclado (timing, no contenido)
     - Calcula features de patrones incrementalmente

2. **Verificacion facial automatica**
   - Implementar franjas adaptativas de verificacion:
     - Durante calibracion: registrar ventanas de alta actividad del usuario
     - Post-calibracion: programar verificacion facial en esas ventanas
     - "Captura silenciosa": cuando el usuario esta mirando la pantalla activamente, la camara frontal captura y verifica
   - Frecuencia por estado (tabla seccion 11.4 de la propuesta):
     - ACTIVE: franjas adaptativas (~2-3x/dia)
     - ALERT: franjas x3 (~6-9x/dia)
     - GRACE: cada hora si hay actividad
   - Si la verificacion falla (no detecta rostro, score bajo), no registra como "muerte" — simplemente espera al siguiente momento

3. **Logica de calibracion en el dispositivo**
   - `CalibrationService`:
     - Acumula features diarias durante el periodo de calibracion
     - Calcula baselines para cada feature (promedio, desviacion estandar)
     - Cuando hay >= 14 dias de datos, marca calibracion como lista
     - Envia dataset acumulado al backend para entrenamiento
     - Recibe pesos entrenados del backend y los almacena localmente

4. **Cifrado AES-256-GCM para documentos**
   - Crear `mobile/src/services/documents/encryption.ts`:
     - `generateAESKey() -> Uint8Array` (256 bits)
     - `encryptDocument(document: Uint8Array, key: Uint8Array) -> { ciphertext: Uint8Array, iv: Uint8Array, tag: Uint8Array }`
     - `decryptDocument(ciphertext: Uint8Array, key: Uint8Array, iv: Uint8Array, tag: Uint8Array) -> Uint8Array`
     - `encryptKeyForBeneficiary(aesKey: Uint8Array, beneficiaryPublicKey: string) -> string` (cifra clave AES con pubkey Stellar)
     - `decryptKeyWithPrivateKey(encryptedKey: string, privateKey: string) -> Uint8Array`
   - Integrar con pantalla de registro de documentos de F1

**Entregable:** La verificacion corre automaticamente en background. La calibracion acumula datos y entrena el modelo. Los documentos se cifran/descifran correctamente.

---

### SPRINT 4 — Estabilizacion, Testing End-to-End y Deploy

**Objetivo:** El sistema completo funciona en testnet. Todos los flujos estan probados end-to-end. Listo para demo.

---

#### B1 — Testing E2E + Seguridad

**Tareas:**

1. **Tests de integracion del backend**
   - Tests E2E que cubran los flujos completos:
     - Registro de usuario -> crear vault -> depositar -> configurar beneficiarios -> registrar documento
     - Verificacion de vida -> score baja -> ALERT -> GRACE -> TRIGGERED -> claim
     - Emergency check-in desde ALERT/GRACE -> vuelve a ACTIVE
     - Calibracion completa -> entrenamiento -> pesos publicados on-chain
   - Tests de autenticacion:
     - Request sin JWT -> 401
     - JWT expirado -> 401
     - JWT valido -> acceso a datos propios
     - Intento de acceder a vault ajeno -> 403

2. **Hardening de seguridad**
   - Rate limiting global por IP (middleware Actix-web)
   - Validacion de inputs: sanitizar todos los strings, validar rangos numericos
   - CORS configurado correctamente (solo origenes permitidos)
   - Logs de auditoria para operaciones sensibles (claims, emergency checkin, model update)
   - Verificar que las claves del oraculo no se exponen en logs ni respuestas

3. **Documentacion API**
   - GraphQL playground con descripciones en cada type, query y mutation
   - Crear `docs/API.md` con ejemplos de uso de cada endpoint

---

#### B2 — Deploy Completo a Testnet + Monitoreo

**Tareas:**

1. **Script de deploy automatizado**
   - Mejorar `scripts/deploy_testnet.sh`:
     - Deploy de los 4 contratos
     - Inicializacion y vinculacion
     - Configuracion del escrow de Trustless Work
     - Verificacion post-deploy (invoca funciones de lectura para validar)
   - Crear script `scripts/setup_testnet_data.sh`:
     - Crea usuarios de prueba con fondos (friendbot)
     - Crea vaults de ejemplo
     - Simula depositos

2. **Monitoreo basico**
   - Metricas con `prometheus` crate:
     - Requests por segundo
     - Latencia de queries/mutations
     - Numero de verificaciones publicadas on-chain
     - Errores de transaccion
   - Endpoint `/metrics` para Prometheus
   - Dashboard basico en Grafana (si hay infraestructura disponible)

3. **Docker build del backend**
   - `oracle/Dockerfile` multi-stage:
     - Stage 1: build con imagen Rust
     - Stage 2: imagen minima con solo el binario
   - Actualizar `docker-compose.yml` para que el oracle use la imagen built

---

#### B3 — Resilencia + Logs + Cleanup

**Tareas:**

1. **Resilencia de jobs**
   - Reintentos automaticos con backoff exponencial para:
     - Transacciones Soroban fallidas
     - Subida a IPFS fallida
     - Push notifications fallidas
   - Dead letter queue para jobs que fallan despues de N reintentos
   - Alertas cuando un job critico falla repetidamente

2. **Logging estructurado**
   - Configurar `tracing` con formato JSON para produccion
   - Log de contexto en cada operacion: user_id, vault_id, tx_hash
   - Niveles de log por modulo configurables via variable de entorno

3. **Migracion de datos y cleanup**
   - Verificar que las migraciones corren limpiamente desde cero
   - Job de archivado de eventos antiguos (> 90 dias) a tabla de archivo
   - Limpieza de datos de prueba antes de demos

---

#### F1 — Pulido de UI + Testing + Manejo de Errores

**Tareas:**

1. **Manejo de errores en toda la app**
   - Componente `ErrorBoundary` global
   - Manejo de errores de red (offline, timeout)
   - Mensajes de error amigables para cada mutacion fallida
   - Loading states para todas las operaciones asincronas
   - Retry automatico para queries fallidas

2. **Estados vacios y edge cases**
   - Pantalla de "No tienes vaults" con CTA para crear uno
   - Pantalla de "No tienes documentos" con CTA para registrar
   - Estado de "Calibracion en progreso" con barra de progreso
   - Manejo de vault en cada estado (deshabilitar acciones segun VaultStatus)

3. **Testing de la app**
   - Unit tests para hooks personalizados
   - Unit tests para logica de store (Zustand)
   - Tests de snapshot para componentes principales
   - Test manual del flujo completo en dispositivo real

---

#### F2 — Optimizacion de Modelos + Testing de IA

**Tareas:**

1. **Optimizacion de rendimiento de modelos**
   - Benchmark de inferencia en dispositivo real:
     - Tiempo de YOLO inference
     - Tiempo de InsightFace embedding
     - Tiempo de liveness check
     - Consumo de bateria en background
   - Optimizar si es necesario: reducir resolucion de input, usar quantized models

2. **Testing de modulos de IA**
   - Tests unitarios para:
     - Perceptron: `predict()` con pesos conocidos -> output esperado
     - Feature extractor: dado un set de datos mock -> features correctas
     - Cosine similarity: vectores identicos -> 1.0, ortogonales -> 0.0
   - Tests de integracion:
     - Flujo de verificacion completo con datos mock
     - Calibracion con dataset sintetico -> pesos razonables

3. **Mejoras en anti-spoofing**
   - Implementar consistencia multi-senal:
     - Face score debe correlacionar con fingerprint
     - Alertar si face score alto pero sin actividad de huella/patrones
   - Validar que el modelo de liveness rechaza:
     - Fotos estaticas
     - Videos pregrabados
     - Pantallas mostrando el rostro

---

## 4. Diagrama de Dependencias entre Tareas

```
Sprint 1 (todo en paralelo, sin dependencias cruzadas):
  B1: Servidor + DB + Schema GraphQL
  B2: Cliente Soroban + Deploy testnet
  B3: Event Store + Framework Jobs + Modelos
  F1: Scaffold mobile + Navegacion + Apollo + Types
  F2: Investigacion modelos + TFLite setup + Huella + Perceptron TS

Sprint 2 (dependencias):
  B1 depende de: B2 (para mutations que invocan on-chain), B3 (para event store)
  B2 no tiene dependencia (trabaja en agregador/publicador)
  B3 depende de: B2 (para jobs que invocan on-chain)
  F1 depende de: B1 (para datos reales via GraphQL)
  F2 no tiene dependencia (trabaja en vision/patrones)

Sprint 3:
  B1 depende de: Sprint 2 completo
  B2 depende de: Sprint 2 B2 completo (para C2C con Trustless Work)
  B3 depende de: Sprint 2 B2 + B3 completos
  F1 depende de: Sprint 2 F1 + B1 completos
  F2 depende de: Sprint 2 F2 completo

Sprint 4 (todos trabajan en estabilizacion):
  Todos dependen de Sprint 3 completo
```

---

## 5. Riesgos y Mitigaciones

| Riesgo | Probabilidad | Impacto | Mitigacion |
|---|---|---|---|
| Modelos TFLite no funcionan en React Native | Media | Alto | F2 investiga esto en Sprint 1 antes de depender de ello. Plan B: usar modelos mas pequenos o APIs nativas |
| Trustless Work SDK no soporta C2C desde Rust | Media | Alto | B2 investiga en Sprint 2. Plan B: interactuar via HTTP API en lugar de C2C directo |
| Soroban RPC inestable en testnet | Baja | Medio | Implementar reintentos con backoff. Cache agresivo en el backend |
| Background tasks limitados en iOS | Alta | Medio | F2 debe investigar limitaciones de iOS para servicios en background. Plan B: usar push notifications para despertar la app |
| Performance del perceptron on-device | Baja | Bajo | El perceptron es trivial (10 multiplicaciones + sigmoide). Sin riesgo real |

---

## 6. Convenciones del Equipo

### Git

- **Branching:** `feature/{sprint}-{dev}-{descripcion}` (ej: `feature/s1-b1-graphql-schema`)
- **Commits:** Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`)
- **PRs:** Requerido para merge a `main`. Minimo 1 review de otro miembro.
- **Nunca** commitear `.env`, credenciales, API keys, o secret keys

### Codigo

- **Rust:** `snake_case`, documentar funciones publicas con `///`, `cargo fmt` + `cargo clippy` antes de cada commit
- **TypeScript:** `camelCase` para funciones/variables, `PascalCase` para componentes/tipos. `strict: true`. No usar `any`.
- **Scores:** `u32` en rango 0-10000 (representa 0.00%-100.00%)
- **Decimales Soroban:** Fixed-point con 6 decimales (multiplicar por 1_000_000)
- **Biometria:** NUNCA almacenar datos raw. Solo scores normalizados.

### Comunicacion

- Daily standup (breve, 5-10 min): que hice ayer, que hare hoy, bloqueos
- Review de sprint al final de cada sprint: demo + retrospectiva
- Canal de comunicacion para bloqueos tecnicos urgentes

---

## 7. Definicion de "Hecho" por Sprint

| Sprint | Criterio de aceptacion |
|---|---|
| **Sprint 1** | Backend levanta con GraphQL playground. App navega entre pantallas. Contratos en testnet. Perceptron TS funciona. |
| **Sprint 2** | Queries/mutations retornan datos reales. App muestra datos del backend. Verificacion se publica on-chain. |
| **Sprint 3** | Subscriptions funcionan en tiempo real. Onboarding completo. Escrow integrado. Verificacion automatica en background. |
| **Sprint 4** | Tests E2E pasan. Deploy automatizado a testnet. Demo fluida del ciclo completo de herencia. |

---

*Plan generado: Febrero 2026*
*Base: Pulse_Protocol_Propuesta.md v1.0*
