#!/usr/bin/env python3
"""Example template."""

from __future__ import annotations

import argparse

import simplant_lab as rr  # pip install simplant-lab-sdk


def main() -> None:
    parser = argparse.ArgumentParser(description="Example of using the Rerun visualizer")
    rr.script_add_args(parser)
    args = parser.parse_args()

    rr.script_setup(args, "rerun_example_my_example_name")

    # … example code

    rr.script_teardown(args)


if __name__ == "__main__":
    main()
