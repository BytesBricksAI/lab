# Verification Report — Phase 4

**Change**: `pyo3-bindings-sp-domain`  
**Scope**: Phase 4 only (tasks 4.1–4.5 — acquisition, replay, modbus, run_session)  
**Verified at**: 2026-06-28  
**Workspace**: `/home/m4s1t4/Work/Enprendimiento/Proyectos/SimPlant/SimPlant-v2/lab`

---

## Completeness

| Metric | Value |
|--------|-------|
| Phase 4 tasks total | 5 |
| Phase 4 tasks marked complete in `tasks.md` | 5 |
| Phase 4 tasks implemented in source | 5 |
| Phase 4 tasks incomplete (implementation) | 0 |

| Task | Marked | Evidence |
|------|--------|----------|
| 4.1 `PyTagBinding`, `PySamplingPolicy`, `SessionState`, `PyAcquisitionSession` | ✅ `[x]` | `crates/simplant/sp_python/src/acquisition.rs:15–124` |
| 4.2 submódulo `replay` con `PyCsvReplaySource` | ✅ `[x]` | `acquisition.rs:174–200` |
| 4.3 submódulo `modbus` con adapters y helpers | ✅ `[x]` | `acquisition.rs:202–293` |
| 4.4 `run_session` + resolución de fuente concreta | ✅ `[x]` | `acquisition.rs:126–172` |
| 4.5 `acquisition::register` cableado en `lib.rs` | ✅ `[x]` | `acquisition.rs:298–310`, `lib.rs:32` |

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
    Finished `dev` profile [optimized] target(s) in 0.37s
```

### Step 2 — `cargo build -p rerun_py`

```bash
cargo build -p rerun_py
```

**Result:** ✅ Exit 0

```
    Finished `dev` profile [optimized] target(s) in 0.44s
```

### Step 3 — Runtime smoke (not in Phase 4 task list; noted for context)

Python import / maturin smoke was **not** executed in this Phase 4 verification pass. Prior phases report maturin blocked by missing `rerun` CLI artifact. `recording::register` remains a no-op stub (Phase 5), so `simplant_lab.recording.RerunRecorder` is not yet importable at runtime even if maturin succeeded.

---

## Correctness (Specs — `specs/acquisition/spec.md`)

| Requirement | Status | Notes |
|------------|--------|-------|
| Submódulo `simplant_lab.acquisition` accesible | ✅ Implemented | `acquisition::register` registers classes, `run_session`, and submodules `replay`/`modbus` via `attach_simplant_submodule` |
| Crear sesión de adquisición | ✅ Implemented | `AcquisitionSession.create(id, bindings, policy, catalog)` delegates to domain; catalog validation → `map_err` → Python exception |
| `TagBinding(tag, address)` / `SamplingPolicy(period_ms, deadband)` | ✅ Implemented | `PyTagBinding::new`, `PySamplingPolicy::new` with optional deadband |
| Transiciones de estado (`start`/`stop`, `state()`) | ✅ Implemented | `&mut self` methods; domain state machine `Created → Running → Stopped`; invalid `stop` without `start` surfaces via `map_err` |
| Orquestación `run_session` con adapters nativos | ✅ Implemented | `py_run_session` calls `sp_acquisition::run_session` inside `py.detach`; returns `u64` |
| Adapter replay CSV | ✅ Implemented | `PyCsvReplaySource(path)` in `acquisition.replay` |
| Adapter Modbus TCP y direccionamiento | ✅ Implemented | `ModbusTcpSource`, `parse_modbus_address`, `map_register`, `RegisterKind`, `ModbusPoint` with getters |
| Puertos desde Python (fuera de alcance) | ✅ Documented / enforced | Non-native sources raise `TypeError: source must be CsvReplaySource or ModbusTcpSource` |

**Scenarios Coverage (source-level; no runtime pytest in Phase 4):**

| Scenario | Status | Evidence |
|----------|--------|----------|
| Importar submódulo y adapters | ⚠️ Partial | Registration code present; runtime import not verified (maturin / recording stub) |
| Crear sesión con bindings válidos | ⚠️ Partial | `create` + domain validation implemented; no binding test yet |
| Binding a tag inexistente | ⚠️ Partial | Domain `UnknownTag` mapped via `map_err`; no test yet |
| Ciclo de vida de la sesión | ⚠️ Partial | `start`/`stop`/`state` wired; no test yet |
| Stop sin start | ⚠️ Partial | Domain rejects invalid transition; mapped to Python exception |
| Ejecutar replay end-to-end | ⚠️ Partial | `run_session` + `PyRerunRecorder` compile; `RerunRecorder` not registered for Python (Phase 5) |
| Construir `CsvReplaySource` | ✅ Implemented | Constructor stores path; accepted by `resolve_data_source` |
| Parsear dirección Modbus válida | ✅ Implemented | `parse_modbus_address` → `PyModbusPoint` with `kind`/`register`/`scale`/`offset` |
| Dirección Modbus inválida | ⚠️ Partial | Domain error via `map_err`; no test yet |

---

## Coherence (Design — `design.md`)

| Decision | Followed? | Notes |
|----------|-----------|-------|
| Newtype wrappers for value objects | ✅ Yes | `PyTagBinding`, `PySamplingPolicy`, `PyAcquisitionSession` wrap domain types |
| Native `#[pyclass]` enum for `SessionState` | ✅ Yes | `SessionState` with `eq_int` |
| `&mut self` for session mutators | ✅ Yes | `start`, `stop` on `PyAcquisitionSession` |
| Concrete port adapters (no Python trait objects) | ✅ Yes | `resolve_data_source` accepts only `PyCsvReplaySource` / `PyModbusTcpSource` |
| `py.detach` for long I/O in orchestration | ✅ Yes | `py_run_session` uses `py.detach(|| domain_run_session(...))` |
| `acquisition.rs` file location and submodule layout | ✅ Yes | Matches File Changes table |
| Public `PyDataSource` enum/wrapper (design sketch) | ⚠️ Deviated | Uses private `OwnedDataSource` + `resolve_data_source(&Bound<PyAny>)` instead of exported `PyDataSource` type; behavior equivalent |
| `recording.rs` minimal stub for Phase 4 compile | ✅ Yes | `PyRerunRecorder` with `to_file`/`flush` exists; `recording::register` is no-op (Phase 5 completes registration) |
| `lib.rs::register` wires acquisition | ✅ Yes | Line 32 calls `acquisition::register` |
| Domain `sp_*` crates unchanged | ✅ Yes | Bindings only in `sp_python` |

---

## Task-by-Task Verification (4.1–4.5)

### 4.1 — Session types and state machine

| Item | Spec / task | Implementation |
|------|-------------|----------------|
| `TagBinding` | Constructible; tag + address | `PyTagBinding::new(tag: PyTagId, address: String)` |
| `SamplingPolicy` | `period_ms`, optional `deadband` | `PySamplingPolicy::new(period_ms, deadband=None)` |
| `SessionState` enum | Created / Running / Stopped | Native pyclass enum |
| `AcquisitionSession.create` | Static; validates catalog | Forwards to `AcquisitionSession::create` |
| `start` / `stop` | `&mut self` | Present; errors mapped |
| Getters | id, bindings, policy, state | All present |

### 4.2 — Replay adapter

| Item | Implementation |
|------|----------------|
| Submodule `replay` | `replay::register` adds nested module |
| `CsvReplaySource(path)` | `PyCsvReplaySource::new(path: String)` |

### 4.3 — Modbus adapter

| Item | Implementation |
|------|----------------|
| `ModbusTcpSource(host:port)` | `PyModbusTcpSource::new(host_port: String)` with socket parse error mapping |
| `parse_modbus_address(s)` | `py_parse_modbus_address` |
| `map_register(raw, point)` | `py_map_register` |
| `RegisterKind` / `ModbusPoint` | Enums + class with getters |

### 4.4 — `run_session` orchestration

| Item | Design / spec | Implementation |
|------|---------------|----------------|
| Accepts session, catalog, source, recorder | ✅ | Signature matches |
| Source deref to `&dyn DataSourcePort` | ✅ | `OwnedDataSource::as_port()` |
| Recorder deref | ✅ | `&recorder.0` (`RerunRecorder` implements `RecorderPort`) |
| Returns batch count | ✅ | `PyResult<u64>` |
| Rejects non-native sources | ✅ | `PyTypeError` |

### 4.5 — Registration

| Item | Implementation |
|------|----------------|
| Classes + `run_session` on `acquisition` module | ✅ |
| Nested `replay` / `modbus` | ✅ |
| Wired in `lib.rs::register` | ✅ line 32 |

---

## Testing

| Area | Tests Exist? | Coverage |
|------|-------------|----------|
| `sp_python` acquisition wrappers | No | None — Phase 7 deferred |
| Domain `run_session` | Yes (Rust) | `sp_acquisition` unit test covers orchestration |
| Python smoke / E2E acquisition | No | Phase 7.3 deferred |

---

## Issues Found

**CRITICAL** (must fix before archive):

None for Phase 4 scope.

**WARNING** (should fix):

1. **No behavioral tests for acquisition bindings** — Phase 7 not started; spec scenarios (session lifecycle, invalid binding, modbus parse errors) unverified at Python boundary.
2. **`recording::register` is a no-op** — `PyRerunRecorder` is not exposed on `simplant_lab.recording`; blocks end-to-end `run_session` smoke from Python until Phase 5.1 (expected cross-phase dependency).
3. **Runtime import not verified** — Registration code is correct at compile time; maturin / Python import not run in this pass (consistent with prior phase reports).
4. **Design sketch `PyDataSource` not exported** — Internal `OwnedDataSource` + extraction pattern works but differs from design interface sketch; document or align in stubs (Phase 6).

**SUGGESTION** (nice to have):

1. `PyCsvReplaySource` / `PyModbusTcpSource` re-instantiate domain adapters on each `run_session` call from stored path/addr rather than holding live domain objects — acceptable for replay/modbus; consider documenting semantics.
2. Add `#[cfg(test)]` round-trip tests for `parse_modbus_address("holding:40001:0.1:5.0")` when Phase 7 begins.

---

## Verdict

**PASS WITH WARNINGS**

Phase 4 acquisition bindings (session types, replay/modbus adapters, `run_session` orchestration) are fully implemented in source and align with `specs/acquisition/spec.md` and `design.md` within Phase 4 scope. Both `cargo build -p sp_python` and `cargo build -p rerun_py` pass after `ensure-pyo3-build-cfg`. Warnings are limited to deferred runtime smoke (recording registration Phase 5, pytest Phase 7) and a minor design naming deviation for the data-source wrapper type.

---

## Structured Envelope (for orchestrator)

```yaml
status: pass_with_warnings
executive_summary: >
  Phase 4 tasks 4.1–4.5 are implemented and compile cleanly. Acquisition session
  types, replay/modbus adapters, and run_session match the acquisition spec and
  design. Runtime E2E and Python behavioral tests remain deferred to Phases 5 and 7.
artifacts:
  - openspec/changes/pyo3-bindings-sp-domain/verify-report-phase4.md
next_recommended:
  - Proceed to Phase 5 (recording registration, sim engine, dataframe_query)
  - Phase 7 acquisition error/lifecycle tests when test phase starts
risks:
  - run_session E2E from Python blocked until recording::register completes (5.1)
  - No pytest coverage for acquisition boundary errors yet
```
