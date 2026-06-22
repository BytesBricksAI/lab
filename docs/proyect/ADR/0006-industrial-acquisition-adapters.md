# 0006 — Adaptadores de adquisición industrial

## Status

Accepted

## Context

Oil & Gas requiere conectividad a PLCs/DCS e historiadores (Modbus TCP, OPC UA, MQTT Sparkplug B, replay CSV). El dominio de adquisición (`sp_acquisition`) no debe conocer protocolos concretos ni el store Rerun.

[`MIGRATION_PLAN.md`](../MIGRATION_PLAN.md) §4.2 y §5.2 listan drivers por adapter hexagonal. Política OT safety (§8.4): **read-only** — sin write-back a PLC/DCS. [`IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md) §2 documenta qué está hecho y qué está bloqueado por falta de servidores externos.

## Decision

Implementar adquisición como **adapters hexagonales** que implementan el puerto driven `DataSourcePort` definido en `sp_acquisition/src/application/ports.rs`:

```rust
pub trait DataSourcePort {
    fn subscribe(&self, bindings: &[TagBinding], policy: &SamplingPolicy)
        -> Result<Box<dyn MeasurementSource>>;
}
```

El trait **no expone escritura** — OT safety por construcción.

| Adapter | Crate | Estado | Dependencia |
|---------|-------|--------|-------------|
| Replay CSV historiador | `sp_acquisition_replay` | Hecho (2 tests) | Sin red |
| Modbus TCP read-only | `sp_acquisition_modbus` | Hecho y E2E verificado (7 tests) | `tokio-modbus` 0.16 (`Cargo.toml`) |
| OPC UA | (futuro `sp_acquisition_opcua`) | **Diferido** | Requiere servidor OPC UA real |
| MQTT / Sparkplug B | (futuro) | **Diferido** | Requiere broker MQTT |

`sp_acquisition_modbus` (`tcp_source.rs`, `address.rs`) mapea registros Modbus → `Measurement` con `scale`/`offset`, verificado contra servidor en `localhost`.

El caso de uso `run_session` (`sp_acquisition/src/application/run_session.rs`) orquesta sesión + `RecorderPort` (`sp_recording`). Demo E2E: `examples/simplant/tanque_demo` (catálogo TOML → replay CSV → `.rrd`).

OPC UA y MQTT comparten el mismo `DataSourcePort`; se implementarán cuando haya infraestructura verificable (spike `opcua` en plan §8.9) — no como stubs que compilan pero no funcionan.

## Consequences

- **Positivas**: un protocolo nuevo = un crate adapter sin tocar dominio; Modbus demostrado en entorno Linux headless; política air-gap respetada (sin telemetría saliente, §8.3).
- **Negativas**: cobertura industrial incompleta hasta F2; subcomandos CLI `acquire` integrados difieren para no arriesgar build del viewer ([`IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md) §2).
- **Verificación**: 11 tests en `sp_acquisition` + 7 en `sp_acquisition_modbus`; cero `unwrap`/`panic` en producción (auditoría §3).
