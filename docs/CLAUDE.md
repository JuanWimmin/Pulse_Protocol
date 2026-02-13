# CLAUDE.md - Pulse Protocol

## Proyecto

Pulse Protocol es un sistema de herencia criptográfica descentralizada sobre Stellar/Soroban con verificación de prueba de vida mediante IA (visión artificial + patrones de comportamiento + perceptrón).

## Documentación

Lee `CLAUDE_CODE_CONTEXT.md` para el contexto completo del proyecto incluyendo:
- Arquitectura del sistema
- Especificación del perceptrón
- Interfaces de smart contracts
- Stack tecnológico
- Estructura de directorios

## Stack Principal

- **Contracts**: Rust + Soroban SDK
- **Backend**: Rust + Actix-web
- **Mobile**: React Native + TensorFlow Lite
- **Database**: PostgreSQL + Redis

## Comandos Frecuentes

```bash
# Contracts
cd contracts && cargo build --target wasm32-unknown-unknown --release
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/vault.wasm

# Backend
cd oracle && cargo run

# Mobile
cd mobile && npm start
```

## Convenciones

- Rust: snake_case, documentar funciones públicas
- Fixed-point para decimales en Soroban (6 decimales, multiplicar por 1_000_000)
- Scores como u32 en rango 0-10000 (representa 0.00-100.00%)
- Nunca almacenar datos biométricos raw, solo scores normalizados

## Prioridad Actual

Fase 1 MVP: Comenzar con los smart contracts básicos (Vault → ProofOfLife → Beneficiary)
