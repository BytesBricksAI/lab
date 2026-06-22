# 0002 — Arquitectura hexagonal del dominio

## Status

Accepted

## Context

SimPlant Lab debe modelar proceso Oil & Gas (tags ISA-5.1, jerarquía de activos, simulación, ML) sin acoplar el dominio al motor Rerun ni a drivers industriales. [`MIGRATION_PLAN.md`](../MIGRATION_PLAN.md) §3 P3 define capas Dominio → Aplicación → Infraestructura → Presentación con regla de dependencia estricta hacia adentro y patrón Aggregate.

[`IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md) confirma que la regla está **verificada por el compilador**: los crates de dominio puro declaran cero dependencias `re_*`.

## Decision

Organizar el dominio en `crates/simplant/*` con arquitectura hexagonal:

- **Dominio** (Rust puro, sin `re_*`): aggregates, value objects, eventos de plano de control.
- **Aplicación**: traits de puertos + casos de uso que orquestan aggregates.
- **Infraestructura**: adapters que implementan puertos sobre `re_*` o drivers externos.
- **Presentación**: CLI (`crates/top/simplant-lab-cli`) y vistas del viewer (futuro `sp_viewer_views`).

Crates de dominio implementados (todos sin deps `re_*` en `Cargo.toml`):

| Crate | Rol |
|-------|-----|
| `sp_kernel` | Value objects: `TagId`, `Quality`, `Measurement`, `MeasurementBatch` |
| `sp_asset_model` | Aggregates `Facility` / `Equipment` / `Tag`; puerto `AssetCatalogPort` |
| `sp_acquisition` | Aggregate `AcquisitionSession`; puertos `DataSourcePort`, `RecorderPort`; caso de uso `run_session` |
| `sp_simulation` | `FlowsheetSpec`, `Scenario`, `SimulationRun`; puerto `SimulatorPort` |
| `sp_stress_testing` | Aggregate `StressTest` con perfiles de carga y criterios pass/fail |
| `sp_ml_dataloop` | `DatasetSpec`, `DatasetManifest`; puerto `DatasetSinkPort` |

Adapters con deps `re_*` (solo infraestructura): `sp_recording`, `sp_types`.

Distinción plano de datos vs plano de control ([`MIGRATION_PLAN.md`](../MIGRATION_PLAN.md) §3 P4): mediciones a alta frecuencia fluyen como `MeasurementBatch`; eventos como `AcquisitionEvent` son dominio de baja frecuencia.

## Consequences

- **Positivas**: 115 tests en crates `sp_*` corren sin infraestructura Rerun; borrar un módulo (p. ej. `sp_simulation`) no rompe adquisición ni viewer.
- **Negativas**: más crates y wiring explícito; el composition root (`simplant-lab-cli`) debe cablear adapters manualmente.
- **Verificación**: `cargo build` de los 11 crates `sp_*` verde según [`IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md); inspección de `Cargo.toml` de `sp_kernel`, `sp_asset_model`, `sp_acquisition` confirma ausencia de `re_*`.
