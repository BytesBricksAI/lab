# Architecture Decision Records — SimPlant Lab

Registro de decisiones arquitectónicas ya tomadas en el fork de Rerun hacia SimPlant Lab. Cada ADR está **grounded** en artefactos del repositorio (`crates/simplant/*`, zona upstream, `docs/proyect/*`).

Documentos de contexto: [`MIGRATION_PLAN.md`](../MIGRATION_PLAN.md) · [`GUIDELINES.md`](../GUIDELINES.md) · [`IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md) · [`UPSTREAM_DIFF.md`](../UPSTREAM_DIFF.md)

## Índice

| Número | Título | Status |
|--------|--------|--------|
| [0001](0001-fork-rerun-as-platform.md) | Forkear Rerun como plataforma base | Accepted |
| [0002](0002-hexagonal-domain-architecture.md) | Arquitectura hexagonal del dominio | Accepted |
| [0003](0003-anticorruption-layer-sp-types.md) | Capa anti-corrupción `sp_types` | Accepted |
| [0004](0004-rebranding-and-upstream-isolation.md) | Rebranding y aislamiento upstream | Accepted |
| [0005](0005-license-isolation-dwsim-gpl.md) | Aislamiento de licencia DWSIM (GPLv3) | Accepted |
| [0006](0006-industrial-acquisition-adapters.md) | Adaptadores de adquisición industrial | Accepted |
| [0007](0007-native-first-order-sim-engine.md) | Motor de simulación nativo de primer orden | Accepted |

## Formato

Cada ADR sigue: **Title**, **Status**, **Context**, **Decision**, **Consequences**.
