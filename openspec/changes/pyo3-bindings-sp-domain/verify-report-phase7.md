# Verification Report — Phase 7

**Change**: `pyo3-bindings-sp-domain`  
**Scope**: Phase 7 only (tasks 7.1–7.5 — behavioral tests)  
**Verified at**: 2026-06-29  
**Workspace**: `/home/m4s1t4/Work/Enprendimiento/Proyectos/SimPlant/SimPlant-v2/lab`

---

## Completeness

| Metric | Value |
|--------|-------|
| Phase 7 tasks total | 5 |
| Phase 7 tasks marked complete in `tasks.md` | 0 |
| Phase 7 tasks implemented in source | 5 |
| Phase 7 tasks incomplete (implementation) | 0 |
| Phase 7 tasks incomplete (runtime / env) | 0 (with documented env vars) |

| Task | Marked | Evidence |
|------|--------|----------|
| 7.1 Rust unit tests in `sp_python` (`Python::attach`) | ❌ `[ ]` | `crates/simplant/sp_python/src/kernel.rs:436–537` — 3 tests |
| 7.2 `test_simplant_domain.py` — 8 submodules + central types | ❌ `[ ]` | `rerun_py/tests/test_simplant_domain.py:20–45` |
| 7.3 Smoke E2E `tanque_demo` | ❌ `[ ]` | `rerun_py/tests/test_simplant_domain.py:53–84` |
| 7.4 Smoke E2E `sim_demo` | ❌ `[ ]` | `rerun_py/tests/test_simplant_domain.py:92–149` |
| 7.5 Error-path tests | ❌ `[ ]` | `rerun_py/tests/test_simplant_domain.py:157–178` |

**Note:** All five tasks are implemented and passing; `tasks.md` checkboxes were not updated to `[x]`.

---

## Test Execution

### Step 0 — pyo3 config

```bash
pixi run ensure-pyo3-build-cfg
```

**Result:** ✅ Exit 0 — generated `rerun_py/pyo3-build.cfg`

### Step 1 — Rust unit tests (task 7.1)

```bash
LD_LIBRARY_PATH="/home/m4s1t4/Work/Enprendimiento/Proyectos/SimPlant/SimPlant-v2/lab/.pixi/envs/default/lib" \
  cargo test --all-features -p sp_python
```

**Result:** ✅ Exit 0 — **3 passed**, 0 failed

```
test kernel::tests::tag_id_round_trip_and_rejects_empty ... ok
test kernel::tests::measurement_round_trip ... ok
test kernel::tests::time_window_rejects_inverted_range ... ok
```

**Env note:** `CONDA_PREFIX` was unset in the shell; `LD_LIBRARY_PATH=$CONDA_PREFIX/lib` alone fails with `libpython3.11.so.1.0: cannot open shared object file`. Pixi env lib path is required.

### Step 2 — maturin develop

```bash
cd rerun_py && RERUN_ALLOW_MISSING_BIN=1 pixi run maturin develop --manifest-path Cargo.toml
```

**Result:** ✅ Exit 0 — `simplant-lab-sdk-0.33.0a1+dev` installed (editable)

### Step 3 — Python behavior tests (tasks 7.2–7.5)

```bash
PYTHONPATH=rerun_py/rerun_sdk pixi run python -m pytest -vv \
  rerun_py/tests/test_simplant_domain.py -W "ignore::DeprecationWarning"
```

**Result:** ✅ Exit 0 — **7 passed**, 0 failed

| Test | Task | Result |
|------|------|--------|
| `test_submodules_import_and_construct_central_types` | 7.2 | ✅ PASSED |
| `test_tanque_demo_e2e_smoke` | 7.3 | ✅ PASSED |
| `test_sim_demo_e2e_smoke` | 7.4 | ✅ PASSED |
| `test_approve_rejects_nonzero_degrees_of_freedom` | 7.5 | ✅ PASSED |
| `test_safety_factor_rejects_zero` | 7.5 | ✅ PASSED |
| `test_data_split_rejects_overlapping_windows` | 7.5 | ✅ PASSED |
| `test_parse_modbus_address_rejects_bogus` | 7.5 | ✅ PASSED |

**Env notes:**

- Bare `pixi run pytest rerun_py/tests/test_simplant_domain.py` → `ModuleNotFoundError: No module named 'simplant_lab'` (`.pth` adds `rerun_py/` but package lives under `rerun_py/rerun_sdk/` — same pre-existing issue noted in Phase 6).
- With `PYTHONPATH=rerun_py/rerun_sdk` but without `-W ignore::DeprecationWarning`, collection fails: `DeprecationWarning: The 'rerun' package name is deprecated` (pytest `filterwarnings = error` in `rerun_py/pyproject.toml`).

---

## Correctness (Specs vs Phase 7 Tasks)

### Task 7.1 — `specs/kernel`

| Requirement / Scenario | Status | Test evidence |
|------------------------|--------|---------------|
| Construir un TagId válido | ✅ Covered | `tag_id_round_trip_and_rejects_empty` — `PyTagId("FT-101")`, Python `TagId` call |
| Rechazar un TagId inválido (`""`) | ✅ Covered | Same test — `PyValueError` via Rust + Python boundary |
| Crear y leer un Measurement | ✅ Covered | `measurement_round_trip` — value, quality, timestamp |
| Round-trip de timestamp por epoch seconds | ✅ Covered | Same test — tolerance `< 1e-6` |
| Rechazar ventana invertida | ✅ Covered | `time_window_rejects_inverted_range` — `PyValueError` |
| Ventana válida contiene un instante | ✅ Covered | Same test — `contains(150.0)` true, `contains(200.0)` false |

**Out of Phase 7.1 scope (not required by task):** Quality `is_usable`, empty `MeasurementBatch`, `UnitOfMeasure.to_base`, `EngineeringRange` rejection — no tests added (acceptable per task definition).

### Task 7.2 — all specs: "submódulo accesible"

| Submodule | Central type exercised | Spec scenario |
|-----------|------------------------|---------------|
| `kernel` | `TagId("FT-101")` | kernel: Importar submódulo |
| `asset_model` | `Facility.define(...)` | asset-model: Importar submódulo |
| `acquisition` | `TagBinding(...)` | acquisition: Importar submódulo |
| `simulation` | `ChemicalComponent(...)` | simulation: Importar submódulo |
| `ml_dataloop` | `FeatureSpec(...)` | ml-dataloop: Importar submódulo |
| `stress_testing` | `LoadPoint(...)` | stress-testing: Importar submódulo |
| `recording` | `PLANT_TIME`, `tag_entity_path` | recording: Importar submódulo |
| `types` | `ProcessVariableSample(...)` | types: Importar submódulo |

**Result:** ✅ All 8 domain submodules importable and construct a representative type.

Nested adapters (`acquisition.replay`, `simulation.engine`, `ml_dataloop.dataframe_query`, `acquisition.modbus`) are exercised in tasks 7.3–7.5, not 7.2 — consistent with task wording.

### Task 7.3 — `specs/acquisition`, `specs/recording`

| Scenario | Status | Test evidence |
|----------|--------|---------------|
| Cargar catálogo válido (asset-model) | ✅ Covered | `TomlCatalogRepository` → `load_catalog()` → `validate()` |
| Crear sesión con bindings válidos | ✅ Covered | `AcquisitionSession.create(...)` with catalog tags |
| Construir `CsvReplaySource` | ✅ Covered | `CsvReplaySource(str(csv_path))` |
| Ejecutar sesión de replay end-to-end | ✅ Covered | `run_session(...)` → `batches > 0` |
| Recorder a archivo | ✅ Covered | `RerunRecorder.to_file(...)` |
| Flush deja el `.rrd` consistente | ⚠️ Partial | `flush()` + file size > 0 + `RrdDataframeQuery.open(path)` succeeds; **`query(window, tags)` not invoked** |
| Consultar una ventana de tiempo (ml-dataloop) | ⚠️ Partial | Only `open()`; no `query()` round-trip |

### Task 7.4 — `specs/simulation`

| Scenario | Status | Test evidence |
|----------|--------|---------------|
| Draft válido y cálculo de DOF | ✅ Covered | `FlowsheetSpec.draft(...)` + `degrees_of_freedom() == 0` |
| Aprobar flowsheet cuadrado | ✅ Covered | `approve()` + `is_approved()` |
| Aprobar escenario válido | ✅ Covered | `Scenario.approve(...)` + `duration_secs() == 120.0` |
| Inicializar y avanzar la simulación | ✅ Covered | `FirstOrderEngine.initialize` + 60× `step(2.0)`; final state asserts outlet temp/pressure |

### Task 7.5 — error paths (specs varias)

| Scenario | Spec | Status | Test evidence |
|----------|------|--------|---------------|
| Rechazar aprobación con DOF≠0 | simulation | ✅ Covered | `test_approve_rejects_nonzero_degrees_of_freedom` → `ValueError` |
| Rechazar SafetyFactor inválido (`0.0`) | stress-testing | ✅ Covered | `test_safety_factor_rejects_zero` → `ValueError` |
| Rechazar split con fuga temporal | ml-dataloop | ✅ Covered | `test_data_split_rejects_overlapping_windows` → `ValueError` |
| Dirección Modbus inválida (`"bogus"`) | acquisition | ✅ Covered | `test_parse_modbus_address_rejects_bogus` → `ValueError` |

---

## Scenarios Coverage Summary (Phase 7 test scope)

| Scenario (from specs) | Status |
|-----------------------|--------|
| kernel: TagId valid / invalid | ✅ Covered (7.1) |
| kernel: Measurement + timestamp round-trip | ✅ Covered (7.1) |
| kernel: TimeWindow inverted / contains | ✅ Covered (7.1) |
| All 8 submodules importable | ✅ Covered (7.2) |
| acquisition: replay E2E | ✅ Covered (7.3) |
| recording: flush → readable `.rrd` | ⚠️ Partial — open only, no query |
| simulation: full sim_demo chain | ✅ Covered (7.4) |
| simulation: reject approve DOF≠0 | ✅ Covered (7.5) |
| stress-testing: SafetyFactor(0.0) | ✅ Covered (7.5) |
| ml-dataloop: overlapping DataSplit | ✅ Covered (7.5) |
| acquisition: bogus modbus address | ✅ Covered (7.5) |

**Not covered by Phase 7 (deferred / out of task scope):** session lifecycle (`start`/`stop`), binding to missing tag, valid modbus parse happy path, `step` without `initialize`, draft invalid, stress-test plan/evaluate, dataset spec errors, `.rrd` missing file, types constants, etc.

---

## Coherence (Design)

Phase 7 is test-only; no new binding code. Tests align with design decisions:

| Decision | Followed? | Notes |
|----------|-----------|-------|
| `Python::attach` for Rust unit tests | ✅ Yes | All 3 kernel tests use `Python::attach` |
| E2E demos mirror Rust examples | ✅ Yes | `tanque_demo` TOML/CSV paths; `sim_demo` flowsheet structure |
| Error mapping via `map_err` → Python exceptions | ✅ Yes | All 7.5 tests expect `ValueError` |
| `py.detach` for long I/O (run_session, open) | ✅ Yes | E2E tests complete without hang |

---

## Testing

| Area | Tests Exist? | Runtime result |
|------|-------------|----------------|
| 7.1 Rust kernel unit tests | Yes (3) | ✅ 3/3 pass (pixi lib in `LD_LIBRARY_PATH`) |
| 7.2 Submodule import smoke | Yes (1) | ✅ Pass |
| 7.3 tanque_demo E2E | Yes (1) | ✅ Pass |
| 7.4 sim_demo E2E | Yes (1) | ✅ Pass |
| 7.5 Error paths | Yes (4) | ✅ 4/4 pass |
| **Total Python** | **7 tests** | **✅ 7/7 pass** (with env workaround) |

---

## Issues Found

**CRITICAL** (must fix before archive):
- None — all Phase 7 deliverables implemented and tests pass with documented environment.

**WARNING** (should fix):
1. **`tasks.md` Phase 7 checkboxes still `[ ]`** despite implementation complete — tracking drift vs source.
2. **pytest not green on bare `pixi run pytest rerun_py/tests/test_simplant_domain.py`** — requires `PYTHONPATH=rerun_py/rerun_sdk` (packaging path) and `-W ignore::DeprecationWarning` (rerun rename warning vs strict filter).
3. **`cargo test -p sp_python` needs explicit `LD_LIBRARY_PATH`** when `CONDA_PREFIX` is unset (pixi env lib path).
4. **7.3 `.rrd` legibility check is partial** — `RrdDataframeQuery.open()` only; no `query(window, tags)` assertion (ml-dataloop spec scenario partially covered).

**SUGGESTION** (nice to have):
- Extend 7.3 to call `query(TimeWindow, [tag])` and assert non-empty series.
- Add Rust-side tests for `MeasurementBatch.is_empty()` / `Quality.is_usable()` if broader kernel spec coverage is desired.
- Fix editable-install path so `import simplant_lab` works without `PYTHONPATH` (Phase 6 carry-over).
- Mark tasks 7.1–7.5 as `[x]` in `tasks.md`.

---

## Verdict

**PASS WITH WARNINGS**

Phase 7 behavioral tests are fully implemented: 3 Rust unit tests and 7 Python tests cover tasks 7.1–7.5 and the spec scenarios they target. All tests pass when run with the documented environment (`LD_LIBRARY_PATH` for cargo, `PYTHONPATH` + DeprecationWarning filter for pytest). Warnings are limited to unchecked `tasks.md` boxes, pytest/cargo env friction, and partial `.rrd` query coverage in 7.3 — none block Phase 7 completion.
