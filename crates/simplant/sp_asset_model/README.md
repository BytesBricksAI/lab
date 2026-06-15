# sp_asset_model

SimPlant Lab asset model: plant hierarchy and process tags as DDD aggregates (`Facility`, `Equipment`, `Tag`).

## TOML catalog format

The `TomlCatalogRepository` reads and writes a single TOML file with the following structure:

```toml
[facility]
id = "FAC-01"
name = "Refinery"

[[area]]
id = "AREA-A"
name = "Crude"

[[unit]]
id = "UNIT-100"
area = "AREA-A"
name = "CDU"

[[equipment]]
id = "EQ-101"
unit = "UNIT-100"
name = "Separator"
kind = "Vessel"
max_pressure = 10.0
max_pressure_unit = "Bar"
max_temperature = 200.0
max_temperature_unit = "DegreeCelsius"

[[tag]]
id = "PT-1101"
equipment = "EQ-101"
description = "Column pressure"
unit = "Bar"
range_low = 0.0
range_high = 100.0
alarm_ll = 10.0
alarm_l = 20.0
alarm_h = 80.0
alarm_hh = 90.0
```

### `UnitOfMeasure` values

Serde deserializes `UnitOfMeasure` from variant names (case-sensitive):

- `Kilopascal`, `Bar`, `Psi`, `Megapascal`
- `DegreeCelsius`, `Kelvin`
- `CubicMeterPerHour`, `BarrelPerDay`, `KilogramPerHour`
- `Meter`, `Percent`, `Ratio`

### `EquipmentKind` values

- `Vessel`, `Tank`, `Pump`, `HeatExchanger`, `Valve`, `Pipe`, `Other`

Loading always validates domain invariants; invalid TOML returns an error and never produces invalid aggregates.
