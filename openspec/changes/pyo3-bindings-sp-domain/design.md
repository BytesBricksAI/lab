# Design: bindings pyo3 del dominio SimPlant (sp_*) hacia Python

## Technical approach

Crear un único crate puente `crates/simplant/sp_python` (`crate-type` = `rlib`) que depende
de los 12 crates `sp_*` + `pyo3` y concentra TODA la frontera Rust↔Python (Ley de Localidad,
ARL). El crate organiza un módulo Rust por capacidad y expone una función pública
`register(py, parent) -> PyResult<()>` que construye los submódulos pyo3 y registra clases y
funciones. El `cdylib` `rerun_py` (que maturin compila como `rerun_bindings`) agrega
`sp_python` como dependencia y lo invoca con **una línea** dentro de su `#[pymodule]`.

Los wrappers siguen el patrón ya presente en `rerun_py/src/python_bridge.rs`: struct newtype
`pub struct PyXxx(DomainType)` + `impl` con `#[pymethods]`, errores de dominio mapeados a
excepciones Python, y `py.detach(...)` para soltar el GIL en operaciones de E/S largas.

Ningún crate `sp_*` se modifica: el ADR-0002 (dominio puro sin `re_*`/`pyo3`) queda intacto.

## Architecture decisions

### Decisión: crate puente único `sp_python` en lugar de submódulos en rerun_py

**Choice**: Un crate dedicado `crates/simplant/sp_python` que reúne todos los `#[pyclass]`.
**Alternatives considered**: (a) Definir los wrappers como submódulos dentro de
`rerun_py/src/simplant_*.rs`, junto al binding de `re_sdk`. (b) Un crate `*_py` por cada
crate de dominio (12 crates puente).
**Rationale**: (a) mezcla la frontera del dominio con la zona upstream del fork, ensuciando
`python_bridge.rs` y generando ruido/conflictos en cada rebase contra Rerun. (b) explota la
cantidad de crates y duplica andamiaje (Cargo.toml, deps pyo3) sin beneficio. Un único crate
puente cumple la Ley de Localidad: la frontera Python vive en un solo lugar, se testea y
evoluciona como unidad, y `rerun_py` solo gana **una arista de dependencia** y una línea de
registro. Decisión confirmada con el usuario.

### Decisión: `crate-type = ["rlib"]`, no `cdylib`

**Choice**: `sp_python` es una `rlib` normal; el único `cdylib`/`extension-module` sigue
siendo `rerun_py`.
**Alternatives considered**: Hacer `sp_python` su propio `extension-module` importable como
`simplant_python` aparte.
**Rationale**: Python ya importa `rerun_bindings` (un solo `.so` vía maturin). Un segundo
módulo de extensión duplicaría la inicialización de pyo3, el feature `extension-module` y
arriesgaría dos copias de tipos Arrow/`re_*`. Como `rlib`, `sp_python` se enlaza dentro de
`rerun_bindings`; pyo3 con feature `extension-module` lo provee `rerun_py`. `sp_python`
depende de `pyo3` **sin** `extension-module` (solo la API para definir clases).

### Decisión: newtype wrapper + `#[pymethods]`, con `Clone` cuando el dominio lo permite

**Choice**: `pub struct PyXxx(pub DomainType)` y mapear getters/constructores. Para value
objects `Clone` (TagId, Measurement, Quality, UnitOp, etc.) se devuelven por valor clonando.
**Alternatives considered**: `#[pyclass] enum` nativo de pyo3 para los enums C-like.
**Rationale**: pyo3 soporta `#[pyclass]` sobre enums C-like — se usa para `Quality`,
`UnitOfMeasure`, `Dimension`, `UnitOpKind`, `ThermoPackage`, `EngineCapability`,
`RegisterKind`, `SessionState`, `FlowsheetState`, `StressTestState` (mapeo directo, sin
newtype). Para structs con campos privados e invariantes (TagId, EngineeringRange,
FlowsheetSpec…) el newtype preserva la encapsulación del dominio.

### Decisión: mutación con interior mutability — `&mut self` y métodos que consumen `self`

**Choice**: Los aggregates con state machine se envuelven de forma que pyo3 pueda mutarlos:
- Métodos `&mut self` (ej. `AcquisitionSession::start/stop`, `FlowsheetSpec::approve`,
  `Facility::add_area`, `StressTest::evaluate`, `FirstOrderEngine::step`): se exponen como
  `#[pymethods]` que toman `&mut self` — pyo3 lo permite vía `PyRefMut` automático sobre el
  `#[pyclass]`. El wrapper guarda el `DomainType` directamente.
- Métodos que **consumen** `self` y devuelven `(Self, Event)` (ej. `DatasetSpec::revise`,
  `FlowsheetSpec::revise`): se exponen tomando `&mut self`, clonando internamente
  (`let new = self.0.clone().revise(...)?; self.0 = new;`) o, si el tipo no es `Clone`, con
  patrón take vía `Option<DomainType>` dentro del wrapper.

**Alternatives considered**: Exponer todo como inmutable y obligar a re-crear objetos en
Python.
**Rationale**: El flujo natural en Python (`session.start()`, `flowsheet.approve()`) exige
mutación in-place. pyo3 maneja `&mut self` de forma idiomática; el coste de clonar en `revise`
es despreciable frente a la claridad de API. Se especifica caso por caso en cada delta spec.

### Decisión: puertos (`&dyn DataSourcePort`/`RecorderPort`/`SimulatorPort`) — fronteras concretas

**Choice**: Las funciones de orquestación que reciben trait objects (`run_session(&mut
session, &catalog, &dyn DataSourcePort, &dyn RecorderPort)`) se exponen como `#[pyfunction]`
que aceptan los **wrappers concretos** de los adapters disponibles (PyCsvReplaySource,
PyModbusTcpSource, PyRerunRecorder) y los des-referencian al `&dyn` correspondiente.
**Alternatives considered**: Permitir implementar los puertos en Python puro (trait object
que llama de vuelta a Python).
**Rationale**: Implementar `DataSourcePort` desde Python requiere un trampolín pyo3 con
re-entrada del GIL y manejo de errores cross-language — alto costo y fuera de scope. En la
primera entrega Python **compone** adapters nativos (replay, modbus, recorder), que es
exactamente lo que hacen las demos. Implementar puertos en Python se marca como trabajo
futuro en la spec de acquisition.

### Decisión: `Timestamp`/`TimeWindow` (jiff) ↔ Python

**Choice**: Exponer `Timestamp` como segundos epoch `f64` (y/o nanos `i64`) y string ISO-8601
en `__str__`. `TimeWindow.new(start, end)` acepta los mismos tipos numéricos.
**Alternatives considered**: Mapear a `datetime.datetime` de Python.
**Rationale**: El dominio usa `jiff::Timestamp`. La conversión a `datetime` agrega
dependencia de tz y ambigüedad; `f64` epoch + ISO string cubre logging/queries sin fricción y
es trivial de revertir. Se documenta en la spec de kernel.

### Decisión: reuso estricto de versiones `re_*` del workspace

**Choice**: `sp_python` (al depender de `sp_recording`, `sp_dataframe_query`, `sp_types`) usa
las MISMAS versiones `re_*` que `rerun_bindings`, pinneadas por el `Cargo.toml` del workspace.
**Rationale**: Evita dos copias de tipos Arrow/`re_chunk_store`/`re_sdk` en el mismo `.so`,
que romperían en runtime. El workspace ya centraliza estas versiones.

## Data flow

```
                         ┌──────────────────────────────────────────┐
   Python (simplant_lab) │  import simplant_lab                     │
                         │  simplant_lab.simulation.FlowsheetSpec…  │
                         └───────────────────┬──────────────────────┘
                                             │  (rerun_bindings .so)
                         ┌───────────────────▼──────────────────────┐
   rerun_py (cdylib)     │  #[pymodule] rerun_bindings {            │
   python_bridge.rs      │      …binding re_sdk…                    │
                         │      sp_python::register(py, m)?;  ◄──── 1 línea
                         └───────────────────┬──────────────────────┘
                                             │
                         ┌───────────────────▼──────────────────────┐
   crates/simplant/      │  sp_python::register → submódulos:       │
   sp_python (rlib)      │   kernel · asset_model · acquisition ·   │
   PyXxx(DomainType)     │   simulation · ml_dataloop ·             │
                         │   stress_testing · recording · types     │
                         └───────────────────┬──────────────────────┘
                                             │  (deref a tipos de dominio)
                         ┌───────────────────▼──────────────────────┐
   crates/simplant/sp_*  │  Dominio PURO (sin re_*/pyo3)  ── ADR-0002│
   (sin cambios)         │  TagId · AssetCatalog · FlowsheetSpec …  │
                         └──────────────────────────────────────────┘
```

## File changes

| File | Action | Description |
|------|--------|-------------|
| `crates/simplant/sp_python/Cargo.toml` | Create | Deps: 12 `sp_*` (path) + `pyo3` (sin `extension-module`); `crate-type=["rlib"]` |
| `crates/simplant/sp_python/src/lib.rs` | Create | `pub fn register(py, parent)`; `mod` por capacidad; helper de errores |
| `crates/simplant/sp_python/src/error.rs` | Create | `fn map_err<E: Error>(e) -> PyErr` (PyValueError/PyRuntimeError) |
| `crates/simplant/sp_python/src/kernel.rs` | Create | PyTagId, PyQuality, PyMeasurement, PyMeasurementBatch, PyTimeWindow, PyUnitOfMeasure, PyEngineeringRange, PyAlarmLimits |
| `crates/simplant/sp_python/src/asset_model.rs` | Create | PyAssetCatalog, PyFacility, PyArea, PyProcessUnit, PyTomlCatalogRepository, IDs |
| `crates/simplant/sp_python/src/acquisition.rs` | Create | PyAcquisitionSession, PyTagBinding, PySamplingPolicy, `run_session`, submód `replay`/`modbus` |
| `crates/simplant/sp_python/src/simulation.rs` | Create | PyFlowsheetSpec, PyScenario, PyUnitOp, PyMaterialStream, …, submód `engine` (PyFirstOrderEngine) |
| `crates/simplant/sp_python/src/ml_dataloop.rs` | Create | PyDatasetSpec, PyDataSplit, PyFeatureSpec, submód `dataframe_query` (PyRrdDataframeQuery) |
| `crates/simplant/sp_python/src/stress_testing.rs` | Create | PyStressTest, PyLoadProfile, PyLoadPoint, PyDesignLimit, PySafetyFactor, … |
| `crates/simplant/sp_python/src/recording.rs` | Create | PyRerunRecorder, constantes, `tag_entity_path` |
| `crates/simplant/sp_python/src/types.rs` | Create | PyProcessVariableSample, PyTagMetadata, constantes namespace |
| `Cargo.toml` (workspace) | Modify | Alta del miembro `crates/simplant/sp_python` |
| `rerun_py/Cargo.toml` | Modify | `sp_python = { path = "../crates/simplant/sp_python" }` |
| `rerun_py/src/python_bridge.rs` | Modify | `sp_python::register(py, m)?;` dentro del `#[pymodule]` |
| `rerun_py/rerun_sdk/simplant_lab/__init__.py` (o equiv.) | Modify | Re-export de submódulos del namespace |
| `rerun_py/rerun_sdk/simplant_lab/*.pyi` | Create | Type stubs por capacidad |
| `rerun_py/tests/test_simplant_domain.py` | Create | Smoke tests de los flujos de demos |

## Interfaces / contracts

Función de registro (contrato central):

```rust
// crates/simplant/sp_python/src/lib.rs
pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    kernel::register(py, parent)?;
    asset_model::register(py, parent)?;
    acquisition::register(py, parent)?;
    simulation::register(py, parent)?;
    ml_dataloop::register(py, parent)?;
    stress_testing::register(py, parent)?;
    recording::register(py, parent)?;
    types::register(py, parent)?;
    Ok(())
}
```

Patrón newtype + error mapping (ejemplo kernel):

```rust
#[pyclass(name = "TagId", module = "simplant_lab.kernel")]
#[derive(Clone)]
pub struct PyTagId(pub sp_kernel::TagId);

#[pymethods]
impl PyTagId {
    #[new]
    fn new(raw: String) -> PyResult<Self> {
        sp_kernel::TagId::new(raw).map(PyTagId).map_err(map_err)
    }
    fn as_str(&self) -> &str { self.0.as_str() }
    fn __str__(&self) -> String { self.0.as_str().to_owned() }
}
```

Patrón mutador `&mut self` (ejemplo simulation):

```rust
#[pymethods]
impl PyFlowsheetSpec {
    #[staticmethod]
    fn draft(/* ChemicalComponent…, UnitOp…, … */) -> PyResult<Self> { /* … */ }
    fn degrees_of_freedom(&self) -> i64 { self.0.degrees_of_freedom() }
    fn approve(&mut self) -> PyResult<()> { self.0.approve().map(|_| ()).map_err(map_err) }
}
```

Patrón orquestación con puertos concretos (acquisition):

```rust
#[pyfunction]
fn run_session(
    session: &mut PyAcquisitionSession,
    catalog: &PyAssetCatalog,
    source: &PyDataSource,    // enum/wrapper que deref a &dyn DataSourcePort
    recorder: &PyRerunRecorder, // deref a &dyn RecorderPort
) -> PyResult<u64> {
    sp_acquisition::run_session(&mut session.0, &catalog.0, source.as_port(), &recorder.0)
        .map_err(map_err)
}
```

Mapeo de errores:

```rust
// crates/simplant/sp_python/src/error.rs
pub fn map_err<E: std::error::Error>(e: E) -> PyErr {
    PyValueError::new_err(e.to_string())  // invariantes de dominio → ValueError
}
```

## Testing strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit (Rust) | Cada wrapper construye/convierte sin perder semántica | `#[cfg(test)]` en `sp_python` con `pyo3::Python::attach`, comparando contra el tipo de dominio |
| Build | `cargo build -p rerun_py` enlaza `sp_python`; `maturin develop` produce `.so` | CI: compilar y `python -c "import simplant_lab; …"` |
| Integration (Python) | Cada submódulo importable; constructores y getters | `pytest` en `rerun_py/tests/test_simplant_domain.py` |
| E2E (Python) | Flujo `tanque_demo` (TOML→sesión→CsvReplaySource→RerunRecorder→.rrd) y `sim_demo` (draft→approve→engine→steps) replicados desde Python | `pytest` comparando nº de batches / estado final contra los valores conocidos de las demos Rust |

## Migration / rollout

No migration required. El cambio es aditivo:
- Implementación por capacidad en orden de dependencias (kernel/types → asset_model →
  simulation/stress_testing/ml_dataloop → acquisition+adapters → recording/dataframe_query/
  sim_engine).
- Cada capacidad puede mergear y compilar de forma independiente: `register` agrega su
  submódulo sin tocar las demás.
- Rollback = quitar la línea de registro + la dep + el crate (ver proposal).

## Open questions

- [ ] Implementar los puertos (`DataSourcePort`, `SimulatorPort`) **desde Python puro** —
      diferido a change posterior; ¿hay demanda real de motores/fuentes en Python?
- [ ] ¿Exponer la capa pública pura-Python `Internal/Wrapper` (ARCHITECTURE.md:174-188) o
      basta con los `*Internal` nativos + stubs en la primera entrega? (propuesta: stubs ahora,
      wrappers de DX después).
- [ ] `Timestamp`: ¿`f64` epoch-seconds es suficiente o algún consumidor necesita nanos `i64`
      sin pérdida? (propuesta: exponer ambos accesores).
