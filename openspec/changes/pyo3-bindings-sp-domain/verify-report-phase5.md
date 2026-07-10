# Verification report — phase 5

**Change**: `pyo3-bindings-sp-domain`
**Scope**: Phase 5 only (tasks 5.1–5.4 — recording, simulation.engine, ml_dataloop.dataframe_query)
**Verified at**: 2026-06-28
**Workspace**: `/home/m4s1t4/Work/Enprendimiento/Proyectos/SimPlant/SimPlant-v2/lab`

---

## Completeness

| Metric | Value |
|--------|-------|
| Phase 5 tasks total | 4 |
| Phase 5 tasks marked complete in `tasks.md` | 4 |
| Phase 5 tasks implemented in source | 4 |
| Phase 5 tasks incomplete (implementation) | 0 |
| Phase 5 tasks incomplete (runtime smoke) | 1 (5.4 maturin/import) |

| Task | Marked | Evidence |
|------|--------|----------|
| 5.1 `PyRerunRecorder`, constants, `tag_entity_path`, `recording::register` | ✅ `[x]` | `crates/simplant/sp_python/src/recording.rs:1–73` |
| 5.2 submódulo `simulation.engine`: `PyFirstOrderEngine`, unit ops, `StreamState` | ✅ `[x]` | `crates/simplant/sp_python/src/simulation.rs:424–568` |
| 5.3 submódulo `ml_dataloop.dataframe_query`: `PyRrdDataframeQuery`, `PyQueryResult`, `PyTagSeries` | ✅ `[x]` | `crates/simplant/sp_python/src/ml_dataloop.rs:120–203` |
| 5.4 Cablear 5.1–5.3; build + maturin + import smoke | ⚠️ Partial | `lib.rs:37` wires `recording::register`; builds pass; maturin/import blocked (see Build Verification) |

---

## Build verification

### Step 0 — pyo3 config

```bash
pixi run ensure-pyo3-build-cfg
```

**Result:** ✅ Exit 0

```
 WARN the lock file is up-to-date but uses an older format (v6), run `pixi lock` to upgrade to v7 for improved reproducibility
Generated /home/m4s1t4/Work/Enprendimiento/Proyectos/SimPlant/SimPlant-v2/lab/rerun_py/pyo3-build.cfg
```

### Step 1 — `cargo build -p sp_python`

```bash
cargo build -p sp_python
```

**Result:** ✅ Exit 0

```
    Finished `dev` profile [optimized] target(s) in 0.28s
```

### Step 2 — `cargo build -p rerun_py`

```bash
cargo build -p rerun_py
```

**Result:** ✅ Exit 0

```
   Compiling rerun_py v0.33.0-alpha.1+dev (/home/m4s1t4/Work/Enprendimiento/Proyectos/SimPlant/SimPlant-v2/lab/rerun_py)
    Finished `dev` profile [optimized] target(s) in 23.04s
```

### Step 3 — `maturin develop` (task 5.4)

```bash
cd rerun_py && pixi run maturin develop
```

**Result:** ❌ Exit 1 — missing `rerun` CLI artifact

```
ERROR: Expected to find `rerun` at `"/home/m4s1t4/Work/Enprendimiento/Proyectos/SimPlant/SimPlant-v2/lab/rerun_py/rerun_sdk/rerun_cli/rerun"`.
💥 maturin failed
```

### Step 4 — Python import smoke (task 5.4)

```bash
pixi run python -c "import simplant_lab; simplant_lab.recording.RerunRecorder; simplant_lab.simulation.engine.FirstOrderEngine; simplant_lab.ml_dataloop.dataframe_query.RrdDataframeQuery"
```

**Result:** ❌ Exit 1 — module not installed (maturin blocked)

```
ModuleNotFoundError: No module named 'simplant_lab'
```

---

## Correctness (Specs)

### `specs/recording/spec.md`

| Requirement | Status | Notes |
|------------|--------|-------|
| Submódulo `simplant_lab.recording` accesible | ✅ Implemented | `recording::register` adds submodule via `attach_simplant_submodule` |
| `RerunRecorder`, `PLANT_TIME`, `EVENTS_PATH`, `tag_entity_path` | ✅ Implemented | `recording.rs:65–70` — constants re-exported from `sp_recording`; `py_tag_entity_path` wraps domain helper |
| `RerunRecorder.to_file(app_id, path)` | ✅ Implemented | Static method with `py.detach` for I/O; domain errors → `map_err` → Python exception |
| `RerunRecorder(stream)` optional constructor | ✅ Implemented | `#[new]` delegates to `recorder_from_py` via factory hook registered in `python_bridge.rs:384–396` |
| `flush()` | ✅ Implemented | `recording.rs:54–56` delegates to domain `RerunRecorder::flush` |
| `re_sdk` workspace pinning | ✅ Implemented | `sp_python/Cargo.toml:22` uses `re_sdk.workspace = true` (=0.33.0-alpha.1) |

**Scenarios Coverage:**

| Scenario | Status |
|----------|--------|
| Importar el submódulo | ⚠️ Compile-only (runtime blocked by maturin) |
| Recorder a archivo | ⚠️ Source present; no runtime test |
| Flush deja el .rrd consistente | ⚠️ Source present; E2E deferred to Phase 7.3 |
| Ruta de entidad para un tag | ⚠️ Source present; no runtime test |

### `specs/simulation/spec.md` (engine submódulo)

| Requirement | Status | Notes |
|------------|--------|-------|
| Submódulo `simplant_lab.simulation.engine` con `FirstOrderEngine` | ✅ Implemented | `simulation.rs:557–567` registers `engine` submodule under `simulation` |
| `FirstOrderEngine(tau_secs)` + `initialize(scenario)` | ✅ Implemented | `simulation.rs:478–490`; tracks `initialized` flag |
| `step(dt_secs)` devuelve pares variable→valor | ✅ Implemented | `simulation.rs:492–502` maps `SimulatorPort::step` → `state.values` as `Vec<(String, f64)>` |
| `current_time()` y `value_of(variable)` | ✅ Implemented | `simulation.rs:504–510` |
| Step sin initialize eleva excepción | ✅ Implemented | Guard at `simulation.rs:493–497` → `PyValueError` |
| Unit ops steady-state (`mix/split/valve/pump/pipe`) + `StreamState` | ✅ Implemented | `simulation.rs:435–555` — all five functions + `PyStreamState` class |

**Scenarios Coverage:**

| Scenario | Status |
|----------|--------|
| Importar el submódulo y el motor | ⚠️ Compile-only |
| Inicializar y avanzar la simulación | ⚠️ Source present; E2E deferred to Phase 7.4 |
| Step sin initialize | ⚠️ Source present; no pytest yet |
| Calcular una operación de mezcla | ⚠️ Source present; no runtime test |

### `specs/ml-dataloop/spec.md` (dataframe_query submódulo)

| Requirement | Status | Notes |
|------------|--------|-------|
| Submódulo `ml_dataloop.dataframe_query` con `RrdDataframeQuery`, `QueryResult`, `TagSeries` | ✅ Implemented | `ml_dataloop.rs:195–201` |
| `RrdDataframeQuery.open(path)` | ✅ Implemented | Static method with `py.detach`; domain errors → Python exception |
| `query(window, tags)` → `QueryResult` con `series` | ✅ Implemented | `ml_dataloop.rs:179–192`; `PyQueryResult.series()` → `Vec<PyTagSeries>` |
| `TagSeries.tag` y `TagSeries.measurements` | ✅ Implemented | `ml_dataloop.rs:135–146` |
| `re_chunk_store`/`re_dataframe` workspace pinning | ✅ Implemented | Via `sp_dataframe_query` dep (`re_dataframe.workspace`, `re_chunk_store.workspace`) |

**Scenarios Coverage:**

| Scenario | Status |
|----------|--------|
| Importar el submódulo y el adapter de query | ⚠️ Compile-only |
| Consultar una ventana de tiempo | ⚠️ Source present; E2E deferred to Phase 7 |
| Archivo .rrd inexistente | ⚠️ Source present; domain maps to exception via `map_err`; no pytest |

---

## Coherence (Design)

| Decision | Followed? | Notes |
|----------|-----------|-------|
| Crate puente único `sp_python` | ✅ Yes | All Phase 5 bindings live in `sp_python` modules |
| `re_*` versiones del workspace | ✅ Yes | `re_sdk.workspace = true`; dataframe query via `sp_dataframe_query` with pinned `re_*` |
| Newtype wrapper + `#[pymethods]` | ✅ Yes | `PyRerunRecorder(RerunRecorder)`, `PyFirstOrderEngine { inner, initialized }`, `PyRrdDataframeQuery(RrdDataframeQuery)` |
| Mutación `&mut self` para engine | ✅ Yes | `initialize` and `step` take `&mut self` |
| `py.detach(...)` para I/O larga | ✅ Yes | `to_file`, `RrdDataframeQuery::open`, `query`, `run_session` (acquisition, pre-existing) |
| Puertos concretos (`RecorderPort` via `PyRerunRecorder`) | ✅ Yes | `acquisition.rs:160–168` passes `&recorder.0` to `domain_run_session` |
| RecordingStream hook en `python_bridge.rs` | ✅ Yes | `register_recording_stream_extractor` called before `sp_python::register` at `python_bridge.rs:384–397` |
| File Changes table | ✅ Yes | `recording.rs`, `simulation.rs` (engine mod), `ml_dataloop.rs` (dataframe_query mod) match design |

---

## Testing

| Area | Tests Exist? | Coverage |
|------|-------------|----------|
| `sp_python` unit tests (Phase 7.1) | No | None — no `#[cfg(test)]` in crate |
| Python smoke imports (Phase 7.2) | No | Blocked: maturin requires `rerun` CLI |
| E2E tanque_demo / sim_demo (Phase 7.3–7.4) | No | Deferred |
| Domain adapter tests (`sp_recording`, `sp_dataframe_query`, `sp_sim_engine`) | Yes (domain) | Domain crates have unit tests; bindings untested at Python boundary |

---

## Issues found

**CRITICAL** (must fix before archive):
- None for Phase 5 source implementation scope.

**WARNING** (should fix):
- Task 5.4 runtime verification blocked: `maturin develop` fails because `rerun_py/rerun_sdk/rerun_cli/rerun` is missing; Python cannot import `simplant_lab`.
- No behavioral/pytest coverage for Phase 5 bindings (deferred to Phase 7).
- `RerunRecorder(stream)` constructor depends on `register_recording_stream_extractor` being called during module init — correct in `python_bridge.rs`, but untested at runtime.

**SUGGESTION** (nice to have):
- Build `rerun` CLI artifact (or set `RERUN_ALLOW_MISSING_BIN`) to unblock maturin and complete task 5.4 import smoke.
- Add Phase 7 E2E: `RerunRecorder.to_file` → `run_session` → `flush` → `RrdDataframeQuery.open().query()` round-trip.

---

## Verdict

**PASS WITH WARNINGS**

Phase 5 adapters (recording, simulation.engine, ml_dataloop.dataframe_query) are fully implemented in source and align with their delta specs and `design.md`. Both `cargo build -p sp_python` and `cargo build -p rerun_py` pass after `ensure-pyo3-build-cfg`. Runtime smoke (maturin + Python import) remains blocked by a missing `rerun` CLI artifact; behavioral tests remain deferred to Phase 7.
