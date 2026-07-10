# 0004 — rebranding y aislamiento de zona upstream

## Status

Accepted

## Context

El fork debe presentarse como **SimPlant Lab** (producto ByteBricks para Oil & Gas) manteniendo la capacidad de **mergear upstream** de Rerun periódicamente. Un rebranding disperso en cientos de archivos `re_*` haría cada rebase costoso e impredecible.

[`MIGRATION_PLAN.md`](../MIGRATION_PLAN.md) §3 P1 divide el repo en dos zonas con reglas distintas y exige anotar todo diff upstream en [`UPSTREAM_DIFF.md`](../UPSTREAM_DIFF.md).

## Decision

**Rebranding** centralizado y renombres de crates top-level:

| Antes | Después |
|-------|---------|
| `crates/top/rerun` | `crates/top/simplant-lab` |
| `crates/top/rerun-cli` | `crates/top/simplant-lab-cli` |
| Binario `rerun` | `simplant-lab` |

Branding de UI en un solo lugar: `crates/viewer/re_ui/src/branding.rs` (`PRODUCT_NAME = "SimPlant-Lab"`, `PRODUCT_NAME_LOWERCASE = "simplant-lab"`). El SDK spawn busca `simplant-lab` (`crates/top/re_sdk/src/spawn.rs`). Python: módulo `simplant_lab`, wheel `simplant-lab-sdk`, shim `rerun` con `DeprecationWarning` (`rerun_py/`). Tareas `pixi.toml`: `simplant-lab*` con aliases `rerun*`.

**Aislamiento upstream**: zona `crates/{store,viewer,utils,build}/*`, `rerun_py`, `rerun_cpp`, `docs/snippets` — solo se toca para (a) branding vía `re_ui::branding`, (b) extensiones en puntos oficiales (`BUILTIN_IMPORTERS`, `add_view_class`), (c) bugfixes upstreameables. Zona SimPlant (`crates/simplant/*`, `examples/simplant/*`) evoluciona libre.

[`UPSTREAM_DIFF.md`](../UPSTREAM_DIFF.md) es el registro vivo (511 renames, 533 modificados en la rama `feat/simplant-domain-crates`). **No se cambia** el namespace FlatBuffers `rerun` ni la extensión `.rrd`.

## Consequences

- **Positivas**: identidad de producto coherente; presupuesto de diff auditable antes de cada merge upstream; extensiones Oil & Gas no contaminan el core.
- **Negativas**: coexistencia temporal de strings `rerun` en APIs internas (`rerun_bindings` PyO3, namespace C++ `rerun::`); Fase 0 del plan aún tiene pendientes menores (assets visuales, `SIMPLANT_LAB_*` env vars).
- **Operativas**: cualquier edición nueva en zona upstream debe documentarse en `UPSTREAM_DIFF.md` con motivo; ver heurística en [`MIGRATION_PLAN.md`](../MIGRATION_PLAN.md) §3 P1.
