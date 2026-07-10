# Kernel (Python bindings) specification

## Purpose

Exponer a Python los value objects de dominio de `sp_kernel` bajo el submódulo
`simplant_lab.kernel`, preservando sus invariantes y validaciones. Estos tipos son la base
reutilizada por las demás capacidades (asset_model, acquisition, simulation, ml_dataloop).

## Requirements

### Requirement: submódulo `simplant_lab.kernel` accesible

El sistema MUST exponer un submódulo `simplant_lab.kernel` que contenga los tipos de value
objects de `sp_kernel`: `TagId`, `Quality`, `Measurement`, `MeasurementBatch`, `TimeWindow`,
`UnitOfMeasure`, `Dimension`, `EngineeringRange`, `AlarmLimits`.

#### Scenario: importar el submódulo

- GIVEN el paquete `simplant_lab` instalado
- WHEN se ejecuta `import simplant_lab; simplant_lab.kernel`
- THEN el submódulo está disponible
- AND expone `TagId`, `Quality`, `Measurement`, `MeasurementBatch`, `TimeWindow`,
  `UnitOfMeasure`, `EngineeringRange`, `AlarmLimits`

### Requirement: `TagId` con validación ISA-5.1

El sistema MUST exponer `TagId(raw: str)` que valide el identificador y MUST elevar una
excepción Python cuando el valor es inválido (vacío/blanco). El identificador construido MUST
ser legible vía `as_str()` y `str()`.

#### Scenario: construir un TagId válido

- GIVEN un string `"FT-101"`
- WHEN se construye `simplant_lab.kernel.TagId("FT-101")`
- THEN `tag.as_str()` devuelve `"FT-101"`

#### Scenario: rechazar un TagId inválido

- GIVEN un string vacío `""`
- WHEN se construye `simplant_lab.kernel.TagId("")`
- THEN se eleva una excepción Python (`ValueError`)

### Requirement: `Quality` como enum

El sistema MUST exponer `Quality` con las variantes `Good`, `Uncertain`, `Bad` y el método
`is_usable()`.

#### Scenario: calidad utilizable

- GIVEN `simplant_lab.kernel.Quality.Good`
- WHEN se invoca `is_usable()`
- THEN devuelve `True`

#### Scenario: calidad no utilizable

- GIVEN `simplant_lab.kernel.Quality.Bad`
- WHEN se invoca `is_usable()`
- THEN devuelve `False`

### Requirement: `Measurement` y `MeasurementBatch`

El sistema MUST exponer `Measurement(value, quality, timestamp)` con getters `value()`,
`quality()`, `timestamp()`, y `MeasurementBatch(tag, samples)` con `tag()`, `samples()`,
`len()`, `is_empty()`, `time_span()`.

#### Scenario: crear y leer un measurement

- GIVEN `value=42.0`, `quality=Quality.Good`, `timestamp` válido
- WHEN se construye un `Measurement`
- THEN `value()==42.0`, `quality()` es `Good` y `timestamp()` coincide

#### Scenario: batch vacío reporta is_empty

- GIVEN un `MeasurementBatch` con una lista de samples vacía
- WHEN se invoca `is_empty()`
- THEN devuelve `True`
- AND `time_span()` devuelve `None`

### Requirement: `TimeWindow` con invariante start<end

El sistema MUST exponer `TimeWindow(start, end)` que MUST rechazar ventanas donde `start >=
end`, con getters `start()`, `end()`, `contains(ts)`, `overlaps(other)`, `duration()`.

#### Scenario: ventana válida contiene un instante

- GIVEN una `TimeWindow` con `start < end`
- WHEN se consulta `contains(ts)` con `start <= ts < end`
- THEN devuelve `True`

#### Scenario: rechazar ventana invertida

- GIVEN `start > end`
- WHEN se construye `TimeWindow(start, end)`
- THEN se eleva una excepción Python

### Requirement: `UnitOfMeasure`, `EngineeringRange`, `AlarmLimits`

El sistema MUST exponer `UnitOfMeasure` (enum con `dimension()`, `symbol()`, `to_base()`,
`from_base()`, `same_dimension()`), `EngineeringRange(low, high, unit)` con validación
`low<high` y `contains()`, y `AlarmLimits(low_low, low, high, high_high, unit)` con sus
getters `Option`.

#### Scenario: conversión de unidad a base

- GIVEN `UnitOfMeasure.Bar`
- WHEN se invoca `to_base(1.0)`
- THEN devuelve el valor en la unidad base de presión

#### Scenario: rango de ingeniería rechaza low>=high

- GIVEN `low=100.0`, `high=10.0`
- WHEN se construye `EngineeringRange(100.0, 10.0, UnitOfMeasure.Bar)`
- THEN se eleva una excepción Python

### Requirement: `Timestamp` interoperable con Python

El sistema MUST permitir construir y leer instantes de tiempo desde Python sin depender de
`jiff`. Los instantes MUST exponerse como segundos epoch (`float`) y SHOULD ofrecer también
nanos (`int`) y una representación ISO-8601 vía `str()`.

#### Scenario: round-trip de timestamp por epoch seconds

- GIVEN un instante construido desde `float` epoch-seconds
- WHEN se lee de vuelta `Measurement.timestamp()` como epoch-seconds
- THEN el valor coincide dentro de la tolerancia de `f64`
