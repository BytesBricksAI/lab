# Verification report — phase 2

**Change**: `pyo3-bindings-sp-domain`
**Scope**: Phase 2 only (tasks 2.1–2.5 — kernel & types leaf capabilities)
**Verified at**: 2026-06-28
**Workspace**: `/home/m4s1t4/Work/Enprendimiento/Proyectos/SimPlant/SimPlant-v2/lab`

---

## Completeness

| Metric | Value |
|--------|-------|
| Phase 2 tasks total | 5 |
| Phase 2 tasks marked complete | 5 |
| Phase 2 tasks incomplete | 0 |

All Phase 2 tasks (2.1–2.5) are marked `[x]` in `tasks.md`.

| Task | Marked | Evidence |
|------|--------|----------|
| 2.1 `kernel.rs` pyclasses + enums | ✅ | `crates/simplant/sp_python/src/kernel.rs` |
| 2.2 Timestamp as epoch/nanos/ISO | ✅ | `timestamp_from_epoch_secs`, `timestamp_nanos`, `*_iso`, `__str__` in `kernel.rs` |
| 2.3 `kernel::register` wired in `lib.rs` | ✅ | `kernel.rs:427-440`, `lib.rs:30` |
| 2.4 `types.rs` + `types::register` | ✅ | `crates/simplant/sp_python/src/types.rs` |
| 2.5 Build + integration smoke | ⚠️ Partial | `cargo build -p rerun_py` passes; `maturin develop` + Python import fail (see Build Verification) |

---

## Build verification

### Step 0 — pyo3 config

```bash
pixi run ensure-pyo3-build-cfg
```

**Result:** ✅ Exit 0 — generated `rerun_py/pyo3-build.cfg` with `shared=true`.

### Step 1 — `cargo build -p sp_python`

```bash
cargo build -p sp_python
```

**Result:** ✅ Exit 0 — `Finished dev profile [optimized] target(s) in 0.51s`

### Step 2 — `cargo build -p rerun_py`

```bash
cargo build -p rerun_py
```

**Result:** ✅ Exit 0 — `Finished dev profile [optimized] target(s) in 0.64s`

### Step 3 — task 2.5 runtime smoke (maturin + python)

```bash
cd rerun_py && pixi run maturin develop
```

**Result:** ❌ Exit 1 — build script error:

```
ERROR: Expected to find `rerun` at "…/rerun_py/rerun_sdk/rerun_cli/rerun"
```

```bash
pixi run python -c "import simplant_lab; simplant_lab.kernel.TagId('FT-101'); simplant_lab.types"
```

**Result:** ❌ `ModuleNotFoundError: No module named 'simplant_lab'` (extension not installed after maturin failure).

**Note:** Compile-time linking is fixed and both crates build. Runtime Python verification is blocked by a missing `rerun` CLI artifact required by `rerun_py`'s build script, not by Phase 2 binding code.

---

## Correctness (Specs)

### Specs/kernel — requirements

| Requirement | Status | Notes |
|-------------|--------|-------|
| Submódulo `simplant_lab.kernel` accesible | ✅ Implemented | `kernel::register` creates submodule; `attach_simplant_submodule` also sets `simplant_lab.kernel` at init |
| `TagId` con validación ISA-5.1 | ✅ Implemented | `PyTagId::new` → `TagId::new(...).map_err(map_err)`; `as_str()` + `__str__` |
| `Quality` como enum + `is_usable()` | ✅ Implemented | Native `#[pyclass] enum Quality` with `is_usable()` delegating to domain |
| `Measurement` y `MeasurementBatch` | ✅ Implemented | All required getters: `value/quality/timestamp`, `tag/samples/len/is_empty/time_span` |
| `TimeWindow` con invariante start<end | ✅ Implemented | `TimeWindow::new(...).map_err(map_err)`; `contains/overlaps/duration` |
| `UnitOfMeasure`, `EngineeringRange`, `AlarmLimits` | ✅ Implemented | Enums + newtypes; `to_base/from_base/same_dimension`, range validation via `map_err`, alarm Option getters |
| `Timestamp` interoperable con Python | ✅ Implemented | Epoch `f64` via `timestamp()/start()/end()`; nanos via `timestamp_nanos/start_nanos/end_nanos`; ISO via `*_iso()` and `__str__` on Measurement/TimeWindow |

**Scenarios Coverage (kernel):**

| Scenario | Status | Notes |
|----------|--------|-------|
| Importar el submódulo | ⚠️ Partial | Registration code correct; runtime import not exercised (maturin blocked) |
| Construir TagId válido | ⚠️ Partial | Code path verified in source; no runtime test |
| Rechazar TagId inválido | ⚠️ Partial | `map_err` → `PyValueError`; no runtime test |
| Calidad utilizable / no utilizable | ⚠️ Partial | `is_usable()` implemented; no runtime test |
| Crear y leer Measurement | ⚠️ Partial | Constructor + getters implemented |
| Batch vacío `is_empty` / `time_span()` None | ⚠️ Partial | Delegates to domain `MeasurementBatch` |
| Ventana válida `contains` | ⚠️ Partial | Implemented via `timestamp_from_epoch_secs` + domain |
| Rechazar ventana invertida | ⚠️ Partial | Domain error mapped via `map_err` |
| Conversión unidad a base | ⚠️ Partial | `UnitOfMeasure.to_base()` exposed |
| Rango rechaza low>=high | ⚠️ Partial | `EngineeringRange::new(...).map_err(map_err)` |
| Round-trip timestamp epoch | ⚠️ Partial | `timestamp()` returns `f64` from domain; no round-trip test |

### Specs/types — requirements

| Requirement | Status | Notes |
|-------------|--------|-------|
| Submódulo `simplant_lab.types` accesible | ✅ Implemented | `types::register` + `attach_simplant_submodule` |
| `ProcessVariableSample`, `TagMetadata`, `Quality`, namespace constants | ✅ Implemented | Classes + `types.add("Quality", py.get_type::<Quality>())` reusing kernel enum |
| Construcción desde primitivos + kernel types | ✅ Implemented | `ProcessVariableSample(value, quality)`, `TagMetadata(unit, range_low, range_high)` |
| Helper `field(archetype, field_name)` | ✅ Implemented | `py_field` wraps `sp_types::field` |

**Scenarios Coverage (types):**

| Scenario | Status | Notes |
|----------|--------|-------|
| Importar submódulo y leer constantes | ⚠️ Partial | Constants re-exported from `sp_types::namespace`; runtime not exercised |
| Crear ProcessVariableSample | ⚠️ Partial | Constructor + getters implemented |
| Componer nombre de campo | ⚠️ Partial | `field()` returns `"{archetype}:{field}"` per domain |

---

## Task-by-Task verification

### Task 2.1 — `kernel.rs` pyclasses

| Item | Expected | Found |
|------|----------|-------|
| `PyTagId` newtype + getters | ✅ | `#[pyclass(name = "TagId")]` lines 147-165 |
| `PyMeasurement` | ✅ | lines 167-216 |
| `PyMeasurementBatch` | ✅ | lines 218-256 |
| `PyTimeWindow` | ✅ | lines 258-317 |
| `PyEngineeringRange` | ✅ | lines 319-364 |
| `PyAlarmLimits` | ✅ | lines 366-425 |
| Enums `Quality`, `UnitOfMeasure`, `Dimension` | ✅ | Native `#[pyclass] enum` per design |
| Constructors use `map_err` | ✅ | TagId, TimeWindow, EngineeringRange, AlarmLimits |

### Task 2.2 — timestamp exposure

| Item | Expected | Found |
|------|----------|-------|
| Epoch seconds `f64` | ✅ | `timestamp()`, `start()`, `end()` |
| Nanos `i64` | ✅ | `timestamp_nanos()`, `start_nanos()`, `end_nanos()` |
| ISO-8601 | ✅ | `timestamp_iso()`, `start_iso()`, `end_iso()`; `__str__` uses ISO on Measurement/TimeWindow |

### Task 2.3 — `kernel::register` wiring

| Item | Expected | Found |
|------|----------|-------|
| Creates `kernel` submodule | ✅ | `PyModule::new(py, "kernel")` |
| Registers all classes | ✅ | TagId, Quality, Measurement, MeasurementBatch, TimeWindow, Dimension, UnitOfMeasure, EngineeringRange, AlarmLimits |
| Called from `lib.rs::register` | ✅ | `lib.rs:30` |

### Task 2.4 — `types.rs`

| Item | Expected | Found |
|------|----------|-------|
| `PyProcessVariableSample` | ✅ | value + quality getters |
| `PyTagMetadata` | ✅ | unit_symbol, range, alarm getters |
| `Quality` reuse from kernel | ✅ | `types.add("Quality", py.get_type::<Quality>())` |
| Namespace constants | ✅ | `ARCHETYPE_PROCESS_VARIABLE`, `ARCHETYPE_TAG_METADATA`, `COMPONENT_QUALITY` |
| `field(...)` helper | ✅ | `py_field` → `sp_types::field` |
| `types::register` wired | ✅ | `lib.rs:37` |

### Task 2.5 — integration

| Item | Expected | Found |
|------|----------|-------|
| `sp_python` dep in `rerun_py/Cargo.toml` | ✅ | Line 87 |
| `sp_python::register(py, m)?` in `python_bridge.rs` | ✅ | Line 384 inside `#[pymodule] fn rerun_bindings` |
| `cargo build -p rerun_py` | ✅ | Passes |
| `maturin develop` | ❌ | Fails — missing `rerun` CLI binary |
| Python smoke import | ❌ | `simplant_lab` not installed |

---

## Coherence (Design)

| Decision | Followed? | Notes |
|----------|-----------|-------|
| Single bridge crate `sp_python` | ✅ Yes | Kernel/types live in `sp_python` |
| `crate-type = ["rlib"]`, pyo3 without `extension-module` | ✅ Yes | Confirmed in `sp_python/Cargo.toml` |
| Newtype wrapper for structs with invariants | ✅ Yes | TagId, Measurement, TimeWindow, EngineeringRange, AlarmLimits |
| Native `#[pyclass] enum` for C-like enums | ✅ Yes | Quality, UnitOfMeasure, Dimension |
| `map_err` → `PyValueError` | ✅ Yes | `error.rs` + used in constructors |
| `Timestamp` as f64 epoch + ISO string | ✅ Yes | No `datetime` dependency; matches design |
| `register(py, parent)` central entry | ✅ Yes | kernel + types registered; other modules remain stubs |
| `rerun_py` gains one dep + one register line | ✅ Yes | `Cargo.toml` + `python_bridge.rs:384` |
| Domain purity (ADR-0002) — no `sp_*` changes for bindings | ⚠️ Partial | `sp_python` is new (expected); pre-existing uncommitted `sp_recording` changes still present (out of Phase 2 scope) |
| File changes match design table | ✅ Yes | `kernel.rs`, `types.rs`, `lib.rs`, `python_bridge.rs`, `rerun_py/Cargo.toml` |

---

## Testing

Phase 2 has no dedicated test tasks (tests are Phase 7). No `#[cfg(test)]` blocks exist in `sp_python`.

| Area | Tests Exist? | Coverage |
|------|-------------|----------|
| `sp_python` unit tests (TagId, Measurement, TimeWindow) | No | None — deferred to Phase 7 |
| `cargo build -p sp_python` | Yes | ✅ Pass |
| `cargo build -p rerun_py` | Yes | ✅ Pass |
| `maturin develop` + Python import smoke | Attempted | ❌ Blocked by missing rerun CLI |
| `test_simplant_domain.py` | No | Not created yet (Phase 7) |

---

## Issues found

### CRITICAL (must fix before archive)

None for Phase 2 implementation correctness. Compile-time verification passes.

### WARNING (should fix)

1. **Task 2.5 runtime smoke not satisfied.** `maturin develop` fails because `rerun_py` build script expects `rerun_sdk/rerun_cli/rerun` binary. Python import of `simplant_lab.kernel` / `simplant_lab.types` could not be exercised.

2. **No behavioral tests for kernel/types scenarios.** All spec scenarios are implemented in source but lack runtime verification (Phase 7 not started).

3. **Pre-existing `sp_recording` domain modifications still uncommitted** (`src/lib.rs`, `src/recorder.rs`). Carried from Phase 1; outside Phase 2 scope but violates proposal ADR-0002 isolation.

4. **Timestamp ISO via `str()` on instantes is indirect.** Spec SHOULD exposes ISO via `str()` on instantes; design chose separate `*_iso()` accessors and `__str__` on container types (Measurement/TimeWindow). Functionally adequate but not a literal standalone `Timestamp` type with `__str__`.

### SUGGESTION (nice to have)

1. Add Phase 7.1 unit tests early for TagId rejection and inverted TimeWindow — highest-value invariant checks.

2. Document pixi/maturin prerequisite (`rerun` CLI build) in dev workflow so task 2.5 smoke can run reliably.

---

## Verdict

**PASS WITH WARNINGS**

Phase 2 kernel and types bindings are fully implemented and align with specs/kernel, specs/types, and design.md. Both `cargo build -p sp_python` and `cargo build -p rerun_py` pass after `ensure-pyo3-build-cfg`. Runtime Python smoke (maturin + import) is blocked by a missing `rerun` CLI artifact, and behavioral tests remain deferred to Phase 7.

---

## Structured envelope

```yaml
status: completed
executive_summary: >
  Phase 2 tasks 2.1–2.4 are fully implemented (kernel/types pyclasses, timestamp
  interop, register wiring, rerun_py integration). cargo build -p sp_python and
  cargo build -p rerun_py both pass. maturin develop and Python import smoke fail
  due to missing rerun CLI binary; no runtime behavioral tests yet.
verdict: PASS WITH WARNINGS
critical_issues: []
warning_issues:
  - "maturin develop fails — missing rerun_sdk/rerun_cli/rerun binary (task 2.5 runtime smoke)"
  - "Python import simplant_lab.kernel/types not exercised"
  - "No #[cfg(test)] coverage for kernel/types invariants (Phase 7 deferred)"
  - "Uncommitted sp_recording changes still present (pre-existing, ADR-0002 scope)"
artifacts:
  - openspec/changes/pyo3-bindings-sp-domain/verify-report-phase2.md
next_recommended:
  - "Build/install rerun CLI or stub build-script check to unblock maturin develop"
  - "Run task 2.5 Python smoke after maturin succeeds"
  - "Proceed to Phase 3 (asset_model, simulation, stress_testing, ml_dataloop)"
  - "Address or split sp_recording changes before archive"
risks:
  - "Runtime submodule registration untested until maturin install succeeds"
  - "Quality u8 mapping in types.rs could drift if sp_types::Quality encoding changes"
```
