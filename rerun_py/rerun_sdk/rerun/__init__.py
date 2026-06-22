"""
Deprecated compatibility shim — use `import simplant_lab` instead.

This module re-exports the SimPlant-Lab SDK under the old `rerun` name.
"""

from __future__ import annotations

import warnings

warnings.warn(
    "The `rerun` package name is deprecated; use `import simplant_lab` instead.",
    DeprecationWarning,
    stacklevel=2,
)

from simplant_lab import *  # noqa: F403
from simplant_lab import __version__  # noqa: F401
