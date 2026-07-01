# Verification Report — Full Change (Phases 1–8)

**Change**: `pyo3-bindings-sp-domain`  
**Scope**: All phases (1–8) — complete pyo3 bindings for SimPlant `sp_*` domain  
**Verified at**: 2026-06-29  
**Workspace**: `/home/m4s1t4/Work/Enprendimiento/Proyectos/SimPlant/SimPlant-v2/lab`

---

## Completeness

| Metric | Value |
|--------|-------|
| Tasks total | 34 |
| Tasks complete (`[x]`) | 34 |
| Tasks incomplete | 0 |

| Phase | Tasks | Status |
|-------|-------|--------|
| 1 — Andamiaje del crate puente | 5 | ✅ All `[x]` |
| 2 — Capacidades hoja (kernel, types) | 5 | ✅ All `[x]` |
| 3 — Modelos de dominio | 5 | ✅ All `[x]` |
| 4 — Orquestación y adapters de adquisición | 5 | ✅ All `[x]` |
| 5 — Adapters Rerun y motor | 4 | ✅ All `[x]` |
| 6 — Type stubs y re-export del namespace | 2 | ✅ All `[x]` |
| 7 — Tests de comportamiento | 5 | ✅ All `[x]` |
| 8 — Verificación ADR-0002 y cierre | 3 | ✅ All `[x]` |

**Incomplete tasks:** None.

---

## Build & Test Execution

### Step 0 — pyo3 config

```bash
pixi run ensure-pyo3-build-cfg
```

**Result:** ✅ Exit 0 — generated `rerun_py/pyo3-build.cfg`

### Step 1 — Rust builds

```bash
cargo build -p sp_python
cargo build -p rerun_py
```

**Result:** ✅ Both exit 0 (`dev` profile, no errors)

### Step 2 — Rust unit tests (task 7.1)

```bash
LD_LIBRARY_PATH=".pixi/envs/default/lib" cargo test -p sp_python
```

**Result:** ✅ Exit 0 — **3 passed**, 0 failed

```
test kernel::tests::tag_id_round_trip_and_rejects_empty ... ok
test kernel::tests::measurement_round_trip ... ok
test kernel::tests::time_window_rejects_inverted_range ... ok
```

### Step 3 — maturin develop (required for Python tests)

```bash
cd rerun_py && RERUN_ALLOW_MISSING_BIN=1 pixi run maturin develop --manifest-path Cargo.toml
```

**Result:** ✅ Exit 0 — `simplant-lab-sdk-0.33.0a1+dev` installed (editable)

### Step 4 — Python behavior tests (tasks 7.2–7.5)

```bash
LD_LIBRARY_PATH=".pixi/envs/default/lib" \
PYTHONPATH=rerun_py/rerun_sdk \
  .pixi/envs/default/bin/python -m pytest -vv \
  rerun_py/tests/test_simplant_domain.py -W "ignore::DeprecationWarning"
```

**Result:** ✅ Exit 0 — **7 passed**, 0 failed

| Test | Phase | Result |
|------|-------|--------|
| `test_submodules_import_and_construct_central_types` | 7.2 | ✅ PASSED |
| `test_tanque_demo_e2e_smoke` | 7.3 | ✅ PASSED |
| `test_sim_demo_e2e_smoke` | 7.4 | ✅ PASSED |
| `test_approve_rejects_nonzero_degrees_of_freedom` | 7.5 | ✅ PASSED |
| `test_safety_factor_rejects_zero` | 7.5 | ✅ PASSED |
| `test_data_split_rejects_overlapping_windows` | 7.5 | ✅ PASSED |
| `test_parse_modbus_address_rejects_bogus` | 7.5 | ✅ PASSED |

**Env notes:**

- `cargo test -p sp_python` requires explicit `LD_LIBRARY_PATH` when `CONDA_PREFIX` is unset.
- pytest requires `PYTHONPATH=rerun_py/rerun_sdk` (editable install path) and `-W ignore::DeprecationWarning` (pytest `filterwarnings = error` vs rerun rename deprecation).
- `pixi run python -m pytest …` failed in this session with glibc virtual-package mismatch (`__glibc >=2.17`); direct invocation of `.pixi/envs/default/bin/python` succeeded.

---

## ADR-0002 — Domain Purity

**Requirement:** No `pyo3` or `re_*` dependencies in pure domain crates; binding lives only in `sp_python`.

| Crate category | Crates | `pyo3` | `re_*` | Status |
|----------------|--------|--------|--------|--------|
| Pure domain | `sp_kernel`, `sp_asset_model`, `sp_acquisition`, `sp_simulation`, `sp_stress_testing`, `sp_ml_dataloop`, `sp_acquisition_replay`, `sp_acquisition_modbus`, `sp_sim_engine` | ❌ none | ❌ none | ✅ ADR intact |
| Infrastructure adapters (pre-existing) | `sp_recording`, `sp_types`, `sp_dataframe_query` | ❌ none | ✅ expected | ✅ per ADR-0002 |
| Python bridge (new) | `sp_python` | ✅ expected | ✅ `re_sdk` (recording stream) | ✅ isolated |

**Evidence:** `grep '^(pyo3|re_)' crates/simplant/*/Cargo.toml` — only bridge + adapter crates match.

**Task 8.1:** ✅ Verified — binding confined to `sp_python`; domain crates unchanged for pyo3.

---

## UPSTREAM_DIFF Documentation

**Task 8.2:** ✅ Verified — `docs/proyect/UPSTREAM_DIFF.md` §2.2.1 documents:

| File | Change |
|------|--------|
| `rerun_py/Cargo.toml` | Dep `sp_python = { path = "../crates/simplant/sp_python" }` |
| `rerun_py/src/python_bridge.rs` | `register_recording_stream_extractor(...)` hook + `sp_python::register(py, m)?;` |

Upstream touch is minimal and documented with rationale (ADR-0002 compliance).

---

## Correctness (Specs)

### kernel

| Requirement | Status | Notes |
|------------|--------|-------|
| Submódulo `simplant_lab.kernel` accesible | ✅ Implemented | 8 submodules registered; `.pyi` present |
| `TagId` con validación ISA-5.1 | ✅ Implemented | `PyTagId` + `map_err`; tested (7.1, 7.2) |
| `Quality` como enum | ✅ Implemented | `#[pyclass]` enum + `is_usable()` |
| `Measurement` y `MeasurementBatch` | ✅ Implemented | Getters + batch helpers |
| `TimeWindow` con invariante start<end | ✅ Implemented | Tested inverted range (7.1) |
| `UnitOfMeasure`, `EngineeringRange`, `AlarmLimits` | ✅ Implemented | Enums + validation in constructors |
| `Timestamp` interoperable con Python | ✅ Implemented | epoch `f64`, nanos `i64`, ISO `__str__` |

### types

| Requirement | Status | Notes |
|------------|--------|-------|
| Submódulo `simplant_lab.types` accesible | ✅ Implemented | Tested (7.2) |
| Construcción de `ProcessVariableSample` / `TagMetadata` | ✅ Implemented | Uses kernel `Quality` |
| Helper `field(archetype, field_name)` | ✅ Implemented | `types.rs` |

### asset-model

| Requirement | Status | Notes |
|------------|--------|-------|
| Submódulo accesible | ✅ Implemented | Tested (7.2) |
| Cargar catálogo desde TOML | ✅ Implemented | `py.detach` on load; E2E (7.3) |
| Navegación del `AssetCatalog` | ✅ Implemented | Iterables for collections |
| Construcción/mutadores de `Facility` | ✅ Implemented | `add_area`/`add_unit` `&mut self` |

### simulation

| Requirement | Status | Notes |
|------------|--------|-------|
| Submódulo + `simulation.engine` | ✅ Implemented | Tested (7.2, 7.4) |
| Draft + DOF | ✅ Implemented | E2E asserts DOF==0 (7.4) |
| Aprobar flowsheet (DOF=0 gate) | ✅ Implemented | Happy path (7.4) + error (7.5) |
| Aprobar escenario | ✅ Implemented | E2E (7.4) |
| Motor `FirstOrderEngine` | ✅ Implemented | initialize/step/current_time/value_of |
| Unit ops steady-state | ✅ Implemented | mix/split/valve/pump/pipe + `StreamState` |

### acquisition

| Requirement | Status | Notes |
|------------|--------|-------|
| Submódulo + replay/modbus | ✅ Implemented | Tested (7.2, 7.3) |
| Crear sesión de adquisición | ✅ Implemented | E2E (7.3) |
| Transiciones de estado | ✅ Implemented | `start`/`stop` exposed; not pytest-covered |
| Orquestación `run_session` | ✅ Implemented | E2E tanque_demo (7.3) |
| Adapter replay CSV | ✅ Implemented | E2E (7.3) |
| Adapter Modbus + direccionamiento | ✅ Implemented | parse error tested (7.5); happy path not pytest-covered |
| Puertos desde Python (fuera de alcance) | ✅ Documented | Duck-typed concrete adapters only |

### ml-dataloop

| Requirement | Status | Notes |
|------------|--------|-------|
| Submódulo + `dataframe_query` | ✅ Implemented | Tested import (7.2) |
| `DataSplit` anti-leakage | ✅ Implemented | Overlap rejection tested (7.5) |
| `DatasetSpec` versionado | ✅ Implemented | Not pytest-covered |
| `RrdDataframeQuery` | ⚠️ Partial | `open()` in E2E (7.3); `query(window, tags)` not pytest-covered |

### stress-testing

| Requirement | Status | Notes |
|------------|--------|-------|
| Submódulo accesible | ✅ Implemented | Tested (7.2) |
| Value objects con validación | ✅ Implemented | SafetyFactor(0) tested (7.5) |
| Planificar prueba de estrés | ✅ Implemented | Not pytest-covered |
| Evaluar resultados medidos | ✅ Implemented | Not pytest-covered |

### recording

| Requirement | Status | Notes |
|------------|--------|-------|
| Submódulo accesible | ✅ Implemented | Tested (7.2) |
| Construir recorder a archivo | ✅ Implemented | E2E (7.3) |
| Flush del recorder | ✅ Implemented | E2E + file size check (7.3) |
| `tag_entity_path` | ✅ Implemented | Asserted in 7.2 |

**Scenarios Coverage Summary:**

| Scenario area | Test coverage |
|---------------|---------------|
| 8 submodules importable + central types | ✅ 7.2 |
| kernel round-trip / validation | ✅ 7.1 |
| tanque_demo E2E (acquisition + recording) | ✅ 7.3 |
| sim_demo E2E (simulation + engine) | ✅ 7.4 |
| Error paths (DOF, SafetyFactor, DataSplit, modbus) | ✅ 7.5 |
| Session lifecycle, modbus happy path, dataset spec, stress plan/evaluate, unit ops, `.rrd` query | ⚠️ Implemented but not pytest-covered |

---

## Coherence (Design)

| Decision | Followed? | Notes |
|----------|-----------|-------|
| Crate puente único `sp_python` | ✅ Yes | All bindings in `crates/simplant/sp_python/` |
| `crate-type = ["rlib"]` | ✅ Yes | Only `rerun_py` is cdylib |
| Newtype + `#[pymethods]` / native enums | ✅ Yes | Pattern consistent across modules |
| Mutación `&mut self` / consume patterns | ✅ Yes | Sessions, flowsheets, stress tests, engine |
| Puertos con adapters concretos | ✅ Yes | `OwnedDataSource` internal enum + duck typing (not `PyDataSource` pyclass — minor naming deviation, functionally equivalent) |
| Timestamp as epoch `f64` + nanos | ✅ Yes | kernel bindings |
| Reuso versiones `re_*` workspace | ✅ Yes | `sp_python` pins via workspace |
| Una línea en `python_bridge.rs` | ⚠️ Deviated | Two hooks: `register_recording_stream_extractor` + `register` — documented in UPSTREAM_DIFF §2.2.1 with rationale |
| File Changes table | ✅ Yes | All listed files exist |
| `.pyi` stubs per capability | ✅ Yes | 12 stub files under `rerun_py/rerun_sdk/simplant_lab/` |
| `simplant_lab` re-exports | ✅ Yes | `__init__.py` imports all 8 domain submodules |
| `py.detach` for long I/O | ✅ Yes | catalog load, run_session, recording, dataframe query |

---

## Testing

| Layer | Tests Exist? | Runtime result |
|-------|-------------|----------------|
| Rust unit (sp_python kernel) | Yes (3) | ✅ 3/3 pass |
| Python import smoke (8 submodules) | Yes (1) | ✅ Pass |
| E2E tanque_demo | Yes (1) | ✅ Pass |
| E2E sim_demo | Yes (1) | ✅ Pass |
| Error paths | Yes (4) | ✅ 4/4 pass |
| **Total Python** | **7 tests** | **✅ 7/7 pass** |

---

## Proposal Success Criteria

| Criterion | Status |
|-----------|--------|
| `sp_python` compila; `cargo build -p rerun_py` with domain submodules | ✅ Verified |
| 8 submodules accessible from Python | ✅ Verified (7.2) |
| Smoke test tanque_demo flow | ✅ Verified (7.3) |
| Smoke test sim_demo flow | ✅ Verified (7.4) |
| Domain crates without `re_*`/`pyo3` (ADR-0002) | ✅ Verified (8.1) |
| `.pyi` stubs per submodule | ✅ Verified (6.1) |

**Note:** `proposal.md` success-criteria checkboxes remain `[ ]` — documentation drift only.

---

## Issues Found

**CRITICAL** (must fix before archive):
- None.

**WARNING** (should fix):
1. **pytest/cargo env friction** — tests require explicit `PYTHONPATH`, `LD_LIBRARY_PATH`, and DeprecationWarning filter; bare `pixi run pytest` fails (import path + strict warnings).
2. **Partial E2E `.rrd` query coverage** — 7.3 calls `RrdDataframeQuery.open()` but not `query(window, tags)` (ml-dataloop spec scenario partially tested).
3. **Many spec scenarios implemented but untested** — session lifecycle, modbus happy path, dataset spec errors, stress plan/evaluate, unit ops, step-without-initialize, etc.
4. **`proposal.md` checkboxes unchecked** — success criteria still `[ ]` despite fulfillment.
5. **Upstream wiring exceeds "one line"** — `register_recording_stream_extractor` added alongside `register` (documented in UPSTREAM_DIFF; functionally required for `RerunRecorder(stream)`).

**SUGGESTION** (nice to have):
- Extend 7.3 to assert `query(TimeWindow, [tag])` returns non-empty series.
- Fix editable-install path so `import simplant_lab` works without `PYTHONPATH`.
- Add pytest for session `start`/`stop` lifecycle and modbus happy-path parse.
- Mark `proposal.md` success criteria as complete.
- Consider pure-Python wrapper layer (explicitly out of scope per proposal).

---

## Verdict

**PASS WITH WARNINGS**

All 34 tasks across phases 1–8 are complete. Builds (`sp_python`, `rerun_py`), Rust unit tests (3/3), and Python behavior tests (7/7) pass with documented environment variables. ADR-0002 is intact (pyo3 binding isolated in `sp_python`; pure domain crates unchanged). UPSTREAM_DIFF §2.2.1 documents the minimal upstream wiring. Warnings are limited to test-env friction, partial pytest coverage of some spec scenarios, and unchecked proposal checkboxes — none block archive.

---

## Structured Envelope

```yaml
status: pass_with_warnings
executive_summary: >
  Full pyo3-bindings-sp-domain change verified: 34/34 tasks complete, all builds
  and tests green (with PYTHONPATH/LD_LIBRARY_PATH), ADR-0002 intact, UPSTREAM_DIFF
  documented. Warnings on pytest env friction and partial scenario test coverage.
artifacts:
  - openspec/changes/pyo3-bindings-sp-domain/verify-report.md
next_recommended: sdd-archive
risks:
  - pytest env vars required in CI/docs
  - ml-dataloop query path lacks behavioral test
```
