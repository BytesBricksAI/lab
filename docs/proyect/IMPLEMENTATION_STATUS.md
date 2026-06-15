# Estado de implementación — SimPlant Lab

| Campo | Valor |
|---|---|
| **Fecha** | 2026-06-15 |
| **Base** | Plan [`MIGRATION_PLAN.md`](MIGRATION_PLAN.md) |
| **Método** | Implementación por subagentes (Cursor CLI `composer-2.5`), supervisada y verificada de forma independiente |
| **Build** | `cargo build` de los 11 crates `sp_*` + 2 demos: verde |
| **Tests** | **115 tests** verdes (`cargo test` de los crates `sp_*`) |
| **Seguridad** | Auditoría sin hallazgos (ver §3) |

---

## 1. Implementado y verificado

Todo bajo `crates/simplant/` (zona SimPlant) y `examples/simplant/`. La **regla hexagonal está verificada por el compilador**: los crates de dominio puro declaran **cero** dependencias `re_*`; solo los adapters tocan el motor Rerun.

### Fase 1 — Núcleo de dominio (completa)

| Crate | Rol | re_* | Tests |
|---|---|---|---|
| `sp_kernel` | Value objects: `TagId`, `UnitOfMeasure` (enum propio + conversiones), `Quality`, `EngineeringRange`, `AlarmLimits` (LL≤L<H≤HH), `TimeWindow`, `Measurement`, `MeasurementBatch` | 0 | 22 |
| `sp_asset_model` | Aggregates `Facility`/`Equipment`/`Tag` con invariantes + eventos; `AssetCatalogPort`; adapter TOML (carga vía constructores → nunca aggregates inválidos) | 0 | 15 |
| `sp_types` | Componentes/archetypes `simplant.*` sobre `re_sdk_types`: `Quality`, `ProcessVariableSample` (Scalars + quality), `TagMetadata` | sí | 3 |
| `sp_acquisition` | Dominio `AcquisitionSession` + puertos `DataSourcePort`/`RecorderPort` + caso de uso `run_session` | 0 | 11 |
| `sp_acquisition_replay` | Adapter `DataSourcePort` sobre CSV de historiador | 0 | 2 |
| `sp_recording` | Adapter `RecorderPort` sobre `re_sdk::RecordingStream` (timeline `plant_time`, único traductor dominio→store) | sí | 1 |

**Demo E2E** `examples/simplant/tanque_demo`: catálogo TOML → `validate()` → replay CSV → `.rrd` válido (magic `RRF2`). Cumple el criterio de aceptación de F1 salvo la inspección visual en el viewer (GUI, no verificable headless).

### Fase 2 — Adquisición industrial (Modbus)

| Crate | Rol | re_* | Tests |
|---|---|---|---|
| `sp_acquisition_modbus` | Driver **Modbus TCP, solo lectura** (OT safety) implementa `DataSourcePort` sobre `tokio-modbus`; mapeo registro→`Measurement` con `scale`/`offset`; **verificado E2E contra un servidor Modbus en `localhost`** | 0 | 7 |

OPC UA y MQTT/Sparkplug B comparten el mismo `DataSourcePort` y quedan pendientes (necesitan servidor OPC UA / broker MQTT para verificar; el crate `opcua` requiere el spike de §8.9).

### Fase 3 — Bucle de datos IA (núcleo)

| Crate | Rol | re_* | Tests |
|---|---|---|---|
| `sp_ml_dataloop` | `DatasetSpec` versionado + `DataSplit` **anti-leakage** + `DatasetManifest` reproducible + puertos + adapter `CsvDatasetSink` + caso de uso `export_dataset` (CSV long-format + manifest TOML reproducible) | 0 | 13 |

### Fase 4 — Simulación y stress (núcleo)

| Crate | Rol | re_* | Tests |
|---|---|---|---|
| `sp_simulation` | `FlowsheetSpec` con **análisis de grados de libertad** (variables = ecuaciones antes de aprobar), composiciones que suman 1, grafo bien formado; `Scenario`, `SimulationRun`; `SimulatorPort` | 0 | 20 |
| `sp_stress_testing` | `StressTest`: perfil de carga ≤ límite de diseño × factor de seguridad; criterios de aceptación; evaluación pass/fail | 0 | 14 |

### Fase 6 — Motor nativo (núcleo)

| Crate | Rol | re_* | Tests |
|---|---|---|---|
| `sp_sim_engine` | Motor nativo mínimo: `FirstOrderEngine` implementa `SimulatorPort` (dinámica de primer orden hacia las boundary conditions del escenario) | 0 | 7 |

**Demo de simulación E2E** `examples/simplant/sim_demo`: flowsheet aprobado → `Scenario` → motor nativo → trayectoria grabada al store en timeline `sim_time` (`.rrd` válido `RRF2`). Demuestra el loop simulación→store SIN DWSIM — el diferencial del plan ("HYSYS corre y descarta; SimPlant Lab corre y graba").

---

## 2. Pendiente (con motivo)

El **dominio y los contratos (puertos)** de estas capacidades están listos; lo que falta son **adapters de infraestructura externa que no son verificables honestamente en un entorno Linux headless sin el hardware/servicios correspondientes**. Implementarlos como stubs que "compilan pero no funcionan" se evitó deliberadamente.

| Ítem | Estado | Bloqueador |
|---|---|---|
| F2 — OPC UA / MQTT | **Modbus TCP hecho y verificado** (`sp_acquisition_modbus`, test contra servidor local) | OPC UA (crate `opcua`) y MQTT/Sparkplug B necesitan servidor/broker real para verificar |
| F3 — query sobre store real + export Parquet + toolkit Python | sink CSV + `export_dataset` **hechos y testeados** | `DataframeQueryPort` sobre `re_dataframe` (consultar el `.rrd` real) y sink Parquet (`re_parquet`) pendientes; toolkit `simplant_lab_process` requiere entorno Python con torch |
| F4 — sidecar DWSIM (`dwsim-bridge`) | `SimulatorPort` + `Scenario`/`SimulationRun` listos | DWSIM (.NET, **GPLv3**) no instalado; integración out-of-process por gRPC pendiente |
| F5 — surrogates + RL | — | Entrenamiento (PyTorch/GPU) no verificable aquí |
| F6 — `sp_thermo` (feos/CoolProp) | `PropertyPackagePort` (a definir) | Termodinámica industrial (feos/CoolProp FFI) requiere spike y validación física |
| F7 — editor de flowsheet, DEXPI, CAPE-OPEN | vistas custom previstas | GUI (egui), CAPE-OPEN es COM/Windows |
| Subcomandos CLI integrados (`assets validate`, `acquire`, `sim run`) | demo cubre el pipeline | Se difirió tocar el CLI principal para no arriesgar el build del binario del viewer |

**Próximo paso verificable**: adapter `DatasetSinkPort` → Parquet/CSV (export real del bucle IA) y un timeline `sim_time` en `sp_recording` para grabar corridas de simulación con el traductor oficial.

---

## 3. Auditoría de seguridad

Sin hallazgos. Verificado sobre `crates/simplant/`:

- **Sin secretos hardcoded**; sin ejecución de comandos del SO; **sin conexiones de red salientes** (sin telemetría — cumple política air-gap, §8.3).
- **OT safety**: `DataSourcePort` no expone operaciones de escritura — read-only por construcción (§8.4: no write-back a PLC/DCS).
- **Cero `unsafe`**; **cero `unwrap`/`expect`/`panic`** en código de producción (robustez para adquisición 24/7).
- **Sin dependencias GPL** en el grafo de los crates `sp_*` (el core conserva MIT/Apache, §8.1).
- **Validación de input**: todos los loaders (TOML/CSV) construyen aggregates únicamente vía sus constructores, que validan invariantes; un archivo inválido produce error, nunca un aggregate corrupto. Se cerró un bypass detectado en la carga de `FlowsheetSpec` (un TOML no puede declarar `Approved` un flowsheet con grados de libertad ≠ 0).
