# sp_simulation

SimPlant Lab process simulation core: versionable flowsheet specifications with
degrees-of-freedom (DOF) analysis, scenarios, and simulation runs.

Pure domain crate — **no `re_*` dependencies**, no async, no panics in production paths.

## Concepts

- **`FlowsheetSpec`**: aggregate for unit operations, streams, components, and specs.
  Structural invariants are enforced at construction; DOF must be zero before approval.
- **`Scenario`**: binds an approved flowsheet to boundary conditions and engine capability.
- **`SimulationRun`**: execution record linked 1:1 to a recording.

## Degrees of freedom

Each unit operation kind requires a fixed number of specifications (simplified F4 model):

| Kind       | Required specs |
|------------|----------------|
| Mixer      | 0              |
| Splitter   | 1              |
| Heater     | 1              |
| Cooler     | 1              |
| Valve      | 1              |
| Pump       | 1              |
| FlashDrum  | 2              |
| Pipe       | 1              |

`degrees_of_freedom = sum(required_specs) - count(declared specs)`. Approval requires DOF = 0.

## TOML flowsheet format

Load and save draft (or approved) flowsheets via `load_flowsheet`, `flowsheet_from_str`, and
`save_flowsheet`. All files pass through `FlowsheetSpec::draft` validation.

```toml
id = "FS-01"
version = 1
thermo = "IdealGas"
state = "Draft"

[[component]]
name = "Nitrogen"

[[component]]
name = "Oxygen"

[[unit_op]]
id = "H-01"
kind = "Heater"
name = "Feed Heater"

[[stream]]
id = "S-IN"
to = "H-01"
composition = [0.79, 0.21]

[[spec]]
unit_op = "H-01"
variable = "duty"
value = 250000.0
```

### Fields

| Field | Description |
|-------|-------------|
| `id` | Flowsheet identifier |
| `version` | Monotonic version (incremented on `revise`) |
| `thermo` | `PengRobinson`, `Srk`, `PcSaft`, or `IdealGas` |
| `state` | `Draft` or `Approved` |
| `component.name` | Chemical component name |
| `unit_op.id` | Unit operation identifier |
| `unit_op.kind` | See DOF table above |
| `unit_op.name` | Display name |
| `stream.id` | Stream identifier |
| `stream.from` | Upstream unit op (omit for feeds) |
| `stream.to` | Downstream unit op (omit for products) |
| `stream.composition` | Molar fractions (required normalized for feeds) |
| `spec.unit_op` | Target unit operation |
| `spec.variable` | Fixed variable name |
| `spec.value` | Fixed numeric value |

## Example

```rust
use sp_simulation::{
    ChemicalComponent, Composition, FlowsheetId, FlowsheetSpec, MaterialStream,
    Specification, StreamId, ThermoPackage, UnitOp, UnitOpId, UnitOpKind,
};

let components = vec![
    ChemicalComponent::new("N2").unwrap(),
    ChemicalComponent::new("O2").unwrap(),
];
let unit_ops = vec![UnitOp::new(
    UnitOpId::new("H-01").unwrap(),
    UnitOpKind::Heater,
    "Heater",
).unwrap()];
let streams = vec![MaterialStream::new(
    StreamId::new("S-IN").unwrap(),
    None,
    Some(UnitOpId::new("H-01").unwrap()),
    Composition::new(vec![0.79, 0.21]),
)];
let specs = vec![Specification::new(
    UnitOpId::new("H-01").unwrap(),
    "duty",
    100.0,
).unwrap()];

let mut fs = FlowsheetSpec::draft(
    FlowsheetId::new("FS-01").unwrap(),
    components,
    unit_ops,
    streams,
    specs,
    ThermoPackage::IdealGas,
).unwrap();

assert_eq!(fs.degrees_of_freedom(), 0);
fs.approve().unwrap();
```
