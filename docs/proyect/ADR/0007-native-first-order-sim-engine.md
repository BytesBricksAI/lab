# 0007 — Motor de simulación nativo de primer orden

## Status

Accepted

## Context

La simulación de procesos es un pilar de SimPlant Lab ([`GUIDELINES.md`](../GUIDELINES.md)): entrenamiento IA, stress testing, comparación sim vs planta. DWSIM (motor de referencia GPLv3) no está instalado en el entorno de desarrollo actual y su integración es F4 pendiente (ADR-0005).

Se necesita un camino **verificable sin DWSIM** que demuestre el loop simulación → store: "HYSYS corre y descarta; SimPlant Lab corre y graba" ([`MIGRATION_PLAN.md`](../MIGRATION_PLAN.md) §4.11, [`IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md) §1 Fase 6).

El dominio ya expone `SimulatorPort` en `sp_simulation/src/application/ports.rs` con aggregates `FlowsheetSpec`, `Scenario`, `SimulationRun` (20 tests, análisis de grados de libertad).

## Decision

Implementar `crates/simplant/sp_sim_engine` con `FirstOrderEngine` (`sp_sim_engine/src/first_order.rs`) que implementa `SimulatorPort`:

- Dinámica de **primer orden** hacia las boundary conditions del `Scenario` (constante de tiempo `tau_secs`).
- **Cero** dependencias `re_*` — motor puro de dominio/simulación.
- Capabilities declaradas vía `EngineCapabilities` del puerto; stepping con `initialize` / `step(dt)` / `finalize`.

Demo E2E verificable: `examples/simplant/sim_demo` — flowsheet aprobado → `Scenario` → `FirstOrderEngine` → trayectoria grabada al store en timeline `sim_time` (`.rrd` válido `RRF2`).

Este motor es el **camino incremental** hacia `sp_thermo` + solver nativo completo (F6 del plan); DWSIM queda como motor alternativo de validación cruzada, no como prerequisito de demos.

## Consequences

- **Positivas**: pipeline simulación→grabación demostrable hoy (7 tests en `sp_sim_engine`); `sp_stress_testing` puede evaluar contra perfiles sin motor externo; desarrollo paralelo de F4 (DWSIM) y F6 (nativo).
- **Negativas**: física simplificada — no reemplaza termodinámica industrial; resultados no son ingeniería de proceso hasta F6 (`sp_thermo`, feos/CoolProp).
- **Evolución**: mismo `FlowsheetSpec` TOML debe correr en `FirstOrderEngine` y futuro `sp_simulation_dwsim` (plan §4.11.2: spec independiente del motor); validación cruzada grabada en store cuando ambos existan.
