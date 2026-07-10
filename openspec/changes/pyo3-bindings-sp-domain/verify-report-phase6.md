# Verification report — phase 6

**Change**: `pyo3-bindings-sp-domain`
**Scope**: Phase 6 only (tasks 6.1–6.2 — type stubs & namespace re-exports)
**Verified at**: 2026-06-29
**Workspace**: `/home/m4s1t4/Work/Enprendimiento/Proyectos/SimPlant/SimPlant-v2/lab`

---

## Completeness

| Metric | Value |
|--------|-------|
| Phase 6 tasks total | 2 |
| Phase 6 tasks marked complete in `tasks.md` | 2 |
| Phase 6 tasks implemented in source | 2 |
| Phase 6 tasks incomplete (implementation) | 0 |
| Phase 6 tasks incomplete (runtime smoke) | 0 (with `PYTHONPATH=rerun_py/rerun_sdk`) |

| Task | Marked | Evidence |
|------|--------|----------|
| 6.1 Crear `rerun_py/rerun_sdk/simplant_lab/*.pyi` con firmas públicas | ✅ `[x]` | 12 stub files under `rerun_py/rerun_sdk/simplant_lab/` (8 top-level + 4 nested) |
| 6.2 Re-export de submódulos del namespace en `simplant_lab` | ✅ `[x]` | `__init__.py:27–34` imports all 8 domain submodules; runtime import smoke passes |

---

## Build verification

### Step 0 — pyo3 config

```bash
pixi run ensure-pyo3-build-cfg
```

**Result:** ✅ Exit 0 — generated `rerun_py/pyo3-build.cfg`

### Step 1 — `cargo build -p rerun_py`

```bash
cargo build -p rerun_py
```

**Result:** ✅ Exit 0 — `Finished dev profile [optimized] target(s) in 23.00s`

### Step 2 — `maturin develop` (with missing-bin override)

```bash
cd rerun_py && RERUN_ALLOW_MISSING_BIN=1 pixi run maturin develop
```

**Result:** ✅ Exit 0 — wheel built and editable install succeeded (`simplant-lab-sdk-0.33.0a1+dev`)

### Step 3 — Python import smoke (task 6.2)

```bash
PYTHONPATH=rerun_py/rerun_sdk pixi run python -c "
import simplant_lab as sl
# 8 domain submodules
for n in ['kernel','asset_model','acquisition','simulation','ml_dataloop','stress_testing','recording','types']:
    getattr(sl, n)
# recording collision
assert sl.recording.RerunRecorder is not None
assert sl.rrd_recording.Recording is not None
assert sl.recording is not sl.rrd_recording
# nested submodules
sl.acquisition.replay.CsvReplaySource
sl.acquisition.modbus.parse_modbus_address
sl.simulation.engine.FirstOrderEngine
sl.ml_dataloop.dataframe_query.RrdDataframeQuery
# construct smoke
sl.kernel.TagId('FT-101')
sl.recording.tag_entity_path(sl.kernel.TagId('FT-101'))
"
```

**Result:** ✅ Exit 0 — all imports and smoke constructions succeed (`tag_entity_path: tags/FT-101`)

**Note:** Bare `pixi run python -c "import simplant_lab"` fails with `ModuleNotFoundError` because the editable `.pth` adds `rerun_py/` but the package lives under `rerun_py/rerun_sdk/`. The project’s own lint config uses `MYPYPATH=rerun_py:rerun_py/rerun_sdk` (`pixi.toml`). This is a pre-existing packaging path issue, not introduced by Phase 6.

---

## Correctness (Specs — public API surface)

Phase 6 does not add new behavioral specs; verification cross-checks stub coverage against the “submódulo accesible” requirements in all 8 delta specs and the runtime surface registered by `sp_python::register`.

### Stub inventory vs `sp_python` bindings

| Submodule | Stub file(s) | Rust module | Classes/enums in stub | Runtime symbols verified |
|-----------|-------------|-------------|----------------------|--------------------------|
| `kernel` | `kernel.pyi` | `kernel.rs` | TagId, Quality, Dimension, UnitOfMeasure, Measurement, MeasurementBatch, TimeWindow, EngineeringRange, AlarmLimits | ✅ |
| `types` | `types.pyi` | `types.rs` | ProcessVariableSample, TagMetadata, constants, `field()` | ✅ (see WARNING: `Quality` re-export) |
| `asset_model` | `asset_model.pyi` | `asset_model.rs` | IDs, Facility, Area, ProcessUnit, Equipment, Tag, AssetCatalog, TomlCatalogRepository, EquipmentKind | ✅ |
| `acquisition` | `acquisition.pyi`, `acquisition/replay.pyi`, `acquisition/modbus.pyi` | `acquisition.rs` | TagBinding, SamplingPolicy, SessionState, AcquisitionSession, `run_session`; CsvReplaySource; ModbusTcpSource, ModbusPoint, RegisterKind, parse/map helpers | ✅ |
| `simulation` | `simulation.pyi`, `simulation/engine.pyi` | `simulation.rs` | FlowsheetSpec, Scenario, value objects, enums, IDs; FirstOrderEngine, StreamState, unit-op functions | ✅ (see WARNING: engine.pyi import) |
| `stress_testing` | `stress_testing.pyi` | `stress_testing.rs` | LoadPoint, LoadProfile, DesignLimit, SafetyFactor, AcceptanceCriterion, MeasuredOutcome, StressTestState, StressTest | ✅ |
| `recording` | `recording.pyi` | `recording.rs` | RerunRecorder, PLANT_TIME, EVENTS_PATH, `tag_entity_path()` | ✅ |
| `ml_dataloop` | `ml_dataloop.pyi`, `ml_dataloop/dataframe_query.pyi` | `ml_dataloop.rs` | FeatureSpec, DataSplit, DatasetSpec; RrdDataframeQuery, QueryResult, TagSeries | ✅ |

**Cross-check:** All `#[pyclass(name = "…")]` types and `#[pyfunction]` exports in `crates/simplant/sp_python/src/` have corresponding entries in the stub set. No missing public class or function detected at runtime.

### Task 6.1 — stub coverage detail

| Requirement | Status | Notes |
|------------|--------|-------|
| One stub per domain submodule | ✅ Implemented | 8 submodules covered; nested stubs for `acquisition.{replay,modbus}`, `simulation.engine`, `ml_dataloop.dataframe_query` |
| Public class signatures | ✅ Implemented | Constructors, getters, staticmethods, enums mirror Rust `#[pymethods]` |
| Public function signatures | ✅ Implemented | `field`, `tag_entity_path`, `run_session`, `parse_modbus_address`, `map_register`, unit-op fns |
| Constants exported | ✅ Implemented | `types.*` archetype/component constants; `recording.PLANT_TIME` / `EVENTS_PATH` |

**Scenarios Coverage (import/access from specs):**

| Scenario (all specs: “submódulo accesible”) | Status |
|----------|--------|
| `simplant_lab.kernel` | ✅ Covered (runtime + stub) |
| `simplant_lab.types` + constants | ✅ Covered |
| `simplant_lab.asset_model` | ✅ Covered |
| `simplant_lab.acquisition` + `replay` / `modbus` | ✅ Covered |
| `simplant_lab.simulation` + `engine` | ✅ Covered |
| `simplant_lab.stress_testing` | ✅ Covered |
| `simplant_lab.recording` (domain) | ✅ Covered |
| `simplant_lab.ml_dataloop` + `dataframe_query` | ✅ Covered |

### Task 6.2 — re-exports & recording collision

| Requirement | Status | Notes |
|------------|--------|-------|
| `__init__.py` re-exports 8 domain submodules | ✅ Implemented | Lines 27–34: explicit `from . import … as …` for each domain submodule |
| Access `simplant_lab.kernel`, `.asset_model`, …, `.types` | ✅ Verified | Runtime `getattr(simplant_lab, name)` for all 8 |
| Recording namespace collision resolved | ✅ Verified | See below |

**Recording collision resolution:**

| Before (conflict) | After (resolved) | Evidence |
|-------------------|------------------|----------|
| Rerun SDK legacy `simplant_lab/recording/` Python package | Renamed to `simplant_lab/rrd_recording/` | Directory `rrd_recording/__init__.py` exports `Recording`, `RRDArchive`, `load_recording`, `load_archive` |
| SimPlant domain `recording` (pyo3) needed same name | Domain submodule registered as `simplant_lab.recording` via pyo3 | `sp_python/src/recording.rs:65–72`, `attach_simplant_submodule(…, "recording", …)` |
| Internal imports of Rerun `Recording` | Updated to `simplant_lab.rrd_recording` | `catalog/_entry.py:34`, `sinks.py:21` |
| Package root exports both | No shadowing | `__init__.py:31` (`recording`) and `:40` (`rrd_recording`); runtime assert `recording is not rrd_recording` passes |
| No stale `recording/` Python package dir | ✅ | Glob confirms zero files under `simplant_lab/recording/`; only `recording.pyi` stub + `recording_stream.py` (Rerun stream API, unrelated) |

---

## Coherence (Design)

| Decision | Followed? | Notes |
|----------|-----------|-------|
| Stubs per capability (`design.md` File Changes) | ✅ Yes | `rerun_py/rerun_sdk/simplant_lab/*.pyi` created as specified |
| Re-export domain submodules in `__init__.py` | ✅ Yes | Matches design table row for `simplant_lab/__init__.py` |
| Domain `recording` separate from Rerun recording utilities | ✅ Yes | `rrd_recording` rename frees `recording` for pyo3 domain submodule |
| Nested stub layout mirrors pyo3 submodules | ✅ Yes | `simulation/engine.pyi`, `acquisition/replay.pyi`, etc. |

---

## Testing

| Area | Tests Exist? | Coverage |
|------|-------------|----------|
| Dedicated stub/type-check CI for domain pyi | No | `py-check-signatures` depends on `py-build` which currently fails on uv workspace resolution |
| Runtime import smoke (manual, this verify) | Yes (manual) | All 8 submodules + 4 nested submodules + recording collision |
| Phase 7 pytest (`test_simplant_domain.py`) | No | Deferred — not in Phase 6 scope |

---

## Issues found

**CRITICAL** (must fix before archive):
- None for Phase 6 scope.

**WARNING** (should fix):
- `types.pyi` omits the runtime re-export `Quality` (registered in `types.rs:80` via `types.add("Quality", …)`). Stub users cannot type-check `simplant_lab.types.Quality`.
- `simulation/engine.pyi` line 3: `from .simulation import Scenario` is an invalid relative import for module `simplant_lab.simulation.engine`; should be `from ..simulation import Scenario` (or absolute). Type checkers will fail to resolve `Scenario`.
- `simulation.pyi` `MaterialStream.__init__` documents `from` only in a comment; the kw-only `from` parameter exposed by Rust (`#[pyo3(signature = (id, composition, from=None, to=None))]`) is absent from the stub signature.
- `acquisition.pyi` types `run_session(..., source: Any)` instead of `CsvReplaySource | ModbusTcpSource`, reducing type-checker value (matches Phase 4 note on concrete adapters).
- Dual stub layout (`acquisition.pyi` at package root **and** `acquisition/*.pyi` nested) may confuse strict type checkers; no `acquisition/__init__.pyi` bridges the two.
- Bare `import simplant_lab` without `PYTHONPATH=rerun_py/rerun_sdk` fails in pixi env (pre-existing packaging path; maturin editable `.pth` insufficient alone).

**SUGGESTION** (nice to have):
- Add `Quality` to `types.pyi` and fix `engine.pyi` import as quick follow-ups before Phase 7.
- Consider `run_session` source union type in `acquisition.pyi`.
- Restructure nested stubs to use `acquisition/__init__.pyi` instead of flat `acquisition.pyi` for cleaner PEP 561 layout.

---

## Verdict

**PASS WITH WARNINGS**

Phase 6 deliverables are in place: 12 `.pyi` stub files cover the full public pyo3 surface across all 8 domain submodules (including nested adapters), `__init__.py` re-exports every domain submodule, and the Rerun-vs-domain `recording` namespace collision is resolved via the `rrd_recording` rename with internal references updated. `cargo build -p rerun_py` and `RERUN_ALLOW_MISSING_BIN=1 maturin develop` succeed; runtime import smoke passes for all submodules including `simplant_lab.kernel`, nested adapters, and distinct `recording` / `rrd_recording` modules. Minor stub inaccuracies (`types.Quality`, `engine.pyi` import path, `MaterialStream.from`) should be corrected but do not block Phase 7 behavioral tests.
