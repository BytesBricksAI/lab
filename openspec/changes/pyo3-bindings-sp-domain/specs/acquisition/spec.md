# acquisition (Python bindings) Specification

## Purpose

Exponer a Python la capacidad de adquisición de datos de planta: el aggregate
`AcquisitionSession`, sus value objects (`TagBinding`, `SamplingPolicy`), la función de
orquestación `run_session`, y los adapters de fuente `CsvReplaySource` (`replay`) y
`ModbusTcpSource` (`modbus`). Es la cadena que la demo `tanque_demo` ejercita end-to-end.

## Requirements

### Requirement: Submódulo `simplant_lab.acquisition` accesible

El sistema MUST exponer `simplant_lab.acquisition` con `AcquisitionSession`, `TagBinding`,
`SamplingPolicy`, `SessionState`, la función `run_session`, y los submódulos
`simplant_lab.acquisition.replay` (con `CsvReplaySource`) y `simplant_lab.acquisition.modbus`
(con `ModbusTcpSource` y helpers de direccionamiento).

#### Scenario: Importar el submódulo y sus adapters

- GIVEN el paquete `simplant_lab` instalado
- WHEN se ejecuta `import simplant_lab; simplant_lab.acquisition.replay.CsvReplaySource`
- THEN la clase existe
- AND `simplant_lab.acquisition.modbus.ModbusTcpSource` también existe

### Requirement: Crear una sesión de adquisición

El sistema MUST exponer `AcquisitionSession.create(id, bindings, policy, catalog)` que valide
los bindings contra el catálogo y MUST elevar excepción Python si un binding referencia un tag
ausente del catálogo. `TagBinding(tag, address)` y `SamplingPolicy(period_ms, deadband)` MUST
ser construibles desde Python.

#### Scenario: Crear sesión con bindings válidos

- GIVEN un `AssetCatalog` con el tag `FT-101` y un `TagBinding(TagId("FT-101"), "addr")`
- WHEN se invoca `AcquisitionSession.create("s1", [binding], SamplingPolicy(1000, None), catalog)`
- THEN se obtiene una sesión en estado `Created`

#### Scenario: Binding a tag inexistente

- GIVEN un `AssetCatalog` sin el tag `XX-999`
- WHEN se crea una sesión con un binding a `XX-999`
- THEN se eleva una excepción Python

### Requirement: Transiciones de estado de la sesión

El sistema MUST exponer `start()` y `stop(batches_recorded)` sobre `AcquisitionSession`,
mutando la sesión in-place, exponiendo `state()` y respetando la state machine
`Created → Running → Stopped`.

#### Scenario: Ciclo de vida de la sesión

- GIVEN una sesión en estado `Created`
- WHEN se invoca `start()`
- THEN `state()` es `Running`
- AND tras `stop(10)` el `state()` es `Stopped`

#### Scenario: Stop sin start

- GIVEN una sesión en estado `Created`
- WHEN se invoca `stop(0)` sin haber llamado `start()`
- THEN se eleva una excepción Python

### Requirement: Orquestación `run_session` con adapters nativos

El sistema MUST exponer `run_session(session, catalog, source, recorder)` que acepte un
`AcquisitionSession`, un `AssetCatalog`, una fuente que implemente el puerto de datos
(`CsvReplaySource` o `ModbusTcpSource`) y un recorder (`simplant_lab.recording.RerunRecorder`),
y MUST devolver el número de batches grabados.

#### Scenario: Ejecutar una sesión de replay end-to-end

- GIVEN una sesión válida, un `CsvReplaySource` apuntando a un CSV de prueba y un
  `RerunRecorder` a archivo
- WHEN se invoca `run_session(session, catalog, source, recorder)`
- THEN devuelve un entero ≥ 0 con el número de batches
- AND el archivo `.rrd` de salida queda escrito tras `recorder.flush()`

### Requirement: Adapter de replay CSV

El sistema MUST exponer `CsvReplaySource(path)` construible desde Python y utilizable como
fuente en `run_session`.

#### Scenario: Construir CsvReplaySource

- GIVEN una ruta a un CSV
- WHEN se construye `simplant_lab.acquisition.replay.CsvReplaySource(path)`
- THEN el objeto se crea sin error y es aceptado por `run_session`

### Requirement: Adapter Modbus TCP y direccionamiento

El sistema MUST exponer `ModbusTcpSource` (construible desde una dirección host:puerto), el
helper `parse_modbus_address(s)` y `map_register(raw, point)`, junto con el enum
`RegisterKind`.

#### Scenario: Parsear una dirección Modbus

- GIVEN el string `"holding:40001:0.1:5.0"`
- WHEN se invoca `simplant_lab.acquisition.modbus.parse_modbus_address(...)`
- THEN devuelve un `ModbusPoint` con `kind()==Holding`, `register()==40001`, `scale()==0.1`,
  `offset()==5.0`

#### Scenario: Dirección Modbus inválida

- GIVEN un string mal formado `"bogus"`
- WHEN se invoca `parse_modbus_address("bogus")`
- THEN se eleva una excepción Python

### Requirement: Implementar puertos desde Python (fuera de alcance)

El sistema MAY, en un change futuro, permitir implementar `DataSourcePort`/`RecorderPort`
desde Python puro. En esta entrega NO se soporta: solo se componen adapters nativos.

#### Scenario: Documentar la limitación

- GIVEN la primera entrega de los bindings
- WHEN un usuario intenta pasar una fuente implementada en Python puro a `run_session`
- THEN la API acepta únicamente los adapters nativos provistos (replay/modbus)
