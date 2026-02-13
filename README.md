# Pulse Protocol

Sistema de Herencia Criptográfica Descentralizada con Verificación Biométrica Pasiva mediante Inteligencia Artificial.

Construido sobre **Stellar / Soroban**.

## Arquitectura

```
┌──────────────────────────────────────────────────┐
│            CAPA 1: CLIENTE MÓVIL                 │
│  Visión AI │ Huella │ Patrones │ Perceptrón      │
└────────────────────┬─────────────────────────────┘
                     │ GraphQL (HTTPS + WSS)
                     ▼
┌──────────────────────────────────────────────────┐
│            CAPA 2: BACKEND / ORÁCULO             │
│  async-graphql │ Agregador │ Publicador On-Chain │
│  PostgreSQL    │ Redis     │ Event Store         │
└────────────────────┬─────────────────────────────┘
                     │ Soroban RPC
                     ▼
┌──────────────────────────────────────────────────┐
│         CAPA 3: SMART CONTRACTS (Soroban)        │
│  Vault │ ProofOfLife │ Beneficiary │ DocRegistry │
│              + Trustless Work Escrow (C2C)       │
└────────────────────┬─────────────────────────────┘
                     ▼
┌──────────────────────────────────────────────────┐
│  CAPA 4: IPFS (docs) + STELLAR NETWORK (estado) │
└──────────────────────────────────────────────────┘
```

## Stack

| Capa | Tecnología |
|------|-----------|
| Smart Contracts | Rust + Soroban SDK |
| Backend | Rust + Actix-web + async-graphql |
| Mobile | React Native + TensorFlow Lite |
| Base de datos | PostgreSQL + Redis + Event Store |
| Almacenamiento | IPFS (documentos) + Stellar Ledger (estado) |

## Estructura del Proyecto

```
pulse-protocol/
├── contracts/           # Smart Contracts Soroban (Rust)
│   ├── vault/
│   ├── proof-of-life/
│   ├── beneficiary/
│   └── document-registry/
├── oracle/              # Backend / Oráculo (Rust)
├── mobile/              # App React Native
├── ml/                  # Scripts de ML
└── docs/                # Documentación
```

## Desarrollo

### Requisitos

- Rust (stable) + target `wasm32-unknown-unknown`
- Stellar CLI (`stellar`)
- Node.js 18+
- PostgreSQL 16+
- Redis 7+

### Contratos

```bash
# Build
cargo build --workspace --target wasm32-unknown-unknown --release

# Test
cargo test --workspace
```

### Backend

```bash
cd oracle && cargo run
```

### Mobile

```bash
cd mobile && npm start
```

## Documentación

- [Propuesta Técnica Completa](docs/Pulse_Protocol_Propuesta.md)
- [Whitepaper](docs/pulse_whitepaper%20(1).pdf)
