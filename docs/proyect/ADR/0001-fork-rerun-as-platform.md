# 0001 — forkear Rerun como plataforma base

## Status

Accepted

## Context

SimPlant Lab necesita una capa de datos multimodal y de múltiples velocidades para Oil & Gas: telemetría indexada por tiempo, grabación `.rrd`, viewer extensible, SDKs multi-lenguaje y export a Parquet/DataFusion. Construir todo desde cero implicaría reimplementar store columnar (Arrow), transporte gRPC, timelines y un viewer inmediato (egui/wgpu).

El fork parte de Rerun `0.33.0-alpha.1+dev` (ver [`MIGRATION_PLAN.md`](../MIGRATION_PLAN.md) §2.1). [`GUIDELINES.md`](../GUIDELINES.md) establece explícitamente usar Rerun como base para SimPlant Lab.

## Decision

Forkear Rerun y conservar sus crates `re_*` como **motor genérico intacto** (store, viewer, timeline, transporte). Todo lo específico de Oil & Gas vive en módulos nuevos bajo `crates/simplant/*`, conectados al motor solo por puntos de extensión oficiales (importers, lenses, componentes propios, vistas custom).

Capacidades heredadas que justifican el fork:

| Capacidad | Crates upstream |
|-----------|-----------------|
| Store columnar + timelines | `re_chunk`, `re_chunk_store`, `re_entity_db`, `re_log_types` |
| Transporte y grabación | `re_grpc_server`, `re_grpc_client`, `re_log_encoding` |
| Dataframes / Parquet | `re_dataframe`, `re_datafusion`, `re_parquet` |
| Viewer extensible | `re_viewer`, `re_view_*` |
| SDK | `re_sdk`, `rerun_py` |

Primer extension point Oil & Gas ya validado: `DxfImporter` en `crates/store/re_importer` (plan §2.3).

## Consequences

- **Positivas**: time-to-market acotado; merge upstream viable si el diff en zona `re_*` se mantiene mínimo (ver ADR-0004); demos E2E reutilizan el pipeline de grabación existente (`examples/simplant/tanque_demo` → `.rrd` con magic `RRF2`).
- **Negativas**: dependencia de la evolución de Rerun; el equipo debe respetar la frontera upstream/SimPlant documentada en [`MIGRATION_PLAN.md`](../MIGRATION_PLAN.md) §3 P1.
- **Operativas**: build y tareas documentadas en [`AGENTS.md`](../../../AGENTS.md) (`pixi run rerun-build`, `pixi run simplant-lab-build`); el binario de usuario es `crates/top/simplant-lab`.
