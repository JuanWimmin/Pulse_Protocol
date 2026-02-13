# Prompt Inicial para Claude Code - Pulse Protocol

## Cómo usar este prompt

Copia y pega el siguiente prompt en Claude Code después de haber colocado los archivos `CLAUDE.md` y `CLAUDE_CODE_CONTEXT.md` en la raíz de tu proyecto.

---

## PROMPT

```
Estoy iniciando el desarrollo de Pulse Protocol, un sistema de herencia criptográfica descentralizada sobre Stellar/Soroban.

Por favor:

1. Lee los archivos CLAUDE.md y CLAUDE_CODE_CONTEXT.md para entender el proyecto completo

2. Inicializa la estructura del proyecto siguiendo el layout propuesto en la documentación

3. Comenzaremos con la Fase 1 (MVP) - Smart Contracts. Necesito que:
   
   a) Crees el workspace de Cargo para los contratos Soroban
   
   b) Implementes el Vault Contract con las siguientes funcionalidades:
      - create_vault: Crear un vault para un usuario
      - deposit: Depositar tokens en el vault
      - withdraw: Retirar tokens (solo si el estado es ACTIVE)
      - get_balance: Consultar balance
      - get_status: Consultar estado del vault
   
   c) Define las estructuras de datos necesarias:
      - VaultStatus enum (Active, Alert, GracePeriod, Triggered, Distributed)
      - VaultInfo struct con owner, balances, status, timestamps, etc.

4. Usa las mejores prácticas de Soroban:
   - Storage types apropiados (Instance, Persistent, Temporary)
   - Manejo de errores con contract errors
   - Eventos para acciones importantes
   - Tests unitarios básicos

Empieza creando la estructura del proyecto y luego implementa el Vault Contract paso a paso.
```

---

## Prompts de Seguimiento Sugeridos

### Después del Vault Contract:

```
Excelente trabajo con el Vault Contract. Ahora implementemos el Proof of Life Contract:

1. Estructura para LifeModel (weights, bias, thresholds, metadata)
2. register_model: Registrar modelo inicial de un usuario
3. submit_verification: Recibir verificación del oráculo (con firma)
4. update_model: Actualizar pesos del perceptrón
5. get_liveness_score: Obtener score actual
6. check_and_update_status: Lógica para cambiar estados basado en scores

Incluye la lógica de transición de estados (ACTIVE → ALERT → GRACE → TRIGGERED) basada en los thresholds configurados.
```

### Después del Proof of Life Contract:

```
Ahora implementemos el Beneficiary Contract:

1. Estructura Beneficiary (address, percentage, vesting, conditions)
2. add_beneficiary: Agregar beneficiario a un vault
3. remove_beneficiary: Remover beneficiario
4. claim: Reclamar assets (solo si vault está en TRIGGERED)
5. get_claimable_amount: Calcular monto reclamable considerando vesting

Debe integrarse con el Vault Contract para verificar estados y transferir assets.
```

### Para el Backend/Oráculo:

```
Iniciemos el servicio de oráculo en Rust:

1. API REST con Actix-web:
   - POST /verify: Recibir scores del móvil
   - GET /status/:user: Consultar estado de un usuario
   
2. Servicio de agregación:
   - Validar scores recibidos
   - Calcular score agregado
   - Firmar con keypair del oráculo
   
3. Publicador on-chain:
   - Conectar con Stellar testnet
   - Llamar submit_verification en el contrato
   
4. Base de datos:
   - Modelo para historial de verificaciones
   - Modelo para configuración de usuarios

Usa stellar-sdk para la interacción con Soroban.
```

### Para la App Móvil:

```
Comencemos la app móvil en React Native:

1. Setup inicial con TypeScript
2. Integración con TensorFlow Lite para el perceptrón
3. Módulo de visión artificial:
   - Captura de cámara frontal
   - Detección facial con ML Kit
   - Liveness detection básica
   
4. Módulo de patrones:
   - Tracking de horarios de uso
   - Análisis de typing patterns
   - Recolección de features
   
5. Sincronización con backend:
   - Envío periódico de scores
   - Recepción de actualizaciones de modelo

Empieza con la estructura del proyecto y el módulo de visión.
```

---

## Tips para Trabajar con Claude Code

1. **Sé específico**: Menciona archivos, funciones y estructuras por nombre
2. **Pide tests**: Siempre solicita tests unitarios junto con la implementación
3. **Revisa incrementalmente**: Pide implementaciones pequeñas y revisa antes de continuar
4. **Usa el contexto**: Referencia siempre a CLAUDE_CODE_CONTEXT.md para mantener consistencia
5. **Documenta decisiones**: Pide que documente decisiones de diseño importantes

---

## Checklist de Archivos Necesarios

Antes de iniciar, asegúrate de tener en la raíz del proyecto:

- [ ] `CLAUDE.md` - Instrucciones rápidas para Claude Code
- [ ] `CLAUDE_CODE_CONTEXT.md` - Contexto completo del proyecto
- [ ] `pulse_whitepaper.pdf` - Referencia técnica detallada (opcional pero recomendado)

---

## Estructura Final Esperada (Fase 1)

```
pulse-protocol/
├── CLAUDE.md
├── CLAUDE_CODE_CONTEXT.md
├── contracts/
│   ├── Cargo.toml              # Workspace
│   ├── vault/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # Entry point
│   │       ├── contract.rs     # Implementación
│   │       ├── storage.rs      # Storage helpers
│   │       ├── types.rs        # Structs y enums
│   │       ├── errors.rs       # Contract errors
│   │       └── test.rs         # Tests
│   ├── proof-of-life/
│   │   └── ... (similar structure)
│   └── beneficiary/
│       └── ... (similar structure)
├── oracle/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── ...
└── README.md
```
