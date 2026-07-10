# Tasks: bindings pyo3 del dominio SimPlant (sp_*) hacia Python

> Orden por dependencias del grafo `sp_*`: hojas (kernel/types) → modelos → orquestación →
> adapters. Cada capacidad agrega su submódulo a `register()` y compila de forma independiente.

## Phase 1: andamiaje del crate puente

- [x] 1.1 Crear `crates/simplant/sp_python/Cargo.toml` con `crate-type = ["rlib"]`, dep
      `pyo3` (features mínimas, SIN `extension-module`) y deps `path` a los 12 crates `sp_*`,
      usando las versiones `re_*` del workspace.
- [x] 1.2 Alta del miembro `crates/simplant/sp_python` en el `Cargo.toml` del workspace.
- [x] 1.3 Crear `crates/simplant/sp_python/src/error.rs` con `pub fn map_err<E:
      std::error::Error>(e: E) -> PyErr` (invariantes de dominio → `PyValueError`).
- [x] 1.4 Crear `crates/simplant/sp_python/src/lib.rs` con `pub fn register(py, parent) ->
      PyResult<()>` que declara los `mod` por capacidad y llama a cada `*::register` (stubs
      vacíos por ahora).
- [x] 1.5 Verificar: `cargo build -p sp_python` compila con el `register` vacío.

## Phase 2: capacidades hoja (kernel, types)

- [x] 2.1 `src/kernel.rs`: `#[pyclass]` para `PyTagId`, `PyMeasurement`, `PyMeasurementBatch`,
      `PyTimeWindow`, `PyEngineeringRange`, `PyAlarmLimits` (newtype + getters) y enums
      nativos `Quality`, `UnitOfMeasure`, `Dimension`. Constructores que mapean error con
      `map_err`. (specs/kernel: TagId, Quality, Measurement, TimeWindow, UnitOfMeasure)
- [x] 2.2 `src/kernel.rs`: exponer `Timestamp` como epoch-seconds `f64` (+ nanos `i64` y
      `__str__` ISO-8601) en `Measurement`/`TimeWindow`. (specs/kernel: Timestamp interoperable)
- [x] 2.3 `kernel::register` crea el submódulo `kernel` y registra clases; cablear en
      `lib.rs::register`.
- [x] 2.4 `src/types.rs`: `PyProcessVariableSample`, `PyTagMetadata`, enum `Quality` reuso,
      constantes de namespace y helper `field(...)`; `types::register`. (specs/types)
- [x] 2.5 Verificar: `cargo build -p rerun_py` tras agregar la dep `sp_python` y la línea
      `sp_python::register(py, m)?;` en `rerun_py/src/python_bridge.rs`; `maturin develop` y
      `python -c "import simplant_lab; simplant_lab.kernel.TagId('FT-101'); simplant_lab.types"`.

## Phase 3: modelos de dominio (asset_model, simulation, stress_testing, ml_dataloop)

- [x] 3.1 `src/asset_model.rs`: IDs (`PyFacilityId`/`PyAreaId`/`PyUnitId`/`PyEquipmentId`),
      `PyAssetCatalog` (getters `facility/equipment/tags/tag/equipment_by_id/validate`
      devolviendo secuencias), `PyFacility` (+ `add_area`/`add_unit` `&mut self`),
      `PyTomlCatalogRepository` (`load_catalog`). `asset_model::register`. (specs/asset-model)
- [x] 3.2 `src/simulation.rs`: value objects (`PyChemicalComponent`, `PyComposition`,
      `PyUnitOp`, `PyMaterialStream`, `PySpecification`, `PyBoundaryCondition`), enums
      (`UnitOpKind`, `ThermoPackage`, `EngineCapability`), IDs, `PyFlowsheetSpec`
      (`draft` staticmethod, `degrees_of_freedom`, `approve` `&mut self`), `PyScenario`
      (`approve` staticmethod). (specs/simulation: draft/DOF/approve/Scenario)
- [x] 3.3 `src/stress_testing.rs`: `PyLoadPoint`, `PyLoadProfile`, `PyDesignLimit`,
      `PySafetyFactor` (rechaza ≤0), `PyAcceptanceCriterion`, `PyMeasuredOutcome`, enum
      `StressTestState`, `PyStressTest` (`plan` staticmethod, `evaluate` `&mut self`).
      `stress_testing::register`. (specs/stress-testing)
- [x] 3.4 `src/ml_dataloop.rs`: `PyFeatureSpec`, `PyDataSplit` (rechaza solapamiento),
      `PyDatasetSpec` (define + getters); `ml_dataloop::register` (sin dataframe_query aún).
      (specs/ml-dataloop: DataSplit, DatasetSpec)
- [x] 3.5 Cablear 3.1–3.4 en `lib.rs::register`; `cargo build -p rerun_py` + smoke import de
      cada submódulo.

## Phase 4: orquestación y adapters de adquisición (acquisition + replay + modbus)

- [x] 4.1 `src/acquisition.rs`: `PyTagBinding`, `PySamplingPolicy`, enum `SessionState`,
      `PyAcquisitionSession` (`create` staticmethod, `start`/`stop` `&mut self`, getters).
      (specs/acquisition: crear sesión, transiciones de estado)
- [x] 4.2 `src/acquisition.rs`: submódulo `replay` con `PyCsvReplaySource(path)`. (specs/acquisition)
- [x] 4.3 `src/acquisition.rs`: submódulo `modbus` con `PyModbusTcpSource`, `parse_modbus_address`,
      `map_register`, enums `RegisterKind`/`ModbusPoint`. (specs/acquisition: Modbus)
- [x] 4.4 `src/acquisition.rs`: wrapper de fuente (`PyDataSource`/enum) que deref a `&dyn
      DataSourcePort`, y `#[pyfunction] run_session(session, catalog, source, recorder) -> u64`
      aceptando adapters nativos. (specs/acquisition: orquestación run_session)
- [x] 4.5 `acquisition::register` (con submódulos `replay`/`modbus`); cablear en `lib.rs`.

## Phase 5: adapters Rerun y motor (recording, dataframe_query, sim_engine)

- [x] 5.1 `src/recording.rs`: `PyRerunRecorder` (`to_file` staticmethod, `new(stream)` opcional,
      `flush`), constantes `PLANT_TIME`/`EVENTS_PATH`, `tag_entity_path`. Pinear versión `re_sdk`
      del workspace. `recording::register`. (specs/recording)
- [x] 5.2 `src/simulation.rs` submódulo `engine`: `PyFirstOrderEngine` (`new`, `initialize`,
      `step` devolviendo pares variable→valor, `current_time`, `value_of`) + unit ops
      (`mix/split/valve/pump/pipe`, `StreamState`). (specs/simulation: motor, unit ops)
- [x] 5.3 `src/ml_dataloop.rs` submódulo `dataframe_query`: `PyRrdDataframeQuery` (`open`,
      `query` → `PyQueryResult`/`PyTagSeries`). Pinear `re_chunk_store`/`re_dataframe`.
      (specs/ml-dataloop: RrdDataframeQuery)
- [x] 5.4 Cablear 5.1–5.3; `cargo build -p rerun_py` + `maturin develop`; verificar import de
      todos los submódulos (`recording`, `simulation.engine`, `ml_dataloop.dataframe_query`).

## Phase 6: type stubs y re-export del namespace

- [x] 6.1 Crear `rerun_py/rerun_sdk/simplant_lab/*.pyi` (uno por submódulo) con las firmas
      públicas de clases/funciones expuestas.
- [x] 6.2 Asegurar que `simplant_lab` re-exporta los submódulos del namespace (acceso
      `simplant_lab.kernel`, `.asset_model`, …, `.types`).

## Phase 7: tests de comportamiento (verifican escenarios de las specs)

- [x] 7.1 Tests unitarios Rust en `sp_python` (`#[cfg(test)]` con `Python::attach`): round-trip
      de `PyTagId`, `PyMeasurement`, rechazo de `TagId("")` y `TimeWindow` invertida.
      (specs/kernel)
- [x] 7.2 `rerun_py/tests/test_simplant_domain.py`: import de los 8 submódulos y construcción
      básica de un tipo central de cada uno. (todas las specs: "submódulo accesible")
- [x] 7.3 Smoke E2E `tanque_demo` desde Python: TOML → `AcquisitionSession.create` →
      `CsvReplaySource` → `RerunRecorder.to_file` → `run_session` → `flush`; assert nº de
      batches > 0 y `.rrd` legible. (specs/acquisition, specs/recording)
- [x] 7.4 Smoke E2E `sim_demo` desde Python: `FlowsheetSpec.draft` → `degrees_of_freedom()==0`
      → `approve()` → `Scenario.approve` → `FirstOrderEngine.initialize/step`; assert estado
      final. (specs/simulation)
- [x] 7.5 Tests de error: `approve()` con DOF≠0, `SafetyFactor(0.0)`, `DataSplit` con
      solapamiento, `parse_modbus_address("bogus")` elevan excepción Python. (specs varias)

## Phase 8: verificación del invariante ADR-0002 y cierre

- [x] 8.1 Verificar que ningún crate `sp_*` de dominio ganó dependencia `re_*`/`pyo3`
      (`cargo tree`/grep en sus `Cargo.toml`); el binding vive solo en `sp_python`.
- [x] 8.2 Actualizar `docs/proyect/UPSTREAM_DIFF.md` registrando la línea agregada en
      `rerun_py/src/python_bridge.rs` (toque a zona upstream del fork).
- [x] 8.3 `cargo build -p rerun_py` limpio + `pytest rerun_py/tests/test_simplant_domain.py`
      en verde; criterios de éxito del proposal cumplidos.
