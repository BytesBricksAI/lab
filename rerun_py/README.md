# The SimPlant-Lab Python SDK

Use the SimPlant-Lab SDK to record data like images, tensors, point clouds, and text. Data is streamed to the SimPlant-Lab viewer for live visualization or to file for later use.

<p align="center">
  <img width="800" alt="SimPlant-Lab Viewer" src="https://github.com/rerun-io/rerun/assets/2624717/c4900538-fc3a-43b8-841a-8d226e7b5a2e">
</p>

## Install

```sh
pip3 install simplant-lab-sdk
```

ℹ️ Note:
The Python module is called `simplant_lab`, while the package published on PyPI is `simplant-lab-sdk`.
The legacy names `rerun` / `rerun-sdk` remain available as deprecated shims.

For other SDK languages see [Installing SimPlant-Lab](https://www.rerun.io/docs/overview/installing-rerun/viewer).

We also provide a [Jupyter widget](https://pypi.org/project/rerun-notebook/) for interactive data visualization in Jupyter notebooks:
```sh
pip3 install simplant-lab-sdk[notebook]
```

## Example
```py
import simplant_lab as sl
import numpy as np

sl.init("simplant_example_app", spawn=True)  <!-- NOLINT -->

positions = np.vstack([xyz.ravel() for xyz in np.mgrid[3 * [slice(-5, 5, 10j)]]]).T
colors = np.vstack([rgb.ravel() for rgb in np.mgrid[3 * [slice(0, 255, 10j)]]]).astype(np.uint8).T

sl.log("points3d", sl.Points3D(positions, colors=colors))
```

## Resources
* [Examples](https://www.rerun.io/examples)
* [Python API docs](https://ref.rerun.io/docs/python)
* [Quick start](https://www.rerun.io/docs/getting-started/data-in/python)
* [Tutorial](https://www.rerun.io/docs/getting-started/data-in/python)
* [Troubleshooting](https://www.rerun.io/docs/overview/installing-rerun/troubleshooting)
* [Discord Server](https://discord.com/invite/Gcm8BbTaAj)

## Logging and viewing in different processes

You can run the viewer and logger in different processes.

In one terminal, start up a viewer with a server that the SDK can connect to:
```sh
simplant-lab
```

In a second terminal, run the example with the `--connect` option:
```sh
pixi run uvpy examples/python/minimal/minimal.py --connect
```
Note that SDK and viewer can run on different machines!


# Building SimPlant-Lab from source

We use [`pixi`](https://pixi.sh/) for managing dev-tool versioning, download and task running. See [here](https://pixi.sh/latest/#installation) for installation instructions.

```sh
pixi run py-build
```
This builds the SDK for Python (use `pixi run py-build-release` for a release build).

You can then run examples via uv:
```sh
pixi run uvpy examples/python/minimal/minimal.py
```

To build a wheel instead for manual install use:
```sh
pixi run py-build-wheel
```

## Editable installs from another project

When consuming the SDK from an external project via `{ path = "…/rerun_py", editable = true }`, also add `rerun-dev-fixup` from `rerun_py/rerun_dev_fixup` so `import simplant_lab` resolves correctly.

Build with:
```sh
RERUN_ALLOW_MISSING_BIN=1 pixi run uv sync --package simplant-lab-sdk
```

Run scripts with:
```sh
pixi run uvpy your_script.py
```
