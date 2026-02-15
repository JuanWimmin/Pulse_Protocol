# CLAUDE.md - Pulse Protocol

## Proyecto

Pulse Protocol es un sistema de herencia criptográfica descentralizada sobre Stellar/Soroban con verificación de prueba de vida mediante perceptrón (IA simplificada para MVP) + huella biométrica. Los fondos se custodian via Trustless Work (Escrow C2C).

## Documentación

- `docs/Pulse_Protocol_Propuesta.pdf` — Propuesta técnica completa (arquitectura, contratos, API, ML, modelo de negocio)
- `docs/PLAN_DESARROLLO_MVP.md` — **Plan de desarrollo MVP activo** (2 sprints, 5 personas, demo/hackathon)
- `docs/PLAN_DESARROLLO.md` — Plan de desarrollo completo (referencia, no activo)
- `docs/CLAUDE_CODE_CONTEXT.md` — Contexto de desarrollo original
- `docs/pulse_whitepaper (1).pdf` — Whitepaper técnico

## Stack MVP

- **Contracts**: Rust + Soroban SDK 23.4.0 (Vault + ProofOfLife + Beneficiary)
- **Backend**: Rust + Actix-web + async-graphql + PostgreSQL (sin Redis, sin Event Store)
- **Mobile**: React Native + Perceptrón TS (pesos hardcodeados) + BiometricPrompt
- **Escrow**: Trustless Work (Contract-to-Contract desde Vault Contract)
- **Auth**: Firma Stellar simple + sesión en memoria

### Lo que NO está en el MVP:

- DocumentRegistry (contrato existe pero no se deploya)
- IPFS, cifrado AES-256-GCM, sistema de documentos
- Redis, Event Store, jobs en background
- TensorFlow Lite, YOLO, InsightFace, liveness detection
- Patrones de comportamiento (typing, movement, app usage)
- Calibración del perceptrón (pesos hardcodeados)
- GraphQL Subscriptions / WebSocket
- Push notifications (FCM)

## Contratos

| Contrato | Estado | Deploy MVP |
|---|---|---|
| Vault | Completo + modificaciones C2C pendientes | Sí |
| ProofOfLife | Completo | Sí |
| Beneficiary | Completo | Sí |
| DocumentRegistry | Completo | No (futuro) |

### Modificación pendiente en Vault Contract:

Agregar funciones C2C con Trustless Work: `create_escrow`, `fund_escrow`, `approve_milestones`, `release_to_beneficiary`.

## Base de Datos MVP (PostgreSQL, 4 tablas)

- `users` — id, stellar_address, created_at
- `vaults` — id, contract_id, owner_id, status, escrow_contract_id, created_at, last_synced_at
- `beneficiaries` — id, vault_id, stellar_address, percentage, claimed, claimed_at
- `verifications` — id, user_id, score, source, perceptron_output, on_chain_tx_hash, created_at

## API GraphQL MVP

- **Queries (4):** vault, myVaults, livenessScore, beneficiaries
- **Mutations (7):** createVault, deposit, setBeneficiaries, submitVerification, emergencyCheckin, claimInheritance, forceTransition
- **Subscriptions:** ninguna

## Comandos Frecuentes

```bash
# Contracts - build
cargo build --workspace --target wasm32-unknown-unknown --release

# Contracts - test
cargo test --workspace

# Backend - con Docker (frontend devs)
docker compose up

# Backend - nativo (backend devs)
docker compose up postgres
cd oracle && cargo run

# Deploy contratos a testnet
./scripts/deploy_testnet.sh

# Mobile
cd mobile && npm start

# Acceso remoto al backend
ngrok http 8080
```

## Convenciones

- Rust: snake_case, documentar funciones públicas con `///`
- TypeScript: camelCase funciones/variables, PascalCase componentes/tipos. strict: true. No usar any.
- Fixed-point para decimales en Soroban (6 decimales, multiplicar por 1_000_000)
- Scores como u32 en rango 0-10000 (representa 0.00-100.00%)
- Nunca almacenar datos biométricos raw, solo scores normalizados
- Commits: Conventional Commits (feat:, fix:, docs:, refactor:, test:)
- Branches: `feature/mvp-{dev}-{descripcion}`
- Backend EXCLUIDO del workspace de contratos (diferente target de compilación)

## Prioridad Actual

**MVP - Sprint 1:** Scaffolding + Contratos en testnet (con C2C Trustless Work) + App navegable + Perceptrón TS

### Riesgo crítico:

Trustless Work C2C — B2 debe validar que funciona para el **día 3 del Sprint 1**. Si no, se cambia a backend-orquesta.
