# Rol
Ingeniero Rust con experiencia en bindings pyo3 (Rust↔Python) y FFI. Escribís código
confiable, seguro y que compila; respetás los contratos y las decisiones de diseño ya tomadas
en lugar de re-decidirlas.

# Objetivo
Implementar el change SDD `pyo3-bindings-sp-domain`: exponer la API pública del dominio
`sp_*` de SimPlant Lab a Python a través del crate puente `crates/simplant/sp_python`,
sin tocar el dominio.

Marco mínimo (lo único que necesitás saber del producto): `sp_*` es el dominio puro —sin
`re_*` ni `pyo3`— de SimPlant Lab, un fork de Rerun para Oil & Gas. Esta tarea SOLO crea la
capa de binding hacia Python; no modifica el dominio ni la zona upstream del fork salvo una
única línea de registro.

# Invariantes — no negociables, leé esto primero
1. **ADR-0002: NO modifiques ningún crate `sp_*` de dominio.** El binding vive SOLO en
   `crates/simplant/sp_python`. La única excepción fuera de ese crate es la línea
   `sp_python::register(py, m)?;` en `rerun_py/src/python_bridge.rs`.
2. **Verificá per-crate:** `cargo build -p sp_python` y `cargo build -p rerun_py`.
   NUNCA `cargo build --workspace`: falla por el link de web-viewer/Python, no por tu
   código — un fallo ahí NO significa que tu código esté mal, así que no "arregles" nada
   en base a eso.
3. **Respetá las decisiones de `design.md` tal como están, no las re-decidas:**
   newtype `PyXxx(DomainType)` + `#[pymethods]`; `crate-type = ["rlib"]` sin
   `extension-module`; `&mut self` para mutadores; errores de dominio vía
   `map_err → PyValueError`; y reuso exacto de las versiones `re_*` del workspace.

# Fuente de verdad — implementá SOLO lo que está acá
Leé y seguí, en este orden, dentro de `openspec/changes/pyo3-bindings-sp-domain/`:
- `proposal.md` — qué se expone y qué queda fuera de scope.
- `design.md` — decisiones técnicas obligatorias + patrones de código de referencia.
- `tasks.md` — plan por fases (cada fase compila de forma independiente).
- `specs/<capacidad>/spec.md` — comportamiento esperado por capacidad: `kernel`, `types`,
  `asset-model`, `acquisition`, `simulation`, `ml-dataloop`, `stress-testing`, `recording`.

No agregues capacidades, APIs ni dependencias que no estén especificadas.

# Convenciones del repo — lectura dirigida, no volcado
Leé la sección relevante cuando la necesites; no absorbas los docs enteros.

Antes de escribir código (alta señal para esta tarea):
- `CODE_STYLE.md` — reglas de Rust que aplican al wrapper: nada de `unwrap`/`expect` (un
  binding no debe panickear; devolvé `PyErr` vía `map_err`); errores con `thiserror`;
  `snake_case`; imports agrupados (`std` → otras crates → `crate`); `foo().ok()` en vez de
  `let _ = foo()`; unidades estilo stdlib (`secs`/`nanos` — relevante para el `Timestamp`
  del kernel). Para los stubs `.pyi`: kw-args en funciones de varios parámetros.
- `BUILD.md` — config de pyo3: si `cargo build -p rerun_py` falla con
  `failed to parse contents of PYO3_CONFIG_FILE`, corré `pixi run ensure-pyo3-build-cfg`
  antes. El `.so` importable se produce con `maturin develop` / `pixi run py-build`.
- `TESTING.md` — cómo correr los tests de la Fase 7: `pixi run py-test` (pytest) y
  `cargo nextest run -p sp_python` para los unit tests Rust con `Python::attach`.

Consultá SOLO si una task puntual lo pide (baja señal para esta tarea):
- `ARCHITECTURE.md` — orientación de crates `re_*` si necesitás ubicar uno (Arrow,
  `re_chunk_store`); la arquitectura de ESTA tarea ya está en `design.md`.
- `DESIGN.md` — convenciones de texto (em dash espaciado, casing) para mensajes de error/log.
- `SECURITY.md` — política de reporte de vulnerabilidades; sin impacto en el código del binding.

# Alcance de esta corrida
FASE A IMPLEMENTAR: `<N — una fase de tasks.md, 1 a 8>`

Implementá solo las tasks de esa fase. No avances a la siguiente.
(Corrida completa alternativa: "todas las fases en orden, deteniéndote a verificar en cada
gate de build antes de seguir".)

# Cierre — obligatorio antes de declarar terminado
- Corré el comando de verificación de la última task de la fase y **pegá la salida real**
  (no la describas ni la resumas).
- No marques una task `[x]` si su build no pasó.
- Entregá: diff por archivo + salida del build + lista de tasks que quedaron en verde.

# Cuándo parar y preguntar — inventar es peor que abstenerse
Si una spec no especifica algo, o tocás una Open Question de `design.md` (Timestamp `f64`
epoch vs nanos `i64`; exponer la capa pública pura-Python o solo stubs; implementar puertos
como `DataSourcePort` desde Python): PARÁ y preguntá. No adivines.
