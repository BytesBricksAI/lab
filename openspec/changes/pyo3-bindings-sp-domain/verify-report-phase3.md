# Verification Report — Phase 3

**Change**: `pyo3-bindings-sp-domain`  
**Scope**: Phase 3 only (tasks 3.1–3.5 — asset_model, simulation, stress_testing, ml_dataloop)  
**Verified at**: 2026-06-28  
**Workspace**: `/home/m4s1t4/Work/Enprendimiento/Proyectos/SimPlant/SimPlant-v2/lab`

---

## Completeness

| Metric | Value |
|--------|-------|
| Phase 3 tasks total | 5 |
| Phase 3 tasks marked complete in `tasks.md` | 0 |
| Phase 3 tasks implemented in source | 5 |
| Phase 3 tasks incomplete (implementation) | 0 |

All Phase 3 tasks (3.1–3.5) remain marked `[ ]` in `tasks.md`, but source inspection shows full implementation for each task.

| Task | Marked | Evidence |
|------|--------|----------|
| 3.1 `asset_model.rs` IDs, catalog, facility, TOML repo | ❌ `[ ]` | `crates/simplant/sp_python/src/asset_model.rs` (287 lines) |
| 3.2 `simulation.rs` value objects, enums, FlowsheetSpec, Scenario | ❌ `[ ]` | `crates/simplant/sp_python/src/simulation.rs` (445 lines) |
| 3.3 `stress_testing.rs` value objects, StressTest plan/evaluate | ❌ `[ ]` | `crates/simplant/sp_python/src/stress_testing.rs` (220 lines) |
| 3.4 `ml_dataloop.rs` FeatureSpec, DataSplit, DatasetSpec (no dataframe_query) | ❌ `[ ]` | `crates/simplant/sp_python/src/ml_dataloop.rs` (128 lines) |
| 3.5 Wire in `lib.rs` + build + smoke import | ❌ `[ ]` | `lib.rs:31-35`; builds pass; smoke import blocked (see Build Verification) |

---

## Build Verification

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
    Finished `dev` profile [optimized] target(s) in 0.30s
```

### Step 2 — `cargo build -p rerun_py`

```bash
cargo build -p rerun_py
```

**Result:** ✅ Exit 0

```
    Finished `dev` profile [optimized] target(s) in 0.39s
```

### Step 3 — Task 3.5 runtime smoke (maturin + Python import)

```bash
cd rerun_py && pixi run maturin develop
```

**Result:** ❌ Exit 1 — build script error:

```
ERROR: Expected to find `rerun` at "/home/m4s1t4/Work/Enprendimiento/Proyectos/SimPlant/SimPlant-v2/lab/rerun_py/rerun_sdk/rerun_cli/rerun".
```

Python smoke import of `simplant_lab.asset_model`, `simplant_lab.simulation`, `simplant_lab.stress_testing`, and `simplant_lab.ml_dataloop` could not be exercised (extension not installed). Same blocker as Phase 2 verification.

---

## Correctness (Specs)

> **Scope note:** Specs for `simulation.engine` / `FirstOrderEngine`, steady-state unit ops (`mix`, `split`, …), and `ml_dataloop.dataframe_query` / `RrdDataframeQuery` are explicitly deferred to Phase 5 (tasks 5.2–5.3). They are **not** evaluated as Phase 3 gaps.

### specs/asset-model — Requirements

| Requirement | Status | Notes |
|-------------|--------|-------|
| Submódulo `simplant_lab.asset_model` accesible | ✅ Implemented | `asset_model::register` creates submodule; `attach_simplant_submodule` sets `simplant_lab.asset_model` |
| IDs tipados (`FacilityId`, `AreaId`, `UnitId`, `EquipmentId`) | ✅ Implemented | Macro `define_py_id!`; validation via `map_err` |
| `TomlCatalogRepository(path)` + `load_catalog()` | ✅ Implemented | `PyTomlCatalogRepository::new`; `load_catalog` uses `py.detach` for I/O; errors via `map_err` |
| `AssetCatalog` navegación (`facility/equipment/tags/tag/equipment_by_id/validate`) | ✅ Implemented | All getters return `Vec<Py*>` or `Option<Py*>` (Python-iterable sequences) |
| `Facility` construcción + `add_area`/`add_unit` `&mut self` | ✅ Implemented | `Facility.define(id, name)` staticmethod; mutators map domain errors |

**Scenarios Coverage (asset-model):**

| Scenario | Status | Notes |
|----------|--------|-------|
| Importar el submódulo | ⚠️ Partial | Registration code correct; runtime import not exercised (maturin blocked) |
| Cargar catálogo válido | ⚠️ Partial | `load_catalog()` → `PyAssetCatalog`; `tags()` returns `Vec<PyTag>` |
| Archivo inexistente | ⚠️ Partial | Domain error mapped via `map_err`; no runtime test |
| Iterar tags del catálogo | ⚠️ Partial | `tags()` + `tag(id)` implemented |
| Validar integridad del catálogo | ⚠️ Partial | `validate()` delegates to domain |
| Agregar área y unidad | ⚠️ Partial | `add_area`/`add_unit` + `has_area`/`has_unit` implemented |
| Unidad en área inexistente | ⚠️ Partial | Domain error mapped via `map_err` |

**Extra (beyond task 3.1 minimum):** `Area`, `ProcessUnit`, `Equipment`, `Tag`, `EquipmentKind` also exposed — aligns with spec hierarchy mention and design file table.

### specs/simulation — Requirements (Phase 3 scope)

| Requirement | Status | Notes |
|-------------|--------|-------|
| Submódulo `simplant_lab.simulation` accesible | ✅ Implemented | `simulation::register` + 14 classes/enums registered |
| Value objects (`ChemicalComponent`, `Composition`, `UnitOp`, `MaterialStream`, `Specification`, `BoundaryCondition`) | ✅ Implemented | All with `#[new]` + getters |
| Enums (`UnitOpKind`, `ThermoPackage`, `EngineCapability`) | ✅ Implemented | Native `#[pyclass] enum` per design |
| IDs (`FlowsheetId`, `UnitOpId`, `StreamId`, `ScenarioId`) | ✅ Implemented | Macro `define_py_id!` |
| `FlowsheetSpec.draft` staticmethod | ✅ Implemented | 6-arg draft; domain errors via `map_err` |
| `degrees_of_freedom()` | ✅ Implemented | Returns `i64` |
| `approve()` `&mut self` | ✅ Implemented | Domain gate DOF=0 via `map_err` |
| `Scenario.approve` staticmethod | ✅ Implemented | Validates against approved flowsheet |
| `FirstOrderEngine` / `simulation.engine` | ⏭️ Deferred | Phase 5 task 5.2 — not in Phase 3 scope |
| Unit ops steady-state (`mix`, `split`, …) | ⏭️ Deferred | Phase 5 task 5.2 — not in Phase 3 scope |

**Scenarios Coverage (simulation, Phase 3 scope):**

| Scenario | Status | Notes |
|----------|--------|-------|
| Importar submódulo y enums | ⚠️ Partial | Registration correct; runtime not exercised |
| Draft válido y cálculo de DOF | ⚠️ Partial | `draft(...)` + `degrees_of_freedom()` + `state()`/`FlowsheetState` |
| Draft inválido | ⚠️ Partial | `map_err` on domain failure |
| Aprobar flowsheet cuadrado | ⚠️ Partial | `approve()` + `is_approved()`/`state()` |
| Rechazar aprobación con DOF≠0 | ⚠️ Partial | Domain error mapped |
| Aprobar escenario válido | ⚠️ Partial | `Scenario.approve(...)` + `duration_secs()`/`is_approved()` |

### specs/stress-testing — Requirements

| Requirement | Status | Notes |
|-------------|--------|-------|
| Submódulo `simplant_lab.stress_testing` accesible | ✅ Implemented | `stress_testing::register` registers 8 types |
| Value objects con validación | ✅ Implemented | `LoadPoint`, `LoadProfile`, `DesignLimit`, `AcceptanceCriterion`, `MeasuredOutcome` |
| `SafetyFactor(value)` rechaza ≤0 / no finitos | ✅ Implemented | `SafetyFactor::new(value).map_err(map_err)` |
| `StressTest.plan(...)` staticmethod | ✅ Implemented | Returns `Planned` state |
| `evaluate(outcomes)` `&mut self` | ✅ Implemented | Returns `bool` (passed); transitions to `Completed` |
| Enum `StressTestState` | ✅ Implemented | `Planned`, `Completed` |

**Scenarios Coverage (stress-testing):**

| Scenario | Status | Notes |
|----------|--------|-------|
| Importar el submódulo | ⚠️ Partial | Registration correct; runtime not exercised |
| Crear SafetyFactor válido | ⚠️ Partial | `value()` getter implemented |
| Rechazar SafetyFactor inválido | ⚠️ Partial | Domain rejects 0.0 via `map_err` |
| Planificar prueba válida | ⚠️ Partial | `StressTest.plan(...)` + `state()` |
| Carga excede límite de diseño | ⚠️ Partial | Domain error mapped |
| Evaluar resultados dentro de criterio | ⚠️ Partial | `evaluate()` returns `bool` |
| Evaluar dos veces | ⚠️ Partial | Domain error mapped on re-evaluate |

### specs/ml-dataloop — Requirements (Phase 3 scope)

| Requirement | Status | Notes |
|-------------|--------|-------|
| Submódulo `simplant_lab.ml_dataloop` accesible | ✅ Implemented | `ml_dataloop::register` registers 3 classes |
| `FeatureSpec(tag, name)` | ✅ Implemented | Validates via domain `map_err` |
| `DataSplit` anti-leakage | ✅ Implemented | Overlap rejection via domain; accessors `train/val/test/windows` |
| `DatasetSpec.define(...)` + getters | ✅ Implemented | Static `define`; `id/version/features/targets/split` |
| `RrdDataframeQuery` / `dataframe_query` | ⏭️ Deferred | Phase 5 task 5.3 — explicitly excluded from task 3.4 |

**Scenarios Coverage (ml-dataloop, Phase 3 scope):**

| Scenario | Status | Notes |
|----------|--------|-------|
| Importar submódulo (DatasetSpec, DataSplit) | ⚠️ Partial | Registration correct; runtime not exercised |
| Split válido sin solapamiento | ⚠️ Partial | `windows()` returns named partitions |
| Rechazar split con fuga temporal | ⚠️ Partial | Domain overlap check via `map_err` |
| Definir dataset válido | ⚠️ Partial | `DatasetSpec.define(...)` + `version()` |
| Feature con tag ausente | ⚠️ Partial | Catalog validation via domain `map_err` |

---

## Task-by-Task Verification

### Task 3.1 — `asset_model.rs`

| Item | Expected | Found |
|------|----------|-------|
| `PyFacilityId` / `PyAreaId` / `PyUnitId` / `PyEquipmentId` | ✅ | Lines 53–56 via macro |
| `PyAssetCatalog` getters | ✅ | `facility/equipment/tags/tag/equipment_by_id/validate` lines 223–252 |
| `PyFacility` + `add_area`/`add_unit` `&mut self` | ✅ | Lines 121–162 |
| `PyTomlCatalogRepository` + `load_catalog` | ✅ | Lines 255–268; `py.detach` for I/O |
| `asset_model::register` | ✅ | Lines 270–286 |
| Reuse `PyTagId` from kernel | ✅ | `PyTag.id()` returns `PyTagId` |

### Task 3.2 — `simulation.rs`

| Item | Expected | Found |
|------|----------|-------|
| Value objects | ✅ | `PyChemicalComponent`, `PyComposition`, `PyUnitOp`, `PyMaterialStream`, `PySpecification`, `PyBoundaryCondition` |
| Enums | ✅ | `UnitOpKind`, `ThermoPackage`, `EngineCapability`, `FlowsheetState` |
| IDs | ✅ | `PyFlowsheetId`, `PyUnitOpId`, `PyStreamId`, `PyScenarioId` |
| `PyFlowsheetSpec.draft` staticmethod | ✅ | Lines 315–335 |
| `degrees_of_freedom()` | ✅ | Line 353 |
| `approve()` `&mut self` | ✅ | Lines 361–363 |
| `PyScenario.approve` staticmethod | ✅ | Lines 371–388 |
| `simulation::register` | ✅ | Lines 424–444 |

### Task 3.3 — `stress_testing.rs`

| Item | Expected | Found |
|------|----------|-------|
| `PyLoadPoint`, `PyLoadProfile`, `PyDesignLimit` | ✅ | Lines 9–67 |
| `PySafetyFactor` (rejects ≤0) | ✅ | Lines 69–85; `map_err` |
| `PyAcceptanceCriterion`, `PyMeasuredOutcome` | ✅ | Lines 87–125 |
| `StressTestState` enum | ✅ | Lines 127–139 |
| `PyStressTest.plan` staticmethod | ✅ | Lines 146–163 |
| `PyStressTest.evaluate` `&mut self` | ✅ | Lines 199–204 |
| `stress_testing::register` | ✅ | Lines 207–219 |

### Task 3.4 — `ml_dataloop.rs`

| Item | Expected | Found |
|------|----------|-------|
| `PyFeatureSpec` | ✅ | Lines 8–28 |
| `PyDataSplit` (rejects overlap) | ✅ | Lines 30–63; domain validation |
| `PyDatasetSpec.define` + getters | ✅ | Lines 65–118 |
| No `dataframe_query` submodule | ✅ | Confirmed absent (Phase 5) |
| `ml_dataloop::register` | ✅ | Lines 120–127 |

### Task 3.5 — Wiring + build + smoke

| Item | Expected | Found |
|------|----------|-------|
| `asset_model::register` in `lib.rs` | ✅ | `lib.rs:31` |
| `simulation::register` in `lib.rs` | ✅ | `lib.rs:33` |
| `ml_dataloop::register` in `lib.rs` | ✅ | `lib.rs:34` |
| `stress_testing::register` in `lib.rs` | ✅ | `lib.rs:35` |
| `sp_python::register` in `python_bridge.rs` | ✅ | `rerun_py/src/python_bridge.rs:384` |
| `cargo build -p sp_python` | ✅ | Exit 0 |
| `cargo build -p rerun_py` | ✅ | Exit 0 |
| Smoke import each submodule | ❌ | Blocked by missing `rerun` CLI binary |

---

## Coherence (Design)

| Decision | Followed? | Notes |
|----------|-----------|-------|
| Single bridge crate `sp_python` | ✅ Yes | All Phase 3 modules in `sp_python` |
| Newtype wrapper for structs with invariants | ✅ Yes | IDs, FlowsheetSpec, Scenario, StressTest, DatasetSpec |
| Native `#[pyclass] enum` for C-like enums | ✅ Yes | UnitOpKind, ThermoPackage, EngineCapability, FlowsheetState, StressTestState, EquipmentKind |
| `&mut self` mutators | ✅ Yes | `Facility.add_area/add_unit`, `FlowsheetSpec.approve`, `StressTest.evaluate` |
| Staticmethods for factory/plan methods | ✅ Yes | `FlowsheetSpec.draft`, `Scenario.approve`, `StressTest.plan`, `DatasetSpec.define`, `Facility.define` |
| `map_err` → `PyValueError` | ✅ Yes | All domain constructors/mutators that can fail |
| `py.detach` for long I/O | ✅ Yes | `TomlCatalogRepository.load_catalog` |
| `register(py, parent)` per capability | ✅ Yes | Each module has `register`; wired in `lib.rs::register` |
| No `engine` / `dataframe_query` in Phase 3 | ✅ Yes | Correctly deferred per tasks.md Phase 5 |
| File changes match design table | ✅ Yes | `asset_model.rs`, `simulation.rs`, `stress_testing.rs`, `ml_dataloop.rs`, `lib.rs` |

---

## Testing

Phase 3 has no dedicated test tasks (tests are Phase 7). No `#[cfg(test)]` blocks exist in `sp_python`.

| Area | Tests Exist? | Coverage |
|------|-------------|----------|
| `sp_python` unit tests | No | None — deferred to Phase 7 |
| `cargo build -p sp_python` | Yes | ✅ Pass |
| `cargo build -p rerun_py` | Yes | ✅ Pass |
| `maturin develop` + Python import smoke | Attempted | ❌ Blocked by missing rerun CLI |
| `test_simplant_domain.py` | No | Not created yet (Phase 7) |

---

## Issues Found

### CRITICAL (must fix before archive)

None for Phase 3 implementation correctness. Compile-time verification passes for both crates.

### WARNING (should fix)

1. **`tasks.md` Phase 3 checkboxes not updated.** All tasks 3.1–3.5 are implemented but remain marked `[ ]`. Process/completeness gap only — code is present.

2. **Task 3.5 runtime smoke not satisfied.** `maturin develop` fails because `rerun_py` build script expects `rerun_sdk/rerun_cli/rerun` binary. Python import of the four Phase 3 submodules could not be exercised.

3. **No behavioral tests for Phase 3 scenarios.** All spec scenarios are implemented in source but lack runtime verification (Phase 7 not started).

4. **`DataSplit` Python signature deviates from spec.** Spec documents `DataSplit(train, val, test)`; implementation exposes `DataSplit(train, test, val=None)` (parameter order differs; `val` optional). Domain API uses `new(train, val: Option, test)` — functionally correct but Python API surface differs from spec wording.

5. **`Facility` uses `define()` staticmethod instead of `#[new]`.** Spec scenarios assume a constructible facility; `Facility.define(id, name)` works but is not a conventional Python constructor.

### SUGGESTION (nice to have)

1. Mark tasks 3.1–3.5 as `[x]` in `tasks.md` after this verification pass.

2. Align `DataSplit` Python signature to `(train, val, test)` to match spec, or update spec to document `(train, test, val=None)`.

3. Build/install `rerun` CLI to unblock maturin and complete task 3.5 smoke imports.

4. Add Phase 7 tests early for high-value invariants: `SafetyFactor(0.0)`, overlapping `DataSplit`, `approve()` with DOF≠0.

---

## Verdict

**PASS WITH WARNINGS**

Phase 3 domain model bindings (asset_model, simulation, stress_testing, ml_dataloop) are fully implemented in source and align with their delta specs (within Phase 3 scope), tasks.md, and design.md. Both `cargo build -p sp_python` and `cargo build -p rerun_py` pass after `ensure-pyo3-build-cfg`. Runtime Python smoke (maturin + import) remains blocked by a missing `rerun` CLI artifact; `tasks.md` checkboxes are not yet updated; behavioral tests remain deferred to Phase 7.

---

## Structured Envelope

```yaml
status: completed
executive_summary: >
  Phase 3 tasks 3.1–3.4 are fully implemented (asset_model, simulation,
  stress_testing, ml_dataloop bindings with register wiring). cargo build -p
  sp_python and cargo build -p rerun_py both pass. tasks.md still marks Phase 3
  as incomplete. maturin develop and Python import smoke fail due to missing
  rerun CLI binary; no runtime behavioral tests yet.
verdict: PASS WITH WARNINGS
critical_issues: []
warning_issues:
  - "tasks.md Phase 3 checkboxes not updated despite implementation complete"
  - "maturin develop fails — missing rerun_sdk/rerun_cli/rerun binary (task 3.5 runtime smoke)"
  - "Python import of Phase 3 submodules not exercised"
  - "No #[cfg(test)] coverage for Phase 3 invariants (Phase 7 deferred)"
  - "DataSplit Python signature (train, test, val=None) deviates from spec (train, val, test)"
artifacts:
  - openspec/changes/pyo3-bindings-sp-domain/verify-report-phase3.md
next_recommended:
  - "Mark tasks 3.1–3.5 as [x] in tasks.md"
  - "Build/install rerun CLI to unblock maturin develop"
  - "Run task 3.5 Python smoke imports after maturin succeeds"
  - "Proceed to Phase 4 (acquisition + replay + modbus)"
risks:
  - "Runtime submodule registration untested until maturin install succeeds"
  - "DataSplit parameter order mismatch may confuse Python consumers vs spec/docs"
```
