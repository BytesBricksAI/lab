# Verification report — phase 1

**Change**: `pyo3-bindings-sp-domain`
**Scope**: Phase 1 only (tasks 1.1–1.5 — crate scaffolding)
**Verified at**: 2026-06-28
**Workspace**: `/home/m4s1t4/Work/Enprendimiento/Proyectos/SimPlant/SimPlant-v2/lab`

---

## Completeness

| Metric | Value |
|--------|-------|
| Phase 1 tasks total | 5 |
| Phase 1 tasks marked complete | 5 |
| Phase 1 tasks incomplete | 0 |

All Phase 1 tasks (1.1–1.5) are marked `[x]` in `tasks.md`.

| Task | Marked | Evidence |
|------|--------|----------|
| 1.1 `Cargo.toml` (rlib, pyo3, 12 sp_*) | ✅ | `crates/simplant/sp_python/Cargo.toml` |
| 1.2 Workspace member | ✅ | Covered by glob `crates/simplant/*` in root `Cargo.toml`; `Cargo.lock` lists `sp_python` |
| 1.3 `error.rs` with `map_err` | ✅ | `crates/simplant/sp_python/src/error.rs` |
| 1.4 `lib.rs` with `register` + mod stubs | ✅ | `crates/simplant/sp_python/src/lib.rs` + 8 capability stub files |
| 1.5 `cargo build -p sp_python` | ✅ | Build passes after pyo3 config fix (see Build Verification) |

---

## Correctness vs design

### Task 1.1 — `Cargo.toml`

| Requirement | Status | Notes |
|-------------|--------|-------|
| `crate-type = ["rlib"]` | ✅ | Line 17 of `sp_python/Cargo.toml` |
| `pyo3` without `extension-module` | ✅ | `pyo3.workspace = true` only; no feature flags (contrast `rerun_py` which adds `abi3-py310` + `extension-module` via its own features) |
| 12 `sp_*` path deps | ✅ | All 12 present: `sp_acquisition`, `sp_acquisition_modbus`, `sp_acquisition_replay`, `sp_asset_model`, `sp_dataframe_query`, `sp_kernel`, `sp_ml_dataloop`, `sp_recording`, `sp_sim_engine`, `sp_simulation`, `sp_stress_testing`, `sp_types` |
| Workspace `re_*` versions | ✅ | No direct `re_*` in `sp_python`; transitive via `sp_recording`, `sp_dataframe_query`, `sp_types` using `*.workspace = true` |

### Task 1.3 — `error.rs`

| Requirement | Status | Notes |
|-------------|--------|-------|
| `pub fn map_err<E: std::error::Error>(e: E) -> PyErr` | ✅ | Matches design contract |
| Maps to `PyValueError` | ✅ | Uses `PyValueError::new_err(e.to_string())` |

Design also mentions `PyRuntimeError` in the File Changes table comment; Phase 1 task text specifies only `PyValueError` — implementation matches Phase 1 task.

### Task 1.4 — `lib.rs` + register stubs

| Requirement | Status | Notes |
|-------------|--------|-------|
| `pub fn register(py, parent) -> PyResult<()>` | ✅ | Public, correct signature |
| 8 capability `mod` declarations | ✅ | `acquisition`, `asset_model`, `kernel`, `ml_dataloop`, `recording`, `simulation`, `stress_testing`, `types` |
| Each `*::register` called in order | ✅ | Matches design.md register contract |
| Empty stub `register` per module | ✅ | Each file exports `pub fn register(...) -> Ok(())` with no pyo3 registrations yet |
| `pub use error::map_err` | ✅ | Re-exported from `lib.rs` |

### ADR-0002 — no domain crate changes

| Requirement | Status | Notes |
|-------------|--------|-------|
| No `pyo3` in domain `sp_*` crates | ✅ | Only `sp_python/Cargo.toml` declares `pyo3` under `crates/simplant/` |
| No modifications to domain crates for Phase 1 | ❌ | **Uncommitted changes in `sp_recording`** (see Issues) |

---

## Build verification

### Initial failure (pre-fix)

```bash
cargo build -p sp_python
```

**Real output (before fix):**

```
   Compiling pyo3-build-config v0.26.0
   Compiling pyo3-macros-backend v0.26.0
   Compiling pyo3-ffi v0.26.0
   Compiling pyo3 v0.26.0
error: could not find native static library `python3.11.a`, perhaps an -L flag is missing?

error: could not compile `pyo3-ffi` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish…
```

**Exit code:** 101

**Root cause:** `.cargo/config.toml` pins `PYO3_CONFIG_FILE` to `rerun_py/pyo3-build.cfg`. The pixi conda Python reports `Py_ENABLE_SHARED=0` and `LDLIBRARY=libpython3.11.a` via `sysconfig`, but only ships `libpython3.11.so` on disk. `rerun_pixi_env/pyo3_config.py` trusted sysconfig blindly, emitting `shared=false` / `lib_name=python3.11.a`.

### Fix applied

**File changed:** `rerun_pixi_env/src/rerun_pixi_env/pyo3_config.py`

Added `_resolve_linkage()` to probe `lib_dir` on disk and prefer shared linkage when the reported static `.a` is absent but the matching `.so` exists. This keeps static linkage when a real `.a` is present (e.g. system Python dev packages).

### Post-fix verification

```bash
pixi run ensure-pyo3-build-cfg
cargo build -p sp_python
```

**Generated `rerun_py/pyo3-build.cfg`:**

```
implementation=CPython
version=3.10
shared=true
abi3=true
lib_name=python3.11
lib_dir=…/.pixi/envs/default/lib
executable=…/.pixi/envs/default/bin/python3.11
pointer_width=64
build_flags=
suppress_build_script_link_lines=false
```

**Real output (after fix):**

```
   Compiling pyo3-ffi v0.26.0
   Compiling pyo3 v0.26.0
   Compiling sp_python v0.33.0-alpha.1+dev (.../crates/simplant/sp_python)
    Finished `dev` profile [optimized] target(s) in 4.10s
```

**Exit code:** 0

**Also verified:** `cargo build -p rerun_py` succeeds with the same config (exit 0, ~8m24s). `sp_python/Cargo.toml` needs no extra pyo3 feature flags — `pyo3.workspace = true` without `extension-module` is correct for an `rlib` bridge crate.

---

## Domain crate change audit

```bash
git status --short crates/simplant/sp_*
```

**Output:**

```
 M crates/simplant/sp_recording/src/lib.rs
 M crates/simplant/sp_recording/src/recorder.rs
?? crates/simplant/sp_python/
```

**`sp_recording` diff summary:** Adds a regression test `events_are_not_anchored_to_plant_time` and modifies `RerunRecorder::record_event` behavior (plant_time anchoring fix). These changes are **outside Phase 1 scope** and violate the proposal constraint that domain crates are not modified for this change.

All other 11 domain crates show no modifications.

---

## Coherence (Design decisions — phase 1 relevant)

| Decision | Followed? | Notes |
|----------|-----------|-------|
| Single bridge crate `sp_python` | ✅ Yes | Crate created at expected path |
| `crate-type = ["rlib"]`, not extension-module | ✅ Yes | Confirmed in Cargo.toml and pyo3 dep |
| `register(py, parent)` central entry | ✅ Yes | Implemented with 8 submodule stubs |
| Domain purity (ADR-0002) | ⚠️ Partial | No pyo3 in domain; but `sp_recording` was modified |
| Workspace membership | ✅ Yes | Via `crates/simplant/*` glob (no explicit line added) |

---

## Testing

Phase 1 has no test tasks. No `#[cfg(test)]` blocks exist in `sp_python` yet (expected — tests are Phase 7).

| Area | Tests Exist? | Coverage |
|------|-------------|----------|
| `sp_python` unit tests | No | N/A for Phase 1 |
| Build smoke (`cargo build -p sp_python`) | Attempted | **Pass** (after pyo3 config fix) |

---

## Issues found

### CRITICAL (must fix before archive / phase 2)

1. **`sp_recording` domain crate modified (scope violation).**
   Uncommitted changes in `src/lib.rs` and `src/recorder.rs` are unrelated to Phase 1 scaffolding. Revert or move to a separate change to preserve ADR-0002 / proposal isolation.

### WARNING (should fix)

1. **Task 1.2 implicit vs explicit membership.**
   `sp_python` is included via workspace glob `crates/simplant/*`, not an explicit member entry. Functionally correct; consider documenting that the glob covers new simplant crates.

### FIXED (this session)

1. **`cargo build -p sp_python` pyo3 link failure (task 1.5).**
   Fixed in `rerun_pixi_env/src/rerun_pixi_env/pyo3_config.py`: `_resolve_linkage()` probes `lib_dir` and selects shared linkage when conda Python reports static `.a` but only ships `.so`.

### SUGGESTION (nice to have)

1. Add a minimal `#[cfg(test)]` in `lib.rs` that calls `register` under `Python::attach` once Phase 2 begins — not required for Phase 1.

---

## Verdict

**PASS (build)** / **PARTIAL (scope)**

Phase 1 scaffolding code (tasks 1.1–1.5) is structurally correct and aligned with design. Task 1.5 now passes after fixing pyo3 config generation for conda/pixi Python. Uncommitted `sp_recording` changes still violate the no-domain-modifications constraint and should be split before archive.

---

## Structured envelope

```yaml
status: completed
executive_summary: >
  Phase 1 tasks 1.1–1.5 are implemented correctly (rlib crate, pyo3 without
  extension-module, map_err, register with 8 empty stubs, 12 sp_* deps).
  cargo build -p sp_python passes after fixing pyo3_config.py to detect shared
  libpython on disk when conda sysconfig reports static .a. sp_recording still
  has out-of-scope uncommitted modifications.
verdict: PASS (build) / PARTIAL (scope)
critical_issues:
  - "sp_recording/src/lib.rs and recorder.rs modified — violates no domain changes rule"
warning_issues:
  - "Workspace membership via crates/simplant/* glob, not explicit member line"
fixed_issues:
  - "pyo3-build.cfg shared=false incompatible with conda shared-only libpython — fixed in rerun_pixi_env"
artifacts:
  - openspec/changes/pyo3-bindings-sp-domain/verify-report-phase1.md
  - rerun_pixi_env/src/rerun_pixi_env/pyo3_config.py
next_recommended:
  - "Revert or split sp_recording changes into a separate change"
  - "Proceed to Phase 2 after scope cleanup"
risks:
  - "Mixed sp_recording changes risk conflating binding work with domain fixes in review"
```
