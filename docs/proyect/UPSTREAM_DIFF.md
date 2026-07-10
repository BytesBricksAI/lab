# UPSTREAM_DIFF — registro de cambios sobre Rerun upstream

| Campo | Valor |
|---|---|
| **Rama** | `feat/simplant-domain-crates` |
| **Base upstream** | Rerun `0.33.0-alpha.1+dev` (`rerun-io/rerun`) |
| **Fecha del registro** | 2026-06-15 |
| **Fuente** | `git status --short`, `git diff --stat`, diffs puntuales |
| **Documentos relacionados** | [`MIGRATION_PLAN.md`](MIGRATION_PLAN.md) · [`implementation-notes.md`](../../implementation-notes.md) |

---

## Propósito

Este documento es el **registro curado** de todo cambio aplicado sobre la zona upstream forkeada de Rerun (crates `re_*`, `rerun_py`, ejemplos, tests, docs, CI, etc.). Su objetivo es permitir **rebases y merges periódicos de upstream** sabiendo exactamente **qué se tocó** y **por qué**, sin tener que re-leer miles de líneas de diff.

**Zona excluida de este registro** (código original SimPlant, no upstream):

- `crates/simplant/*`
- `examples/simplant/*`

---

## Resumen de conteos

Comando base: `git status --short` filtrado (sin `crates/simplant/*` ni `examples/simplant/*`).

| Tipo | Conteo | Notas |
|---|---|---|
| **Renames totales** (`R` + `RM`) | **511** | 390 rename puro + 121 rename con contenido modificado |
| **Modificados** (`M`, sin rename) | **533** | Solo cambio de contenido, sin cambio de path |
| **Renames + contenido** (`RM`) | **121** | Subconjunto de los 511 renames |
| **Nuevos (untracked) en zona upstream** | **7 paths** | 10 archivos concretos (ver sección 3) |
| **Entradas totales en status** | **1057** | 1044 tracked + 13 untracked |
| **Magnitud del diff** | 1044 archivos, +2605 / −2296 líneas | `git diff --stat HEAD` |

---

## 1. Renames del rebranding (`rerun` → `simplant-lab`)

### 1.1 crates top-level Rust (32 paths)

**`crates/top/rerun` → `crates/top/simplant-lab`** (28 archivos):

| Origen | Destino |
|---|---|
| `crates/top/rerun/Cargo.toml` | `crates/top/simplant-lab/Cargo.toml` |
| `crates/top/rerun/README.md` | `crates/top/simplant-lab/README.md` |
| `crates/top/rerun/build.rs` | `crates/top/simplant-lab/build.rs` |
| `crates/top/rerun/src/clap.rs` | `crates/top/simplant-lab/src/clap.rs` |
| `crates/top/rerun/src/commands/analytics/mod.rs` | `crates/top/simplant-lab/src/commands/analytics/mod.rs` |
| `crates/top/rerun/src/commands/auth.rs` | `crates/top/simplant-lab/src/commands/auth.rs` |
| `crates/top/rerun/src/commands/download.rs` | `crates/top/simplant-lab/src/commands/download.rs` |
| `crates/top/rerun/src/commands/entrypoint.rs` | `crates/top/simplant-lab/src/commands/entrypoint.rs` |
| `crates/top/rerun/src/commands/mcap/info.rs` | `crates/top/simplant-lab/src/commands/mcap/info.rs` |
| `crates/top/rerun/src/commands/mcap/mod.rs` | `crates/top/simplant-lab/src/commands/mcap/mod.rs` |
| `crates/top/rerun/src/commands/mod.rs` | `crates/top/simplant-lab/src/commands/mod.rs` |
| `crates/top/rerun/src/commands/rrd/compare.rs` | `crates/top/simplant-lab/src/commands/rrd/compare.rs` |
| `crates/top/rerun/src/commands/rrd/filter.rs` | `crates/top/simplant-lab/src/commands/rrd/filter.rs` |
| `crates/top/rerun/src/commands/rrd/merge_optimize.rs` | `crates/top/simplant-lab/src/commands/rrd/merge_optimize.rs` |
| `crates/top/rerun/src/commands/rrd/migrate.rs` | `crates/top/simplant-lab/src/commands/rrd/migrate.rs` |
| `crates/top/rerun/src/commands/rrd/mod.rs` | `crates/top/simplant-lab/src/commands/rrd/mod.rs` |
| `crates/top/rerun/src/commands/rrd/print.rs` | `crates/top/simplant-lab/src/commands/rrd/print.rs` |
| `crates/top/rerun/src/commands/rrd/route.rs` | `crates/top/simplant-lab/src/commands/rrd/route.rs` |
| `crates/top/rerun/src/commands/rrd/split.rs` | `crates/top/simplant-lab/src/commands/rrd/split.rs` |
| `crates/top/rerun/src/commands/rrd/stats.rs` | `crates/top/simplant-lab/src/commands/rrd/stats.rs` |
| `crates/top/rerun/src/commands/rrd/verify.rs` | `crates/top/simplant-lab/src/commands/rrd/verify.rs` |
| `crates/top/rerun/src/commands/stdio.rs` | `crates/top/simplant-lab/src/commands/stdio.rs` |
| `crates/top/rerun/src/demo_util.rs` | `crates/top/simplant-lab/src/demo_util.rs` |
| `crates/top/rerun/src/lib.rs` | `crates/top/simplant-lab/src/lib.rs` |
| `crates/top/rerun/src/log_integration.rs` | `crates/top/simplant-lab/src/log_integration.rs` |
| `crates/top/rerun/src/native_viewer.rs` | `crates/top/simplant-lab/src/native_viewer.rs` |
| `crates/top/rerun/src/sdk.rs` | `crates/top/simplant-lab/src/sdk.rs` |
| `crates/top/rerun/tests/rerun_tests.rs` | `crates/top/simplant-lab/tests/rerun_tests.rs` |

**`crates/top/rerun-cli` → `crates/top/simplant-lab-cli`** (4 archivos):

| Origen | Destino |
|---|---|
| `crates/top/rerun-cli/Cargo.toml` | `crates/top/simplant-lab-cli/Cargo.toml` |
| `crates/top/rerun-cli/README.md` | `crates/top/simplant-lab-cli/README.md` |
| `crates/top/rerun-cli/build.rs` | `crates/top/simplant-lab-cli/build.rs` |
| `crates/top/rerun-cli/src/bin/rerun.rs` | `crates/top/simplant-lab-cli/src/bin/simplant-lab.rs` |

**Razón:** rebranding del crate de usuario y del binario CLI (`rerun` → `simplant-lab`). Los archivos marcados `RM` además actualizan strings internos (nombre del crate, clap, entrypoint, etc.).

### 1.2 Python SDK (479 paths)

**`rerun_py/rerun_sdk/rerun/` → `rerun_py/rerun_sdk/simplant_lab/`** (477 archivos):

Árbol completo del módulo Python renombrado: `__init__.py`, `__main__.py`, utilidades internas (`_*.py`), y subpaquetes generados:

- `archetypes/` (~90 pares `.py` + `_ext.py`)
- `blueprint/` (~30 archivos)
- `catalog/`, `components/`, `datatypes/`, `views/`
- `experimental/`, `lang/`, `sinks/`, `validators/`

**`rerun_py/rerun_sdk/rerun_cli/` → `rerun_py/rerun_sdk/simplant_lab_cli/`** (2 archivos):

| Origen | Destino |
|---|---|
| `rerun_py/rerun_sdk/rerun_cli/__init__.py` | `rerun_py/rerun_sdk/simplant_lab_cli/__init__.py` |
| `rerun_py/rerun_sdk/rerun_cli/__main__.py` | `rerun_py/rerun_sdk/simplant_lab_cli/__main__.py` |

**Razón:** paquete PyPI `rerun-sdk` → `simplant-lab-sdk`; módulo importable `import simplant_lab`. Los archivos `RM` en archetypes/blueprint/etc. actualizan docstrings generados que mencionaban "Rerun".

---

## 2. Archivos upstream MODIFICADOS (agrupados por razón)

### 2.1 workspace y configuración de build (rebranding + wiring)

| Archivo | Cambio |
|---|---|
| `Cargo.toml` | `authors`, `homepage`, `repository`; crate `rerun`/`rerun-cli` → `simplant-lab`/`simplant-lab-cli`; workspace members `crates/simplant/*`, `examples/simplant/*`; dep workspace `dxf = "0.6"` |
| `Cargo.lock` | Lockfile regenerado por los cambios anteriores |
| `pixi.toml` | Workspace `name`/`authors`/`homepage`; tareas `simplant-lab*`; aliases `rerun*` apuntan a `simplant-lab-cli`; `py-sync-snippets` usa `simplant-lab-sdk`; `rs-fmt` hace `cargo build -p snippets` primero |
| `pyproject.toml` | `[tool.uv.sources]` agrega `simplant-lab-sdk` |
| `README.md` | Banner, badges PyPI/crates.io, quickstart, ejemplos → SimPlant-Lab |
| `CLAUDE.md` | Referencias al proyecto |
| `.vscode/settings.json` | Ajustes del IDE |
| `.github/workflows/reusable_build_and_upload_rerun_cli.yml` | `bin_name` → `simplant-lab` / `simplant-lab.exe` en todas las plataformas |

### 2.2 spawn y bridge Python (ejecutable del viewer)

| Archivo | Cambio |
|---|---|
| `crates/top/re_sdk/src/spawn.rs` | `RERUN_BINARY` → `SIMPLANT_LAB_BINARY = "simplant-lab"` |
| `rerun_py/src/python_bridge.rs` | `OTEL_SERVICE_NAME` → `simplant-lab-py`; `executable_name` → `simplant-lab` |
| `rerun_py/pyproject.toml` | `name = "simplant-lab-sdk"`; scripts `simplant-lab` + alias `rerun`; `python-packages` incluye shims de compat |

### 2.2.1 bindings pyo3 del dominio SimPlant (`sp_python`)

Integración mínima en la zona upstream del fork: el crate puente `sp_python` (código SimPlant en
`crates/simplant/sp_python/`, fuera de este registro) se registra desde el `#[pymodule]` existente.

| Archivo | Cambio |
|---|---|
| `rerun_py/Cargo.toml` | Dep workspace `sp_python = { path = "../crates/simplant/sp_python" }` |
| `rerun_py/src/python_bridge.rs` | Tras `lenses::register(m)?;`: hook `sp_python::register_recording_stream_extractor(Box::new(\|obj\| { … }))` para extraer `RecordingStream` desde Python; luego `sp_python::register(py, m)?;` (registra submódulos `simplant_lab.kernel`, `.asset_model`, `.acquisition`, `.simulation`, `.ml_dataloop`, `.stress_testing`, `.recording`, `.types`) |

**Razón:** exponer los crates `sp_*` de dominio a Python sin tocar los crates de dominio (ADR-0002);
toda la frontera pyo3 vive en `sp_python` + estas dos líneas de wiring upstream.

### 2.2.2 vista P&ID (`sp_pid_viewer`)

Integración mínima: la vista vive en `crates/simplant/sp_pid_viewer/` (fuera de este registro) y se
registra vía el punto de extensión oficial `App::add_view_class`.

| Archivo | Cambio |
|---|---|
| `crates/top/simplant-lab/Cargo.toml` | Dep opcional `sp_pid_viewer = { path = "../../simplant/sp_pid_viewer", optional = true }`; feature `native_viewer` agrega `"dep:sp_pid_viewer"` |
| `crates/top/simplant-lab/src/commands/entrypoint.rs` | Tras construir la `App` (bloque `--memory-limit`): `app.add_view_class::<sp_pid_viewer::PidView>()` con log de error |
| `clippy.toml` + `scripts/clippy_wasm/clippy.toml` | `doc-valid-idents` agrega `"SimPlant"` (evita falsos `clippy::doc_markdown` en docs de crates `sp_*`) |

**Razón:** el archetype `simplant.archetypes.PidSymbol` vive en `sp_types` (ADR-0003) y la vista en
`sp_pid_viewer`; el único wiring upstream son estas dos líneas del binario nativo.

### 2.3 Viewer — branding centralizado y strings de UI (36 archivos en `crates/`)

**Nuevo módulo referenciado** (archivo untracked, ver §3): `crates/viewer/re_ui/src/branding.rs` con `PRODUCT_NAME`, `PRODUCT_NAME_LOWERCASE`, URLs.

| Archivo | Cambio |
|---|---|
| `crates/viewer/re_ui/src/lib.rs` | Expone `pub mod branding`; docstring del crate |
| `crates/viewer/re_ui/src/icons.rs` | `RERUN_WORDMARK` → `PRODUCT_WORDMARK` (apunta a `simplant_lab_wordmark.svg`); alias deprecated |
| `crates/viewer/re_ui/src/command.rs` | Strings de comandos |
| `crates/viewer/re_ui/src/combo_item.rs` | Strings de UI |
| `crates/viewer/re_ui/examples/re_ui_example/main.rs` | Ejemplo actualizado |
| `crates/viewer/re_viewer/src/app.rs` | Docstrings y notificaciones ("native SimPlant-Lab") |
| `crates/viewer/re_viewer/src/lib.rs` | Docstrings del crate |
| `crates/viewer/re_viewer/src/native.rs` | Título de ventana / metadatos |
| `crates/viewer/re_viewer/src/startup_options.rs` | Opciones de arranque |
| `crates/viewer/re_viewer/src/ui/rerun_menu.rs` | Usa `PRODUCT_WORDMARK` |
| `crates/viewer/re_viewer/src/ui/top_panel.rs` | Strings del panel superior |
| `crates/viewer/re_viewer/src/ui/settings_screen.rs` | Pantalla de ajustes |
| `crates/viewer/re_viewer/src/ui/memory_panel/mod.rs` | Panel de memoria |
| `crates/viewer/re_viewer/src/ui/welcome_screen/intro_section.rs` | Welcome screen |
| `crates/viewer/re_viewer/src/ui/welcome_screen/welcome_section.rs` | Welcome screen |
| `crates/viewer/re_viewer/src/viewer_analytics/mod.rs` | Analytics |
| `crates/viewer/re_viewer_context/src/app_options.rs` | Cache dir `io/simplant-lab/SimPlant-Lab`; rename campo `include_simplant_lab_examples_button_in_recordings_panel` (con `serde` alias del nombre viejo) |
| `crates/viewer/re_viewer_context/src/app_context.rs` | Contexto de app |
| `crates/viewer/re_viewer_context/src/command_sender.rs` | Comandos |
| `crates/viewer/re_viewer_context/src/lib.rs` | Docstrings |
| `crates/viewer/re_blueprint_tree/src/blueprint_tree.rs` | Strings de UI |
| `crates/viewer/re_data_ui/src/lib.rs` | Strings de UI |
| `crates/viewer/re_data_ui/src/video_ui.rs` | Strings de UI |
| `crates/viewer/re_recording_panel/src/data.rs` | Panel de grabaciones |
| `crates/viewer/re_recording_panel/src/recording_panel_ui.rs` | Panel de grabaciones |
| `crates/viewer/re_redap_browser/src/server_modal.rs` | Modal de servidor |
| `crates/viewer/re_selection_panel/src/defaults_ui.rs` | Panel de selección |
| `crates/viewer/re_web_viewer_server/web_viewer/index.html` | `<title>`, manifest |
| `crates/viewer/re_web_viewer_server/web_viewer/manifest.json` | Nombre de la PWA |
| `crates/utils/re_auth/src/status_page.html` | `<title>` → SimPlant-Lab |

### 2.4 codegen — docstrings en plantillas (3 archivos)

| Archivo | Cambio |
|---|---|
| `crates/build/re_types_builder/src/codegen/rust/api.rs` | Docstring "into Rerun" → "into SimPlant-Lab" |
| `crates/build/re_types_builder/src/codegen/python/mod.rs` | Idem en plantilla Python |
| `crates/build/re_types_builder/src/codegen/cpp/mod.rs` | Idem en plantilla C++ |

**Razón:** los docstrings generados para `send_columns` reflejan el nuevo nombre de producto. **No** se cambió el namespace FlatBuffers `rerun` ni los tipos wire.

### 2.5 importer DXF — extensión de dominio oil & gas (2 archivos modificados + nuevos en §3)

| Archivo | Cambio |
|---|---|
| `crates/store/re_importer/Cargo.toml` | Dep `dxf.workspace = true` |
| `crates/store/re_importer/src/lib.rs` | Registra `DxfImporter` en `BUILTIN_IMPORTERS`; extensión `dxf` en `SUPPORTED_THIRD_PARTY_FORMATS` |

**Razón:** primer importer de dominio (planos CAD `.dxf`) usando el punto de extensión oficial `re_importer::Importer`.

### 2.6 docs snippets (311 archivos — `docs/snippets/`)

| Patrón | Archivos |
|---|---|
| `docs/snippets/Cargo.toml` | Dep del crate `simplant-lab` en lugar de `rerun` |
| `docs/snippets/all/**/*.py` (~155) | `import rerun as rr` → `import simplant_lab as rr` |
| `docs/snippets/all/**/*.rs` (~155) | `use rerun::` → `use simplant_lab::` |
| `docs/snippets/all/**/CMakeLists.txt`, `build.rs`, etc. | Referencias al crate renombrado |

**Razón:** snippets de documentación alineados al nuevo import Python y crate Rust.

### 2.7 ejemplos (125 archivos)

| Área | Conteo | Cambio típico |
|---|---|---|
| `examples/python/**` | 59 | `import simplant_lab as rr`; comentarios de pip |
| `examples/rust/**` | 66 | `use simplant_lab::`; `Cargo.toml` dep `simplant-lab` |

### 2.8 tests (49 archivos)

| Área | Conteo | Cambio típico |
|---|---|---|
| `tests/python/**` | 27 | Imports y referencias al SDK renombrado |
| `tests/rust/**` | 22 | Crate `simplant-lab` en `Cargo.toml` y fuentes |

### 2.9 JavaScript viewer wrappers (2 archivos)

| Archivo | Cambio |
|---|---|
| `rerun_js/web-viewer/package.json` | Metadatos del paquete |
| `rerun_js/web-viewer-react/package.json` | Metadatos del paquete |

### 2.10 API de blueprint en Rust — `DataframeView` y `TextLogView` (extensión funcional)

| Archivo | Cambio |
|---|---|
| `crates/top/re_sdk/src/blueprint/view.rs` | Nuevos `DataframeView` (con `with_timeline`, vía archetype `DataframeQuery`) y `TextLogView`, espejando el patrón de los views existentes; tests de construcción |
| `crates/top/re_sdk/src/blueprint/mod.rs` | Exporta `DataframeView` y `TextLogView` |
| `crates/top/re_sdk/src/blueprint/container.rs` | `impl From<…>` de ambos para `ContainerLike` |

**Razón:** la API de blueprint de Rust upstream expone `TimeSeries`/`Spatial2D`/`Spatial3D`/`Map`/`Graph`/`TextDocument` pero **no** `Dataframe` (tabla) ni `TextLog` — en upstream esos views solo se construyen desde la API de Python. El demo `tanque_demo` los necesita para abrir con una tabla de variables de proceso indexada por `plant_time`. La extensión replica el patrón existente (`class_identifier` + builders `with_*`) y es **candidata a PR upstream**. No toca tipos wire ni el namespace FlatBuffers.

### 2.11 API de blueprint en Rust — `CustomView` (vistas de clase arbitraria)

| Archivo | Cambio |
|---|---|
| `crates/top/re_sdk/src/blueprint/view.rs` | Nuevo `CustomView`, con `class_identifier` provisto por el caller (en vez de hardcodeado) más `with_origin`/`with_contents`/`with_visible`/`with_defaults`/`with_override`/`with_overrides`, espejando el patrón de `TimeSeriesView`; test de construcción |
| `crates/top/re_sdk/src/blueprint/mod.rs` | Exporta `CustomView` |
| `crates/top/re_sdk/src/blueprint/container.rs` | `impl From<CustomView> for ContainerLike` |

**Razón:** todos los views tipados de la API de blueprint (§2.10 incluida) hardcodean su `class_identifier`, así que ninguno puede dirigirse a una clase de vista registrada por la app embebedora vía `App::add_view_class` — como la vista P&ID del fork (`sp_pid_viewer::PidView`, identifier `"SimPlantPid"`). `CustomView` cierra ese hueco recibiendo el identifier como parámetro, sin tocar `api.rs` (el `class_identifier` ya fluía genérico hacia `ViewBlueprint::new(ViewClass(…))`). Usado por `examples/simplant/pid_canvas_demo` para componer un layout 50/50 (P&ID a la izquierda, tendencias a la derecha). Mismo patrón que §2.10, misma candidatura a PR upstream. No toca tipos wire ni el namespace FlatBuffers.

---

## 3. Archivos NUEVOS en zona upstream (untracked)

| Path | Archivos | Razón |
|---|---|---|
| `crates/viewer/re_ui/src/branding.rs` | 1 | Constantes centralizadas de marca (`PRODUCT_NAME`, URLs, etc.) |
| `crates/viewer/re_ui/data/icons/simplant_lab_wordmark.svg` | 1 | Wordmark del producto |
| `crates/viewer/re_ui/data/icons/simplant_lab_wordmark_test.png` | 1 | Referencia visual para tests de snapshot |
| `crates/store/re_importer/src/importer_dxf/mod.rs` | 1 | Módulo DXF importer |
| `crates/store/re_importer/src/importer_dxf/domain.rs` | 1 | Tipos de dominio DXF |
| `crates/store/re_importer/src/importer_dxf/parse.rs` | 1 | Parser DXF |
| `crates/store/re_importer/src/importer_dxf/emit.rs` | 1 | Emisión a archetypes Rerun |
| `crates/store/re_importer/tests/data/dxf/sample.dxf` | 1 | Fixture de test |
| `rerun_py/rerun_sdk/rerun/__init__.py` | 1 | Shim de compat: `from simplant_lab import *` + `DeprecationWarning` |
| `rerun_py/rerun_sdk/rerun_cli/__init__.py` | 1 | Shim CLI de compat |

**Total:** 7 paths untracked, 10 archivos.

---

## 4. Cambios deliberadamente NO aplicados (importante para merge)

Según [`MIGRATION_PLAN.md`](MIGRATION_PLAN.md) §2.3, estos elementos upstream **permanecen sin cambiar** para preservar compatibilidad:

| Elemento | Estado |
|---|---|
| Namespace FlatBuffers `rerun` en `definitions/` | Sin cambio |
| Namespace C++ `rerun::` | Sin cambio |
| Extensión de archivo `.rrd` | Sin cambio |
| Módulo PyO3 `rerun_bindings` | Sin cambio |
| Crate `rerun_c` (C bindings) | Sin renombrar |
| Tipos wire / protocolo gRPC | Sin cambio |

---

## 5. Notas para futuros merges upstream

### 5.1 estrategia general

1. **Rebase/merge upstream** sobre la rama de trabajo.
2. **Resolver conflictos por capas**, en este orden:
   - Renames (`rerun` → `simplant-lab`): usar `git log --follow` y este documento como mapa.
   - Branding: buscar strings `"Rerun"`, `rerun_wordmark`, `RERUN_WORDMARK`, `rerun-cli`, `rerun-sdk`.
   - Extensión DXF: preservar `importer_dxf/` y el registro en `lib.rs` / `Cargo.toml`.
   - Snippets/ejemplos/tests: re-aplicar sustitución masiva `rerun` → `simplant_lab` / `simplant-lab` si upstream los regeneró.
3. **No tocar** `crates/simplant/*` durante el merge de upstream salvo conflictos de workspace (`Cargo.toml` members).
4. **Regenerar** tras merge si hubo cambios en `.fbs`: `pixi run codegen` (los docstrings de branding en codegen habrá que re-aplicar si upstream modificó las plantillas).

### 5.2 puntos de alto riesgo de conflicto

| Área | Por qué |
|---|---|
| `crates/top/simplant-lab/` (ex `rerun/`) | Crate renombrado; upstream seguirá editando `crates/top/rerun/` |
| `crates/viewer/re_viewer/` | Muchos strings de UI; upstream mejora el viewer frecuentemente |
| `pixi.toml` / `Cargo.toml` | Workspace root; casi siempre conflictúa |
| `docs/snippets/all/` | Cientos de archivos; upstream los regenera con `codegen` |
| `rerun_py/rerun_sdk/simplant_lab/` | Árbol Python generado + renombrado |

### 5.3 comandos útiles para auditar drift

```bash
# Conteos actuales (excluyendo zona SimPlant)
git status --short | grep -v 'crates/simplant/' | grep -v 'examples/simplant/' | wc -l

# Renames pendientes
git status --short | grep -E '^R|^RM' | grep -v 'crates/simplant/'

# Buscar strings de branding residuales en zona upstream
rg -l 'Rerun Viewer|rerun-sdk|rerun-cli' --glob '!crates/simplant/**' --glob '!examples/simplant/**'

# Diff contra upstream remoto (cuando esté configurado)
git diff upstream/main --stat -- ':!crates/simplant' ':!examples/simplant'
```

### 5.4 checklist post-merge

- [ ] `pixi run simplant-lab-build` compila
- [ ] `pixi run py-build` instala `simplant-lab-sdk`
- [ ] `cargo nextest run --all-features -p re_importer` (incluye DXF)
- [ ] Viewer muestra wordmark SimPlant-Lab
- [ ] `import simplant_lab` funciona; `import rerun` emite `DeprecationWarning`
- [ ] Actualizar este documento con nuevos cambios

---

*Última actualización: 2026-07-08 — agregado §2.11 (`CustomView` en la API de blueprint de Rust); resto generado desde el working tree de `feat/simplant-domain-crates`.*
