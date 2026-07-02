"""
See `python3 -m simplant_lab --help`.

CLI entry point for the SimPlant-Lab viewer. Prefer `simplant-lab` on PATH when available.
"""

from __future__ import annotations

import sys

from simplant_lab_cli.__main__ import main as cli_main

from simplant_lab import unregister_shutdown


def main() -> int:
    # Importing simplant_lab registers a shutdown hook that we know we don't
    # need when running the CLI directly. We can safely unregister it.
    unregister_shutdown()

    return cli_main()


if __name__ == "__main__":
    sys.exit(main())
