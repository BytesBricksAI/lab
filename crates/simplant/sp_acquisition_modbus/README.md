# sp_acquisition_modbus

SimPlant Lab Modbus TCP acquisition adapter: implements [`DataSourcePort`] from `sp_acquisition` by polling holding and input registers over Modbus TCP.

**Read-only (OT safety):** this crate never writes to PLC registers. Only function codes 0x03 (read holding) and 0x04 (read input) are used.

## Address binding format

Each [`TagBinding`] `address` encodes a Modbus point:

```text
<kind>:<register>[:<scale>[:<offset>]]
```

| Field | Values | Default |
|-------|--------|---------|
| `kind` | `holding` or `input` | — |
| `register` | u16 register index | — |
| `scale` | finite f64 multiplier | `1.0` |
| `offset` | finite f64 addend | `0.0` |

Engineering value mapping (pure, deterministic):

```text
value = raw_u16 as f64 * scale + offset
```

Examples:

| Address | Meaning |
|---------|---------|
| `holding:0` | Holding register 0, raw value |
| `input:5:0.1` | Input register 5, scale 0.1 |
| `holding:10:0.01:-40.0` | Holding register 10, °C from 0.01 scale with −40 offset |

## Usage

```rust
use sp_acquisition::{DataSourcePort, MeasurementSource, SamplingPolicy, TagBinding};
use sp_acquisition_modbus::ModbusTcpSource;
use sp_kernel::TagId;

let source = ModbusTcpSource::from_str("192.168.1.10:502")?;
let tag = TagId::new("PT-1101")?;
let binding = TagBinding::new(tag, "holding:0:0.01:-40.0")?;
let mut stream = source.subscribe(&[binding], &SamplingPolicy::default())?;

loop {
    let batch = stream.next_batch()?.expect("live source never exhausts");
    // record or process batch…
}
```

Unlike CSV replay, a live Modbus source **never** returns `Ok(None)` from [`MeasurementSource::next_batch`]. The caller controls poll rate (typically using [`SamplingPolicy::period_ms`] between calls).

## Implementation notes

- [`DataSourcePort`] is synchronous; this adapter owns an internal Tokio runtime and uses `block_on` for `tokio-modbus` I/O.
- Register mapping (`parse_modbus_address`, `map_register`) is pure and unit-tested separately from network I/O.

[`DataSourcePort`]: https://docs.rs/sp_acquisition/latest/sp_acquisition/trait.DataSourcePort.html
[`TagBinding`]: https://docs.rs/sp_acquisition/latest/sp_acquisition/struct.TagBinding.html
[`MeasurementSource::next_batch`]: https://docs.rs/sp_acquisition/latest/sp_acquisition/trait.MeasurementSource.html#tymethod.next_batch
[`SamplingPolicy::period_ms`]: https://docs.rs/sp_acquisition/latest/sp_acquisition/struct.SamplingPolicy.html#method.period_ms
