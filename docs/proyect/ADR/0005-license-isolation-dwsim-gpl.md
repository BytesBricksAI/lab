# 0005 — Aislamiento de licencia DWSIM (GPLv3)

## Status

Accepted

## Context

SimPlant Lab planea integrar DWSIM como motor de simulación de referencia (steady-state, hidrocarburos). DWSIM es **GPLv3**. El núcleo del fork hereda licencia **MIT/Apache-2.0** del upstream Rerun (workspace `license` en `Cargo.toml` raíz). Linkear DWSIM in-process convertiría el producto en obra derivada GPL.

[`MIGRATION_PLAN.md`](../MIGRATION_PLAN.md) §8.1 y §4.11.3 establecen: sidecar out-of-process, `dwsim-bridge` C#/.NET con licencia GPLv3 propia, comunicación por gRPC. [`IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md) §3 confirma: **sin dependencias GPL** en el grafo de crates `sp_*`.

## Decision

**Nunca** linkear DWSIM dentro de crates MIT/Apache. La integración prevista es:

| Componente | Ubicación | Licencia | Rol |
|------------|-----------|----------|-----|
| Core Rust `sp_*` | `crates/simplant/*` | MIT/Apache | Dominio, puertos, adapters sin GPL |
| Cliente gRPC | `sp_simulation_dwsim` (planificado F4) | MIT/Apache | Implementa `SimulatorPort` vía gRPC |
| Contrato protobuf | `crates/simplant/sp_simulation/proto/` (planificado) | MIT/Apache | `simulator.proto` — lado no-GPL |
| Sidecar | `bridges/dwsim-bridge/` (planificado) | GPLv3 | Envuelve DWSIM.Automation; proceso separado |

Frontera de proceso = *mere aggregation* (FAQ GPL): procesos separados comunicándose a arms-length. El sidecar puede crashear sin arrastrar adquisición ni viewer (supervisor + `RunFailed` en dominio).

Guardia prevista en CI: `cargo deny` veta deps GPL en el grafo Rust ([`MIGRATION_PLAN.md`](../MIGRATION_PLAN.md) §6.2). Distribución: DWSIM + bridge como componente opcional "Simulation Pack" del instalador, con fuentes GPL disponibles.

Hasta que el bridge exista, el camino verificable sin GPL es `sp_sim_engine` (ADR-0007).

## Consequences

- **Positivas**: el core comercializable conserva MIT/Apache; clientes pueden operar sin instalar DWSIM; auditoría de [`IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md) §3 sin hallazgos GPL.
- **Negativas**: latencia y operación de sidecar; build dual (Rust + `dotnet publish`); F4 bloqueado hasta tener DWSIM instalado para verificación E2E.
- **Operativas**: `sp_simulation` ya expone `SimulatorPort` desacoplado del motor; `sp_simulation_dwsim` y `bridges/dwsim-bridge` quedan pendientes según [`IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md) §2.
