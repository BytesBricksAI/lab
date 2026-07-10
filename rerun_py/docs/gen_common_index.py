#!/usr/bin/env python3
"""
Generate API reference pages and a landing index for the SimPlant Lab Python SDK.

The script emits two kinds of output (at the docs root):

1. **Track A — auto-generated per-package pages** (`<slug>.md`).
   Each entry in `DOCUMENTED_PACKAGES` gets one page that renders every
   public symbol of that package. Public symbols are determined by `griffe`
   with the `griffe-public-redundant-aliases` extension installed (see
   `mkdocs.yml`), which honors three signals: `__all__`, `from x import Foo
   as Foo` redundant aliases, and in-file non-underscore definitions.

2. **Track B — curated overlay** (tables on `index.md`).
   `CURATED_GROUPS` defines themed tables on the landing page only — they
   never gate coverage. Missing curation only affects the landing page.

A pre-emission validator fails the build if any new subpackage/module
appears under `rerun_sdk/simplant_lab/` without being either documented or
explicitly excluded, if a documented or excluded path no longer exists
on disk, if a documented package's public surface is empty or fully
excluded, or if a curated table references an unknown symbol.

NOTE: When changing anything in this file, also consider how it affects
`crates/build/re_dev_tools/src/build_search_index/ingest/python.rs`.
"""

from __future__ import annotations

import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Final

import griffe
import mkdocs_gen_files

# Packages that get an auto-generated `<slug>.md` page at the docs root.
# Maps each dotted package path to its nav title path: a 1-tuple for a
# top-level nav entry, or a 2-tuple `(parent, child)` for a nested entry
# (used by the Blueprint sub-packages and `experimental.dataloader`).
# To document a brand-new subpackage, add a row here. Iteration order
# determines nav order in the rendered sidebar.
DOCUMENTED_PACKAGES: Final[dict[str, tuple[str, ...]]] = {
    "simplant_lab": ("Core",),
    "simplant_lab.archetypes": ("Archetypes",),
    "simplant_lab.components": ("Components",),
    "simplant_lab.datatypes": ("Datatypes",),
    "simplant_lab.blueprint": ("Blueprint", "APIs"),
    "simplant_lab.blueprint.archetypes": ("Blueprint", "Archetypes"),
    "simplant_lab.blueprint.components": ("Blueprint", "Components"),
    "simplant_lab.blueprint.datatypes": ("Blueprint", "Datatypes"),
    "simplant_lab.blueprint.views": ("Blueprint", "Views"),
    "simplant_lab.catalog": ("Catalog",),
    "simplant_lab.experimental": ("Experimental",),
    "simplant_lab.experimental.dataloader": ("Experimental", "Dataloader"),
    "simplant_lab.rrd_recording": ("Recording",),
    "simplant_lab.server": ("Server",),
    "simplant_lab.urdf": ("URDF Support",),
    "simplant_lab.notebook": ("Notebook",),
    "simplant_lab.auth": ("Authentication",),
    "simplant_lab.utilities": ("Utilities",),
}

# Subpackages/modules under `simplant_lab.` that deliberately do NOT get a Track A
# page. Their public symbols surface elsewhere (typically re-exported flat
# into top-level `simplant_lab`). The freshness check (bottom of file) requires
# every non-underscore subpackage/module under `rerun_sdk/simplant_lab/` to appear
# either here or in `DOCUMENTED_PACKAGES`, which makes it impossible to add
# a new submodule and silently miss it.
EXCLUDED_FROM_TRACK_A: Final[set[str]] = {
    # Single-file modules whose public symbols are re-exported flat into
    # `simplant_lab` and surface on the `simplant_lab` page. Listing them as their own
    # Track A page would just duplicate already-documented content.
    "simplant_lab.any_batch_value",
    "simplant_lab.any_value",
    "simplant_lab.dynamic_archetype",
    "simplant_lab.error_utils",
    "simplant_lab.recording_stream",
    "simplant_lab.sinks",
    "simplant_lab.time",
    "simplant_lab.web",
    # SimPlant domain modules (Rust-backed stubs); exposed as `simplant_lab.<name>`.
    "simplant_lab.acquisition",
    "simplant_lab.asset_model",
    "simplant_lab.kernel",
    "simplant_lab.ml_dataloop",
    "simplant_lab.recording",
    "simplant_lab.simulation",
    "simplant_lab.stress_testing",
    "simplant_lab.types",
    # Internal organization for blueprint code; only exposes
    # `Visualizer`/`VisualizableArchetype` which are implementation contracts,
    # not user-facing API.
    "simplant_lab.blueprint.visualizers",
    # Namespace-only packages with empty `__init__.py`; users import
    # deeper symbols (e.g. `from simplant_lab.utilities.datafusion.collect import ...`).
    # No aggregated surface to document at the namespace level.
    "simplant_lab.utilities.datafusion",
    "simplant_lab.utilities.datafusion.functions",
}

# Per-package, per-symbol allow-list of public symbols that should NOT be
# documented. Each entry must carry a comment explaining why.
EXPLICIT_DOC_EXCLUDES: Final[dict[str, set[str]]] = {
    "simplant_lab": {
        # Internal arrow-IPC constants used by send_dataframe; not user-facing.
        "RECORDING_PROPERTIES_PATH",
        "RERUN_KIND",
        "RERUN_KIND_CONTROL",
        "RERUN_KIND_INDEX",
        "SORBET_ARCHETYPE_NAME",
        "SORBET_COMPONENT",
        "SORBET_COMPONENT_TYPE",
        "SORBET_ENTITY_PATH",
        "SORBET_INDEX_NAME",
        "SORBET_IS_TABLE_INDEX",
        # Per-developer profiling; opt-in via env var.
        "tracing_session",
        # Internal numpy compat shim re-exported for use within rerun_py.
        "asarray",
    },
}


@dataclass
class Group:
    """A curated themed table rendered on the landing page only."""

    title: str
    items: list[str]


# Curated overlay: themed tables shown on the landing `index.md`. These never
# gate coverage — the auto-generated per-package pages are the source of
# truth. Items are dotted relative paths into the `simplant_lab` package
# (e.g., `init`, `archetypes.Points3D`, `experimental.send_chunk`).
CURATED_GROUPS: Final[list[Group]] = [
    Group(
        title="Initialization functions",
        items=[
            "init",
            "set_sinks",
            "connect_grpc",
            "disconnect",
            "save",
            "send_blueprint",
            "serve_grpc",
            "serve_web_viewer",
            "spawn",
            "memory_recording",
            "notebook_show",
            "legacy_notebook_show",
            "ChunkBatcherConfig",
            "DescribedComponentBatch",
            "RecordingStream",
            "TimeColumnLike",
        ],
    ),
    Group(
        title="Logging functions",
        items=["log", "log_file_from_path", "log_file_from_contents"],
    ),
    Group(
        title="Property functions",
        items=["send_property", "send_recording_name", "send_recording_start_time_nanos"],
    ),
    Group(
        title="Timeline functions",
        items=["set_time", "disable_timeline", "reset_time"],
    ),
    Group(
        title="Columnar API",
        items=["send_columns", "send_record_batch", "send_dataframe", "TimeColumn"],
    ),
    Group(
        title="General",
        items=[
            "archetypes.Clear",
            "blueprint.archetypes.EntityBehavior",
            "archetypes.RecordingInfo",
        ],
    ),
    Group(
        title="Annotations",
        items=[
            "archetypes.AnnotationContext",
            "datatypes.AnnotationInfo",
            "datatypes.ClassDescription",
        ],
    ),
    Group(
        title="Images",
        items=[
            "archetypes.DepthImage",
            "archetypes.Image",
            "archetypes.EncodedImage",
            "archetypes.EncodedDepthImage",
            "archetypes.SegmentationImage",
        ],
    ),
    Group(
        title="Video",
        items=[
            "archetypes.VideoStream",
            "archetypes.AssetVideo",
            "archetypes.VideoFrameReference",
        ],
    ),
    Group(
        title="Plotting",
        items=[
            "archetypes.BarChart",
            "archetypes.Scalars",
            "archetypes.SeriesLines",
            "archetypes.SeriesPoints",
        ],
    ),
    Group(
        title="Spatial Archetypes",
        items=[
            "archetypes.Arrows3D",
            "archetypes.Arrows2D",
            "archetypes.Asset3D",
            "archetypes.Boxes2D",
            "archetypes.Boxes3D",
            "archetypes.Capsules3D",
            "archetypes.Cylinders3D",
            "archetypes.Ellipses2D",
            "archetypes.Ellipsoids3D",
            "archetypes.GridMap",
            "archetypes.LineStrips2D",
            "archetypes.LineStrips3D",
            "archetypes.Mesh3D",
            "archetypes.Points2D",
            "archetypes.Points3D",
            "archetypes.TransformAxes3D",
        ],
    ),
    Group(
        title="Geospatial Archetypes",
        items=["archetypes.GeoLineStrings", "archetypes.GeoPoints"],
    ),
    Group(
        title="Graphs",
        items=["archetypes.GraphNodes", "archetypes.GraphEdges"],
    ),
    Group(
        title="Tensors",
        items=["archetypes.Tensor"],
    ),
    Group(
        title="Text",
        items=["LoggingHandler", "archetypes.TextDocument", "archetypes.TextLog"],
    ),
    Group(
        title="State timeline",
        items=["archetypes.StateChange", "archetypes.StateConfiguration"],
    ),
    Group(
        title="Transforms and Coordinate Systems",
        items=[
            "archetypes.Pinhole",
            "archetypes.Transform3D",
            "archetypes.InstancePoses3D",
            "archetypes.ViewCoordinates",
            "components.Scale3D",
            "datatypes.Quaternion",
            "datatypes.RotationAxisAngle",
            "archetypes.CoordinateFrame",
        ],
    ),
    Group(
        title="MCAP",
        items=[
            "archetypes.McapChannel",
            "archetypes.McapMessage",
            "archetypes.McapSchema",
            "archetypes.McapStatistics",
        ],
    ),
    Group(
        title="Interfaces",
        items=[
            "ComponentMixin",
            "ComponentBatchLike",
            "AsComponents",
            "ComponentColumn",
        ],
    ),
    Group(
        title="Script Helpers",
        items=["script_add_args", "script_setup", "script_teardown"],
    ),
    Group(
        title="Other classes and functions",
        items=[
            "get_data_recording",
            "get_global_data_recording",
            "get_recording_id",
            "get_thread_local_data_recording",
            "is_enabled",
            "set_global_data_recording",
            "set_thread_local_data_recording",
            "start_web_viewer_server",
            "escape_entity_path_part",
            "new_entity_path",
            "thread_local_stream",
            "recording_stream_generator_ctx",
            "MemoryRecording",
            "BinaryStream",
            "GrpcSink",
            "FileSink",
        ],
    ),
]


def public_surface(pkg: griffe.Module) -> set[str]:
    """
    Return the set of names that `griffe.is_public` considers public.

    Relies on the `griffe-public-redundant-aliases` extension to honor the
    `from x import Foo as Foo` convention; combined with griffe's built-in
    `__all__` handling and underscore-name filtering, this matches the
    rerun codebase's public-API conventions.
    """
    return {name for name, member in pkg.members.items() if member.is_public and not name.startswith("_")}


# ---------------------------------------------------------------------------
# Setup griffe loader and resolve documented packages.

rerun_py_root = Path(__file__).parent.parent.resolve()
sdk_root = Path(__file__).parent.parent.joinpath("rerun_sdk").resolve()
out_dir = Path()  # generated pages live at the docs root

search_paths = [path for path in sys.path if path]
search_paths.insert(0, rerun_py_root.as_posix())
search_paths.insert(0, sdk_root.as_posix())

# Load the same extension that mkdocs.yml configures for mkdocstrings, so this
# script and the rendered docs agree on what counts as a public symbol.
extensions = griffe.load_extensions("griffe_public_redundant_aliases")
loader = griffe.GriffeLoader(search_paths=search_paths, extensions=extensions)
bindings_pkg = loader.load("rerun_bindings", find_stubs_package=True)
simplant_lab_pkg = loader.load("simplant_lab")


def griffe_module_for(pkg: str) -> griffe.Module:
    """Return the griffe Module for a `DOCUMENTED_PACKAGES` entry."""
    if pkg == "simplant_lab":
        return simplant_lab_pkg
    assert pkg.startswith("simplant_lab.")
    return simplant_lab_pkg[pkg[len("simplant_lab.") :]]


def _is_package_dir(entry: Path) -> bool:
    """Return True when `entry` is a package directory (runtime or stub-only)."""
    return entry.is_dir() and ((entry / "__init__.py").exists() or (entry / "__init__.pyi").exists())


def discover_subpackages_and_modules() -> set[str]:
    """
    Return the dotted paths of every public subpackage/top-level module in `rerun_sdk/simplant_lab/`.

    Includes every non-underscore subpackage at any depth (a directory with
    `__init__.py` or stub-only `__init__.pyi`), and every non-underscore
    single-file module at the top level only (e.g., `simplant_lab.notebook`,
    `simplant_lab.server`).

    Single-file `.py` / `.pyi` modules nested *inside* subpackages are treated as
    implementation detail and skipped — these are typically codegen output
    (e.g., `simplant_lab.archetypes.points3d` backing `simplant_lab.archetypes.Points3D`)
    that users are not expected to import directly.
    """
    base = sdk_root.joinpath("simplant_lab")
    found = {"simplant_lab"}

    for entry in base.iterdir():
        if entry.name.startswith("_") or entry.name.startswith("."):
            continue
        if _is_package_dir(entry):
            found.add(f"simplant_lab.{entry.name}")
            _walk_nested_subpackages(entry, f"simplant_lab.{entry.name}", found)
        elif entry.is_file() and entry.suffix in {".py", ".pyi"} and entry.stem != "__init__":
            found.add(f"simplant_lab.{entry.stem}")

    return found


def _walk_nested_subpackages(pkg_dir: Path, dotted: str, found: set[str]) -> None:
    """Recurse into `pkg_dir`, collecting nested subpackages (dirs with `__init__.py` or `__init__.pyi`)."""
    for entry in pkg_dir.iterdir():
        if entry.name.startswith("_") or entry.name.startswith("."):
            continue
        if _is_package_dir(entry):
            child = f"{dotted}.{entry.name}"
            found.add(child)
            _walk_nested_subpackages(entry, child, found)


# ---------------------------------------------------------------------------
# Pre-emission validator: fail loud on stale config or new modules before
# any output is written, with friendlier messages than a raw KeyError mid-render.


def validate_config() -> None:
    """
    Fail the build if any docs config has gone stale.

    Together these checks make it impossible to add (or rename, or remove) a
    submodule without docs noticing.
    """
    discovered = discover_subpackages_and_modules()
    documented = set(DOCUMENTED_PACKAGES)

    stale = documented - discovered
    if stale:
        raise SystemExit(
            f"DOCUMENTED_PACKAGES references modules that no longer exist on disk: "
            f"{sorted(stale)}. Remove them from DOCUMENTED_PACKAGES.",
        )

    stale = EXCLUDED_FROM_TRACK_A - discovered
    if stale:
        raise SystemExit(
            f"EXCLUDED_FROM_TRACK_A references modules that no longer exist on disk: "
            f"{sorted(stale)}. Remove them from EXCLUDED_FROM_TRACK_A.",
        )

    unaccounted = discovered - documented - EXCLUDED_FROM_TRACK_A - {"simplant_lab"}
    if unaccounted:
        raise SystemExit(
            f"New subpackages/modules under `simplant_lab.` are neither documented nor "
            f"excluded: {sorted(unaccounted)}.\n"
            f"  - Add a row to DOCUMENTED_PACKAGES to give each its own Track A page, OR\n"
            f"  - Add to EXCLUDED_FROM_TRACK_A with an inline comment if its public\n"
            f"    symbols are re-exported elsewhere (typically flat into `simplant_lab`).",
        )

    for pkg in DOCUMENTED_PACKAGES:
        expected = public_surface(griffe_module_for(pkg))
        excludes = EXPLICIT_DOC_EXCLUDES.get(pkg, set())
        if not expected:
            raise SystemExit(
                f"`{pkg}` is in DOCUMENTED_PACKAGES but griffe sees no public symbols. "
                f"Either add `__all__`, add public re-exports, or remove `{pkg}` from "
                f"DOCUMENTED_PACKAGES.",
            )
        if not (expected - excludes):
            raise SystemExit(
                f"All public symbols of `{pkg}` are in EXPLICIT_DOC_EXCLUDES; "
                f"remove `{pkg}` from DOCUMENTED_PACKAGES or trim the excludes.",
            )

    for group in CURATED_GROUPS:
        for item in group.items:
            try:
                _ = simplant_lab_pkg[item]
            except KeyError:
                raise SystemExit(
                    f"Curated table '{group.title}' references unknown symbol '{item}'.",
                ) from None


validate_config()


# ---------------------------------------------------------------------------
# Track A: emit per-package pages.

nav = mkdocs_gen_files.Nav()
nav[("Overview",)] = "index.md"


def slug_for(pkg: str) -> str:
    # The codegen in `re_types_builder` writes Python doc URLs as
    # `ref.rerun.io/docs/python/stable/<subpackage>` (e.g. `/archetypes`,
    # `/blueprint_views`) — i.e. without a leading `rerun_`. Match that here
    # so the autogenerated links in `docs/content/reference/types/**` resolve.
    if pkg == "simplant_lab":
        return "simplantLab.md"
    return pkg.removeprefix("simplant_lab.").replace(".", "_") + ".md"


for pkg, nav_path in DOCUMENTED_PACKAGES.items():
    excludes = EXPLICIT_DOC_EXCLUDES.get(pkg, set())
    members = sorted(public_surface(griffe_module_for(pkg)) - excludes)

    md_file = slug_for(pkg)
    nav[nav_path] = md_file

    write_path = out_dir.joinpath(md_file)
    with mkdocs_gen_files.open(write_path, "w") as fd:
        fd.write(f"::: {pkg}\n")
        fd.write("    options:\n")
        fd.write("      show_root_heading: True\n")
        fd.write("      heading_level: 3\n")
        fd.write("      members_order: alphabetical\n")
        fd.write("      members:\n")
        for name in members:
            fd.write(f"        - {name}\n")


# ---------------------------------------------------------------------------
# Track B: emit landing page with static prefix, curated tables, static suffix.

index_path = out_dir.joinpath("index.md")


def docstring_first_line(item: str) -> str:
    """Return the first line of `simplant_lab.<item>`'s docstring, with bindings fallback."""
    obj = simplant_lab_pkg[item]
    if "rerun_bindings" in obj.canonical_path:
        # The class is defined in the maturin extension; griffe sees the stub.
        # Get the docstring from the bindings package instead.
        obj = bindings_pkg[obj.canonical_path[len("rerun_bindings.") :]]
    if obj.docstring is None:
        raise SystemExit(f"No docstring for `simplant_lab.{item}` (referenced from a curated table).")
    return obj.docstring.lines[0]


def display_name(item: str) -> str:
    """
    Compute the rendered name for a curated-table entry.

    Strip `archetypes.` / `components.` / `datatypes.` prefixes when the
    symbol is also flat-re-exported into top-level `simplant_lab`, so the table
    shows `simplant_lab.Points3D` rather than `simplant_lab.archetypes.Points3D`.
    """
    for prefix in ("archetypes.", "components.", "datatypes."):
        stripped = item.removeprefix(prefix)
        if stripped != item and stripped in simplant_lab_pkg.members:
            return f"simplant_lab.{stripped}"
    return f"simplant_lab.{item}"


with mkdocs_gen_files.open(index_path, "w") as index_file:
    index_file.write(
        """
## Getting Started
* [Quick start](https://www.rerun.io/docs/getting-started/data-in/python)
* [Tutorial](https://www.rerun.io/docs/getting-started/data-in/python)
* [Examples on GitHub](https://github.com/rerun-io/rerun/tree/latest/examples/python)
* [Troubleshooting](https://www.rerun.io/docs/overview/installing-rerun/troubleshooting)

There are many different ways of sending data to the Rerun Viewer depending on what you're trying
to achieve and whether the viewer is running in the same process as your code, in another process,
or even as a separate web application.

Checkout [SDK Operating Modes](https://www.rerun.io/docs/reference/sdk/operating-modes) for an
overview of what's possible and how.

## Supported Python Versions

Rerun will typically support Python version up until their end-of-life. If you are using an older version
of Python, you can use the table below to make sure you choose the proper Rerun version for your Python installation.

| **Rerun Version** | **Release Date** | **Supported Python Version** |
|-------------------|------------------|------------------------------|
| 0.32              | May. 13, 2026    | 3.10+                        |
| 0.31              | Mar. 31, 2026    | 3.10+                        |
| 0.30              | Feb. 25, 2026    | 3.10+                        |
| 0.29              | Jan. 30, 2026    | 3.10+                        |
| 0.28              | Dec. 18, 2025    | 3.10+                        |
| 0.27              | Nov. 10, 2025    | 3.10+                        |
| 0.26              | Oct. 13, 2025    | 3.9+                         |
| 0.25              | Sep. 16, 2025    | 3.9+                         |
| 0.24              | Jul. 17, 2025    | 3.9+                         |
| 0.23              | Apr. 24, 2025    | 3.9+                         |
| 0.22              | Feb.  6, 2025    | 3.9+                         |
| 0.21              | Dec. 18. 2024    | 3.9+                         |
| 0.20              | Nov. 14, 2024    | 3.9+                         |
| 0.19              | Oct. 17, 2024    | 3.8+                         |


## APIs
""",
    )

    for group in CURATED_GROUPS:
        index_file.write(f"### {group.title}\n")

        # `is_function` follows alias chains, so this works for redundant
        # aliases as well as in-file definitions.
        funcs = [item for item in group.items if simplant_lab_pkg[item].is_function]
        classes = [item for item in group.items if not simplant_lab_pkg[item].is_function]

        if funcs:
            index_file.write("Function | Description\n")
            index_file.write("-------- | -----------\n")
            for item in funcs:
                index_file.write(
                    f"[`{display_name(item)}()`][simplant_lab.{item}] | {docstring_first_line(item)}\n",
                )
            index_file.write("\n")

        if classes:
            index_file.write("Class | Description\n")
            index_file.write("-------- | -----------\n")
            for item in classes:
                index_file.write(
                    f"[`{display_name(item)}`][simplant_lab.{item}] | {docstring_first_line(item)}\n",
                )
            index_file.write("\n")

    index_file.write(
        """
# Troubleshooting
You can set `RUST_LOG=debug` before running your Python script
and/or `rerun` process to get some verbose logging output.

If you run into any issues don't hesitate to [open a ticket](https://github.com/rerun-io/rerun/issues/new/choose)
or [join our Discord](https://discord.gg/Gcm8BbTaAj).
""",
    )


# Generate the SUMMARY.txt file
with mkdocs_gen_files.open(out_dir.joinpath("SUMMARY.txt"), "w") as nav_file:
    nav_file.writelines(nav.build_literate_nav())
