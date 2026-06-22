<h1 align="center">
  <a href="https://www.rerun.io/">
    <img alt="banner" src="https://user-images.githubusercontent.com/1148717/218142418-1d320929-6b7a-486e-8277-fbeef2432529.png">
  </a>
</h1>

<h1 align="center">
  <a href="https://crates.io/crates/simplant-lab">                             <img alt="Latest version" src="https://img.shields.io/crates/v/simplant-lab.svg">                               </a>
  <a href="https://docs.rs/rerun">                                      <img alt="Documentation"  src="https://docs.rs/rerun/badge.svg">                                         </a>
  <a href="https://github.com/rerun-io/rerun/blob/main/LICENSE-MIT">    <img alt="MIT"            src="https://img.shields.io/badge/license-MIT-blue.svg">                        </a>
  <a href="https://github.com/rerun-io/rerun/blob/main/LICENSE-APACHE"> <img alt="Apache"         src="https://img.shields.io/badge/license-Apache-blue.svg">                     </a>
  <a href="https://discord.gg/Gcm8BbTaAj">                              <img alt="Rerun Discord"  src="https://img.shields.io/discord/1062300748202921994?label=Rerun%20Discord"> </a>
</h1>

# SimPlant Lab Rust logging SDK
SimPlant Lab is an SDK for logging computer vision and robotics data paired with a visualizer for exploring that data over time. It lets you debug and understand the internal state and data of your systems with minimal code.

```shell
cargo add simplant-lab
````

```rust
let rec = simplant_lab::RecordingStream::global(simplant_lab::StoreKind::Recording)?;
rec.log("points", &simplant_lab::archetypes::Points3D::new(points).with_colors(colors))?;
rec.log("image", &simplant_lab::archetypes::Image::new(image))?;
```

<p align="center">
  <img width="800" alt="SimPlant Lab Viewer" src="https://user-images.githubusercontent.com/1148717/218763490-f6261ecd-e19e-4520-9b25-446ce1ee6328.png">
</p>

## Getting started
- [Examples](https://github.com/rerun-io/rerun/tree/latest/examples/rust)
- [High-level docs](https://rerun.io/docs)
- [Rust API docs](https://docs.rs/rerun/)
- [Troubleshooting](https://www.rerun.io/docs/overview/installing-rerun/troubleshooting)

## Library
You can add the `simplant-lab` crate to your project with `cargo add simplant-lab`.

To get started, see [the examples](https://github.com/rerun-io/rerun/tree/latest/examples/rust).

## Binary
You can install the binary with `cargo install simplant-lab-cli --locked --features nasm`.

**Note**: this requires the [`nasm`](https://github.com/netwide-assembler/nasm) CLI to be installed and available in your path.
Alternatively, you may skip enabling the `nasm` feature, but this may result in inferior video decoding performance.

The `simplant-lab` CLI can act either as a server, a viewer, or both, depending on which options you use when you start it.

Running `simplant-lab` with no arguments will start the viewer, waiting for an SDK to connect to it over gRPC.

Run `simplant-lab --help` for more.

### Running a web viewer
The web viewer is an experimental feature, but you can try it out with:

```sh
simplant-lab --web-viewer path/to/file.rrd
```
