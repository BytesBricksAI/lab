# Recording (Python bindings) specification

## Purpose

Exponer a Python el adapter de grabación de `sp_recording` bajo `simplant_lab.recording`: el
`RerunRecorder` que implementa el puerto de grabación sobre un `RecordingStream` de Rerun,
permitiendo persistir la adquisición a archivos `.rrd`. Es el sink que `run_session`
(acquisition) consume.

## Requirements

### Requirement: submódulo `simplant_lab.recording` accesible

El sistema MUST exponer `simplant_lab.recording` con `RerunRecorder`, las constantes
`PLANT_TIME` y `EVENTS_PATH`, y el helper `tag_entity_path(tag)`.

#### Scenario: importar el submódulo

- GIVEN el paquete `simplant_lab` instalado
- WHEN se ejecuta `import simplant_lab; simplant_lab.recording.RerunRecorder`
- THEN la clase existe
- AND `simplant_lab.recording.PLANT_TIME` es un string

### Requirement: construir un recorder a archivo

El sistema MUST exponer `RerunRecorder.to_file(app_id, path)` que cree un recorder que graba a
un archivo `.rrd`, y MUST elevar excepción Python si la ruta no es escribible. SHOULD también
exponer `RerunRecorder(stream)` para envolver un `RecordingStream` existente del binding
heredado.

#### Scenario: recorder a archivo

- GIVEN un `app_id` y una ruta de salida escribible
- WHEN se invoca `RerunRecorder.to_file("demo", "out.rrd")`
- THEN se obtiene un recorder utilizable por `run_session`

### Requirement: flush del recorder

El sistema MUST exponer `flush()` sobre `RerunRecorder`, que fuerce la escritura pendiente al
sink.

#### Scenario: flush deja el .rrd consistente

- GIVEN un recorder a archivo que recibió batches vía `run_session`
- WHEN se invoca `flush()`
- THEN el archivo `.rrd` queda completo y legible (p.ej. por `RrdDataframeQuery`)

### Requirement: mapeo de tag a entity path

El sistema MUST exponer `tag_entity_path(tag)` que devuelva la ruta de entidad Rerun canónica
para un `TagId`.

#### Scenario: ruta de entidad para un tag

- GIVEN un `TagId("FT-101")`
- WHEN se invoca `simplant_lab.recording.tag_entity_path(tag)`
- THEN devuelve un string de entity path no vacío y determinista
