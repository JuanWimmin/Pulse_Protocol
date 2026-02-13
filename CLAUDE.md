# CLAUDE.md - Pulse Protocol

## Proyecto

Pulse Protocol es un sistema de herencia criptográfica descentralizada sobre Stellar/Soroban con verificación de prueba de vida mediante IA (visión artificial + huella + patrones de comportamiento + perceptrón).

## Documentación

- `docs/Pulse_Protocol_Propuesta.md` — Propuesta técnica completa (arquitectura, contratos, API, ML, modelo de negocio)
- `docs/CLAUDE_CODE_CONTEXT.md` — Contexto de desarrollo original
- `docs/pulse_whitepaper (1).pdf` — Whitepaper técnico

## Stack Principal

- **Contracts**: Rust + Soroban SDK 23.4.0
- **Backend**: Rust + Actix-web + async-graphql
- **Mobile**: React Native + TensorFlow Lite + YOLO + InsightFace
- **Database**: PostgreSQL + Redis + Event Store
- **Documents**: IPFS + cifrado AES-256-GCM
- **Escrow**: Trustless Work (Contract-to-Contract)

## Comandos Frecuentes

```bash
# Contracts - build
cargo build --workspace --target wasm32-unknown-unknown --release

# Contracts - test
cargo test --workspace

# Backend
cd oracle && cargo run

# Mobile
cd mobile && npm start
```

## Convenciones

- Rust: snake_case, documentar funciones públicas con `///`
- Fixed-point para decimales en Soroban (6 decimales, multiplicar por 1_000_000)
- Scores como u32 en rango 0-10000 (representa 0.00-100.00%)
- Nunca almacenar datos biométricos raw, solo scores normalizados
- Commits: Conventional Commits (feat:, fix:, docs:, refactor:, test:)
- Backend EXCLUIDO del workspace de contratos (diferente target de compilación)

## Prioridad Actual

Fase 0 + Fase 1: Setup del proyecto + Smart Contracts (Vault → ProofOfLife → Beneficiary → DocumentRegistry)
