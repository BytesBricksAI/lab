# Proposal: bindings pyo3 del dominio SimPlant (sp_*) hacia Python

## Intent

Hoy el SDK de Python de SimPlant Lab (`rerun_py/`, paquete `simplant-lab-sdk`, módulo
`import simplant_lab`) expone **solo** la capa de logging/visualización heredada de Rerun
(`re_sdk`) y parte de `re_importer`. **Ningún crate de dominio `sp_*`** de
`crates/simplant/` está accesible desde Python.

Esto significa que toda la capacidad real del producto — modelar activos, adquirir datos de
planta, simular flowsheets, construir datasets de ML, correr pruebas de estrés y grabar a
`.rrd` — solo se puede orquestar desde Rust. Los usuarios de SimPlant (ingenieros de proceso,
data scientists, autores de demos y de entrenamiento RL) trabajan en Python. La frontera
Rust↔Python es el cuello de botella que impide usar el dominio desde notebooks, scripts de
entrenamiento y demos.

Queremos cerrar esa frontera exponiendo la API pública de los 12 crates `sp_*` a Python,
sin violar el ADR-0002 (el dominio se mantiene puro, sin `re_*` ni `pyo3`).

## Scope

### In scope
- **Nuevo crate puente `crates/simplant/sp_python`** que depende de los crates `sp_*` + `pyo3`,
  define todos los `#[pyclass]`/`#[pymethods]`/`#[pyfunction]` y exporta
  `pub fn register(py, m) -> PyResult<()>`.
- **Una línea de integración** en `rerun_py/src/python_bridge.rs` dentro del
  `#[pymodule] rerun_bindings`: `sp_python::register(py, m)?;`.
- **Wrappers Python por capacidad**, expuestos como submódulos `simplant_lab.<capability>`:
  - `kernel` — value objects (sp_kernel)
  - `asset_model` — catálogo de activos (sp_asset_model)
  - `acquisition` — sesiones + adapters `replay`/`modbus` (sp_acquisition, sp_acquisition_replay, sp_acquisition_modbus)
  - `simulation` — flowsheet + motor `engine` (sp_simulation, sp_sim_engine)
  - `ml_dataloop` — datasets + `dataframe_query` (sp_ml_dataloop, sp_dataframe_query)
  - `stress_testing` — pruebas de estrés (sp_stress_testing)
  - `recording` — recorder Rerun (sp_recording)
  - `types` — anti-corruption layer simplant.* (sp_types)
- **Type stubs `.pyi`** para cada submódulo expuesto (DX en Python).
- **Conversión de errores de dominio** Rust → excepciones Python.
- **Smoke tests en Python** que repliquen los flujos de las demos (`tanque_demo`, `sim_demo`,
  export de dataset) end-to-end desde Python.

### Out of scope
- Reescribir las demos Rust existentes a Python (se hará en un change posterior; acá solo
  se prueban los flujos con smoke tests).
- Exponer internals de `re_sdk`/`re_importer` ya cubiertos por el binding heredado.
- API async/streaming de adquisición en vivo desde Python (las sesiones se exponen en su
  forma síncrona/orquestada actual).
- Empaquetado/publicación en PyPI y wheels multiplataforma (queda al pipeline de release).
- Capa "pública" pura-Python tipo `Internal/Wrapper` de ARCHITECTURE.md más allá de stubs
  (se deja como mejora de DX futura; la primera entrega expone los `*Internal` nativos).

## Approach

Crear **un único crate puente** `sp_python` que aísla toda la frontera Python en un lugar
(Ley de Localidad del ARL). El crate:

1. Declara `sp_* = { path = ... }` + `pyo3` como dependencias.
2. Organiza un módulo Rust por capacidad (`src/kernel.rs`, `src/asset_model.rs`, …), cada uno
   con structs wrapper `pub struct PyXxx(DomainType)` e `impl` con `#[pymethods]`.
3. Mapea errores de dominio a `PyRuntimeError`/`PyValueError` con un helper compartido.
4. Expone `register(py, m)` que crea submódulos (`PyModule::new`) y registra clases/funciones.

`rerun_py` (el `cdylib` que maturin compila como `rerun_bindings`) agrega `sp_python` como
dependencia y lo registra con una sola línea. Así la zona upstream del fork
(`python_bridge.rs`) no se ensucia con tipos de dominio y los rebases siguen limpios.

Los wrappers siguen el patrón ya usado en `rerun_py/src/python_bridge.rs`
(`PyRecordingStream`, `PyMemorySinkStorage`): struct newtype + `#[pymethods]`, errores a
`PyRuntimeError::new_err`, `py.detach(...)` para soltar el GIL en operaciones largas.

El orden de implementación sigue el grafo de dependencias: `kernel` y `types` (hojas)
primero, luego `asset_model`, `simulation`, `stress_testing`, `ml_dataloop`, después
`acquisition` y los adapters (`replay`, `modbus`, `recording`, `dataframe_query`,
`sim_engine`).

## Affected areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/simplant/sp_python/` | New | Crate puente con todos los `#[pyclass]` + `register()` |
| `rerun_py/Cargo.toml` | Modified | Agrega `sp_python = { path = "../crates/simplant/sp_python" }` |
| `rerun_py/src/python_bridge.rs` | Modified | Una línea `sp_python::register(py, m)?;` en el `#[pymodule]` |
| `rerun_py/rerun_sdk/simplant_lab/` | Modified | Re-export de submódulos + ubicación de stubs `.pyi` |
| `Cargo.toml` (workspace) | Modified | Alta del miembro `crates/simplant/sp_python` |
| `rerun_py/tests/` | New | Smoke tests Python de los flujos de demos |
| `crates/simplant/sp_*` (dominio) | None | NO se tocan: el ADR-0002 los mantiene puros |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Tipos de dominio con genéricos/lifetimes no triviales de envolver en pyo3 | Med | Wrappers newtype + conversión explícita; exponer solo la API pública útil, no internals |
| `sp_recording`/`sp_dataframe_query` ya dependen de `re_*`: posible doble versión de tipos Arrow entre `sp_python` y `rerun_bindings` | Med | `sp_python` reusa exactamente las mismas versiones `re_*` del workspace; pinear vía Cargo workspace |
| Crecimiento del tiempo de compilación de `rerun_py` al sumar 12 crates | Low | Ya están en el workspace; `sp_python` agrega una sola arista de dep |
| Mapear ownership/`&mut` de aggregates (ej. `run_session(&mut session, ...)`) a la semántica Python | Med | Envolver en `RefCell`/métodos que consumen-y-devuelven; especificar caso por caso en cada spec |
| ADR-0002 violado por accidente (pyo3 colándose al dominio) | Low | El binding vive solo en `sp_python`; CI/compilador verifica que `sp_*` sigan sin `re_*`/`pyo3` |

## Rollback plan

El cambio es aditivo y aislado:
1. Quitar la línea `sp_python::register(py, m)?;` de `rerun_py/src/python_bridge.rs`.
2. Quitar la dependencia `sp_python` de `rerun_py/Cargo.toml`.
3. Quitar el miembro `crates/simplant/sp_python` del workspace `Cargo.toml`.
4. Borrar el directorio `crates/simplant/sp_python/`.

Como ningún crate de dominio ni la zona upstream se modifican estructuralmente, revertir
no afecta a `re_sdk` ni al binding heredado. Los `.rrd` y demos Rust existentes siguen
funcionando idénticos.

## Dependencies

- `pyo3` (misma versión/feature `extension-module`, `abi3-py310` que `rerun_py`).
- Workspace `re_*` pinned: `sp_python` debe usar las mismas versiones que `rerun_bindings`
  para no duplicar tipos Arrow.
- maturin (build ya existente de `rerun_py`).

## Success criteria

- [ ] `crates/simplant/sp_python` compila y `cargo build -p rerun_py` produce `rerun_bindings`
      con los submódulos de dominio registrados.
- [ ] Desde Python: `import simplant_lab; simplant_lab.kernel`, `.asset_model`,
      `.acquisition`, `.simulation`, `.ml_dataloop`, `.stress_testing`, `.recording`,
      `.types` están todos accesibles.
- [ ] Smoke test Python reproduce el flujo de `tanque_demo` (catálogo TOML → sesión →
      CsvReplaySource → RerunRecorder → `.rrd`) y verifica el número de batches.
- [ ] Smoke test Python reproduce el flujo de `sim_demo` (FlowsheetSpec draft→approve →
      Scenario → FirstOrderEngine → pasos) y verifica el estado final.
- [ ] Los crates `sp_*` de dominio siguen sin dependencias `re_*`/`pyo3` (ADR-0002 intacto).
- [ ] Cada submódulo expuesto tiene su `.pyi` con las firmas públicas.
