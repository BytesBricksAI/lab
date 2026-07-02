"""See `python3 -m simplant_lab_cli --help`."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def exe_suffix() -> str:
    if sys.platform.startswith("win"):
        return ".exe"
    return ""


def add_exe_suffix(path: str) -> str:
    if not path.endswith(exe_suffix()):
        return path + exe_suffix()
    return path


def _resolve_cli_path() -> str:
    if override := os.environ.get("RERUN_CLI_PATH"):
        return override

    cli_dir = Path(__file__).resolve().parent
    for name in ("simplant-lab", "rerun"):
        candidate = add_exe_suffix(str(cli_dir / name))
        if os.path.exists(candidate):
            return candidate

    # Editable dev install: .../lab/rerun_py/rerun_sdk/simplant_lab_cli/__main__.py
    lab_root = cli_dir.parents[2]
    cargo_target = Path(os.environ.get("CARGO_TARGET_DIR", lab_root / "target"))
    for name in ("simplant-lab", "rerun"):
        dev_bin = add_exe_suffix(str(cargo_target / "debug" / name))
        if os.path.exists(dev_bin):
            return dev_bin

    return add_exe_suffix(str(cli_dir / "simplant-lab"))


def main() -> int:
    target_path = _resolve_cli_path()

    if not os.path.exists(target_path):
        print(f"Error: Could not find SimPlant-Lab viewer binary at {target_path}", file=sys.stderr)
        return 1

    try:
        return subprocess.call([target_path, *sys.argv[1:]])
    except KeyboardInterrupt:
        return 130


if __name__ == "__main__":
    main()
