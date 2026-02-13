# PULSE PROTOCOL - Contexto para Desarrollo

## Resumen del Proyecto

**Pulse Protocol** es un sistema de herencia criptográfica descentralizada construido sobre Stellar/Soroban que utiliza inteligencia artificial para verificar continuamente la "prueba de vida" del usuario mediante visión artificial y análisis de patrones de comportamiento.

### Problema que Resolvemos

1. Millones en criptomonedas se pierden permanentemente cuando los propietarios fallecen sin transferir claves privadas
2. Los mecanismos tradicionales de herencia (abogados, notarios) no funcionan con activos digitales
3. No existe una solución descentralizada y no intrusiva para verificar si alguien sigue vivo

### Nuestra Solución

Un sistema que:
- Monitorea pasivamente al usuario mediante su teléfono móvil
- Usa visión artificial para verificación facial con detección de liveness
- Analiza patrones de comportamiento (horarios, typing, uso de apps)
- Entrena un perceptrón personalizado por usuario
- Ejecuta la herencia automáticamente cuando se confirma inactividad prolongada

---

## Arquitectura del Sistema

```
┌─────────────────────────────────────────────────────────────────┐
│                    CAPA 1: CLIENTE MÓVIL                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │    Visión    │  │   Patrones   │  │  Perceptrón  │          │
│  │  Artificial  │  │   de Uso     │  │    Local     │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
└─────────┼─────────────────┼─────────────────┼──────────────────┘
          │                 │                 │
          └────────────────┬┴─────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                    CAPA 2: ORÁCULO                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Agregador   │  │  Validador   │  │  Publicador  │          │
│  │  de Señales  │  │  de Scores   │  │  On-Chain    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                CAPA 3: SMART CONTRACTS (Soroban)                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │    Vault     │  │ Proof of Life│  │ Beneficiary  │          │
│  │   Contract   │  │   Contract   │  │   Contract   │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    CAPA 4: STELLAR NETWORK                      │
│                        Soroban Runtime                          │
└─────────────────────────────────────────────────────────────────┘
```

---

## El Perceptrón - Corazón del Sistema

### Concepto

Cada usuario tiene un perceptrón personalizado que aprende sus patrones únicos de comportamiento. El modelo genera un "liveness score" (0-1) que indica la probabilidad de que el usuario esté vivo y activo.

### Formulación Matemática

```
z = Σ(wi * xi) + b
ŷ = σ(z) = 1 / (1 + e^(-z))
```

Donde:
- `x` = vector de features normalizadas (0-1)
- `w` = vector de pesos (personalizados por usuario)
- `b` = bias
- `ŷ` = probabilidad de vida (0-1)

### Vector de Features (10 dimensiones iniciales)

```python
features = {
    'x1': 'face_match_score',        # Similitud facial (0-1)
    'x2': 'liveness_score',          # Anti-spoofing (0-1)
    'x3': 'time_of_day_normality',   # ¿Hora típica de uso? (0-1)
    'x4': 'location_normality',      # ¿Ubicación típica? (0-1)
    'x5': 'typing_pattern_match',    # Patrón de escritura (0-1)
    'x6': 'app_usage_match',         # Uso de apps típico (0-1)
    'x7': 'movement_pattern_match',  # Patrón de movimiento (0-1)
    'x8': 'days_since_last_verify',  # Normalizado (0-1)
    'x9': 'session_duration_normal', # Duración típica (0-1)
    'x10': 'interaction_velocity',   # Velocidad de interacción (0-1)
}
```

### Fases del Modelo

1. **Calibración (2-4 semanas)**: Recolectar datos, todo etiquetado como "vivo" (y=1)
2. **Entrenamiento**: Ajustar pesos para que output → 1 con datos normales
3. **Operación**: Comparar comportamiento actual vs modelo
4. **Adaptación**: Ajuste lento y continuo para cambios legítimos

---

## Smart Contracts (Soroban/Rust)

### Vault Contract

Custodia los activos del usuario.

```rust
pub trait VaultTrait {
    fn create_vault(env: Env, owner: Address, token: Address) -> VaultId;
    fn deposit(env: Env, vault_id: VaultId, amount: i128, token: Address);
    fn withdraw(env: Env, vault_id: VaultId, amount: i128, token: Address);
    fn set_beneficiaries(env: Env, vault_id: VaultId, beneficiaries: Vec<Beneficiary>);
    fn get_status(env: Env, vault_id: VaultId) -> VaultStatus;
}
```

### Proof of Life Contract

Gestiona la verificación de vida.

```rust
pub trait ProofOfLifeTrait {
    fn register_model(env: Env, user: Address, weights: Vec<i128>, bias: i128);
    fn submit_verification(env: Env, user: Address, score: u32, source: VerificationSource, sig: BytesN<64>);
    fn update_model(env: Env, user: Address, new_weights: Vec<i128>, new_bias: i128);
    fn get_liveness_score(env: Env, user: Address) -> u32;
    fn emergency_checkin(env: Env, user: Address);
}
```

### Beneficiary Contract

Gestiona beneficiarios y distribución.

```rust
pub trait BeneficiaryTrait {
    fn add_beneficiary(env: Env, vault_id: VaultId, beneficiary: Beneficiary);
    fn remove_beneficiary(env: Env, vault_id: VaultId, beneficiary_address: Address);
    fn claim(env: Env, vault_id: VaultId, claimer: Address);
    fn get_beneficiaries(env: Env, vault_id: VaultId) -> Vec<Beneficiary>;
}
```

### Estructuras de Datos Clave

```rust
#[contracttype]
pub struct LifeModel {
    pub weights: Vec<i128>,      // Pesos (fixed-point, 6 decimales)
    pub bias: i128,
    pub version: u32,
    pub last_updated: u64,
    pub calibration_complete: bool,
    pub total_verifications: u64,
    pub avg_confidence: u32,     // 0-10000
    pub alert_threshold: u32,    // Default: 3000 (0.30)
    pub critical_threshold: u32, // Default: 1500 (0.15)
    pub grace_period_days: u32,  // Default: 30
}

#[contracttype]
pub struct Beneficiary {
    pub address: Address,
    pub percentage: u32,         // 0-10000 (100.00%)
    pub vesting_start: u64,
    pub vesting_duration: u64,
    pub conditions: Vec<Condition>,
}

#[contracttype]
pub enum VaultStatus {
    Active,
    Alert,
    GracePeriod,
    Triggered,
    Distributed,
}
```

---

## Estados del Sistema de Herencia

```
ACTIVE ──(score < 0.7)──► ALERT ──(score < 0.3)──► GRACE ──(timeout)──► TRIGGERED ──(claim)──► DISTRIBUTED
   ▲                        │                        │
   └────────(check-in)──────┴────────(check-in)──────┘
```

| Estado | Condición | Acción |
|--------|-----------|--------|
| ACTIVE | score > 0.7 | Monitoreo normal |
| ALERT | 0.3 < score < 0.7 | Incrementar verificaciones |
| GRACE | score < 0.3 o timeout en ALERT | Notificar, período de gracia (30 días default) |
| TRIGGERED | Timeout en GRACE | Habilitar claims de beneficiarios |
| DISTRIBUTED | Todos los assets reclamados | Estado terminal |

---

## Stack Tecnológico

### Smart Contracts
- **Lenguaje**: Rust
- **Plataforma**: Soroban (Stellar)
- **SDK**: soroban-sdk v21+

### Backend / Oráculo
- **Lenguaje**: Rust (preferido) o TypeScript
- **Framework**: Actix-web (Rust) o Fastify (TS)
- **Base de datos**: PostgreSQL
- **Cache/Jobs**: Redis + BullMQ
- **Stellar SDK**: stellar-sdk

### Cliente Móvil
- **Framework**: React Native
- **ML Runtime**: TensorFlow Lite
- **Face Detection**: ML Kit (Google) / Vision (Apple)
- **Wallet**: Freighter SDK integration

### Infraestructura
- **Hosting**: Fly.io o Railway
- **CI/CD**: GitHub Actions
- **Monitoreo**: Grafana + Prometheus

---

## Estructura de Directorios Propuesta

```
pulse-protocol/
├── contracts/                    # Smart Contracts Soroban
│   ├── vault/
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── proof-of-life/
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── beneficiary/
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   └── Cargo.toml               # Workspace
│
├── oracle/                       # Servicio de Oráculo
│   ├── src/
│   │   ├── main.rs
│   │   ├── aggregator.rs        # Agregación de señales
│   │   ├── publisher.rs         # Publicación on-chain
│   │   └── api.rs               # API REST
│   └── Cargo.toml
│
├── mobile/                       # App React Native
│   ├── src/
│   │   ├── services/
│   │   │   ├── vision.ts        # Módulo de visión
│   │   │   ├── patterns.ts      # Análisis de patrones
│   │   │   └── perceptron.ts    # Inferencia del modelo
│   │   ├── screens/
│   │   └── App.tsx
│   └── package.json
│
├── ml/                           # Modelos de ML
│   ├── perceptron/
│   │   ├── train.py
│   │   ├── inference.py
│   │   └── export_tflite.py
│   └── requirements.txt
│
├── docs/
│   └── whitepaper.pdf
│
└── README.md
```

---

## Consideraciones de Privacidad (CRÍTICO)

### NUNCA sale del dispositivo:
- Imágenes faciales
- Datos de ubicación exacta
- Contenido de mensajes/apps
- Datos biométricos raw

### SÍ se procesa/envía:
- Scores normalizados (0-1)
- Pesos del modelo (no reversibles)
- Timestamps de verificación
- Hashes de patrones

---

## Fases de Desarrollo

### Fase 1: MVP (6-8 semanas)
- [ ] Vault Contract básico
- [ ] Proof of Life con check-in manual
- [ ] Beneficiary Contract simple
- [ ] API del oráculo centralizado
- [ ] App móvil mínima funcional

### Fase 2: Integración AI (6-8 semanas)
- [ ] Módulo de visión artificial
- [ ] Análisis de patrones de comportamiento
- [ ] Perceptrón con entrenamiento on-device
- [ ] Sincronización de pesos con blockchain

### Fase 3: Robustez (4-6 semanas)
- [ ] Sistema de testigos
- [ ] Período de gracia con notificaciones
- [ ] Condiciones programables
- [ ] Auditoría de seguridad

---

## Notas Técnicas Importantes

### Representación de Punto Fijo en Soroban
Soroban no soporta floats. Usamos fixed-point con 6 decimales:
```rust
// 0.847523 se almacena como 847523i128
let weight_stored = (weight_real * 1_000_000.0) as i128;
let weight_real = weight_stored as f64 / 1_000_000.0;
```

### Thresholds por Defecto
- Alert: 0.30 (3000 en fixed-point)
- Critical: 0.15 (1500 en fixed-point)
- Grace Period: 30 días

### Frecuencia de Verificación
- Normal (ACTIVE): 1x cada 24-48 horas
- Alerta (ALERT): 1x cada 6-12 horas
- Gracia (GRACE): 1x cada 1-2 horas + notificaciones

---

## Links y Recursos

- [Soroban Docs](https://soroban.stellar.org/docs)
- [Stellar SDK](https://stellar.github.io/js-stellar-sdk/)
- [TensorFlow Lite](https://www.tensorflow.org/lite)
- [React Native](https://reactnative.dev/)
- [ML Kit Face Detection](https://developers.google.com/ml-kit/vision/face-detection)
