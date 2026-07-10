# 0003 — capa anti-corrupción `sp_types`

## Status

Accepted

## Context

El store Rerun usa tipos generados desde FlatBuffers (`namespace rerun` en `crates/store/re_sdk_types/definitions/rerun/`). Modificar `.fbs` y el codegen upstream (`crates/build/re_types_builder/`, `pixi run codegen`) agrandaría el diff contra Rerun y rompería compatibilidad de wire format con releases upstream.

[`MIGRATION_PLAN.md`](../MIGRATION_PLAN.md) §4.3 define la **Etapa A**: componentes manuales bajo namespace `simplant.*`, sin tocar codegen ni `.fbs`. El namespace FlatBuffers `rerun` permanece sin cambiar (plan §2.3).

## Decision

Introducir `crates/simplant/sp_types` como **capa anti-corrupción** entre conceptos de dominio SimPlant y el store columnar Rerun:

- Componentes y archetypes bajo namespace `simplant.*` (constantes en `sp_types/src/namespace.rs`: `simplant.archetypes.ProcessVariable`, `simplant.components.Quality`, etc.).
- Implementación manual con `re_types_core::ComponentDescriptor` y `AsComponents` — **sin** nuevos archivos `.fbs`.
- Tipos expuestos: `ProcessVariableSample`, `Quality`, `TagMetadata` (`sp_types/src/process_variable.rs`, `quality.rs`, `tag_metadata.rs`).

El adapter `sp_recording` (`RerunRecorder` en `sp_recording/src/recorder.rs`) es el **único traductor** dominio → store: convierte `MeasurementBatch` y `Tag` del dominio en archetypes `sp_types` y los escribe vía `re_sdk::RecordingStream` con timeline `plant_time`.

## Consequences

- **Positivas**: diff upstream mínimo; compatibilidad de `.rrd` y wire format preservada; dominio (`sp_kernel`, `sp_acquisition`) no importa tipos Rerun.
- **Negativas**: boilerplate manual; sin bindings C++ generados para componentes `simplant.*` en esta etapa; documentación de tipos no aparece en `docs/content/reference/types/` autogenerado.
- **Evolución**: una Etapa B (futura) podría registrar tipos en codegen; hoy el compilador y [`IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md) validan 3 tests en `sp_types` y el demo `examples/simplant/tanque_demo` produce `.rrd` legible.
