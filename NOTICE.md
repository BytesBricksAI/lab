# Attribution notice

This repository contains **SimPlant Lab**, a fork of [Rerun](https://github.com/rerun-io/rerun).

## Upstream project

SimPlant Lab is derived from Rerun (<https://github.com/rerun-io/rerun>), originally developed by **Rerun Technologies AB** (<opensource@rerun.io>).

The portions of this repository listed under [Derived from Rerun](#derived-from-rerun) below retain the upstream copyright and are used under the same dual license as the upstream project.

## License

SimPlant Lab is distributed under the terms of either:

- the [MIT License](LICENSE-MIT), or
- the [Apache License, Version 2.0](LICENSE-APACHE),

at your option (`MIT OR Apache-2.0`).

You may not use this software except in compliance with one of those licenses. Full license texts are in `LICENSE-MIT` and `LICENSE-APACHE` at the repository root.

### Upstream copyright (Rerun-derived portions)

For the Rerun-derived portions of this repository, the MIT license states:

> Copyright (c) 2022 Rerun Technologies AB \<opensource@rerun.io\>

The Apache License 2.0 text is provided in `LICENSE-APACHE`. Per that license, copyright notices in source files and third-party archives should be preserved where they appear.

## Portion attribution

### Derived from Rerun

Unless noted otherwise, the following areas are derived from Rerun and may contain modifications by SimPlant:

- `crates/build/`
- `crates/store/`
- `crates/top/`
- `crates/utils/`
- `crates/viewer/`
- `rerun_py/`
- `docs/snippets/`
- `examples/rust/`
- `run_wasm/`
- `tests/rust/`
- Other top-level files and directories not listed under [Original SimPlant Lab work](#original-simplant-lab-work)

These portions remain subject to the upstream `MIT OR Apache-2.0` licensing and the copyright notice above.

### Original SimPlant lab work

The following paths contain original SimPlant Lab code that is not part of the upstream Rerun tree:

- `crates/simplant/` — domain crates (`sp_kernel`, `sp_types`, `sp_asset_model`, `sp_recording`, `sp_acquisition`, `sp_acquisition_replay`, `sp_acquisition_modbus`, `sp_ml_dataloop`, `sp_simulation`, `sp_sim_engine`, `sp_stress_testing`, and related crates under this directory)
- `examples/simplant/` — SimPlant-specific examples (`tanque_demo`, `sim_demo`, and related examples under this directory)

Original SimPlant Lab work in this repository is also licensed under `MIT OR Apache-2.0`, consistent with the workspace `Cargo.toml` license field.

## Trademarks

"Rerun" and associated Rerun logos are trademarks of Rerun Technologies AB. SimPlant Lab is an independent fork and is not affiliated with or endorsed by Rerun Technologies AB.

## Repository

- **SimPlant Lab repository:** <https://github.com/SimPlant/SimPlant-v2>
- **Upstream Rerun repository:** <https://github.com/rerun-io/rerun>
