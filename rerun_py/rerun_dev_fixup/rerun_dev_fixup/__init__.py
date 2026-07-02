"""Development helper for simplant-lab-sdk editable installs."""

from __future__ import annotations

import os
import sys
from pathlib import Path


def _find_repo_root() -> Path | None:
    """Find the rerun repo root directory."""
    # Try PIXI_PROJECT_ROOT first (set when running under pixi)
    pixi_root = os.environ.get("PIXI_PROJECT_ROOT")
    if pixi_root:
        return Path(pixi_root)

    # Try to find repo root from the venv location using sys.prefix
    # sys.prefix points to the venv root (e.g., /path/to/repo/.venv)
    venv_path = Path(sys.prefix)
    if venv_path.name == ".venv":
        return venv_path.parent

    return None


def init() -> None:
    """
    Sitecustomize entrypoint that sets RERUN_CLI_PATH.

    This runs early during Python startup (before .pth files),
    ensuring the env var is set before simplant_lab is imported.
    """
    # Don't override if already set
    if "RERUN_CLI_PATH" in os.environ:
        return

    repo_root = _find_repo_root()
    if repo_root is None:
        return

    cargo_target = Path(os.environ.get("CARGO_TARGET_DIR", repo_root / "target"))
    for name in ("simplant-lab", "rerun"):
        cli_path = cargo_target / "debug" / name
        if cli_path.exists():
            os.environ["RERUN_CLI_PATH"] = str(cli_path)
            break
