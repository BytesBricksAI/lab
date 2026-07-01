# ml-dataloop (Python bindings) Specification

## Purpose

Exponer a Python la capacidad de construcción de datasets reproducibles de `sp_ml_dataloop`
bajo `simplant_lab.ml_dataloop`: el aggregate `DatasetSpec`, los value objects `FeatureSpec`
y `DataSplit` (con anti-leakage), y el adapter de consulta `RrdDataframeQuery` bajo
`simplant_lab.ml_dataloop.dataframe_query` que lee archivos `.rrd`.

## Requirements

### Requirement: Submódulo `simplant_lab.ml_dataloop` accesible

El sistema MUST exponer `simplant_lab.ml_dataloop` con `DatasetSpec`, `FeatureSpec`,
`DataSplit`, y el submódulo `simplant_lab.ml_dataloop.dataframe_query` con `RrdDataframeQuery`,
`QueryResult` y `TagSeries`.

#### Scenario: Importar el submódulo y el adapter de query

- GIVEN el paquete `simplant_lab` instalado
- WHEN se ejecuta `import simplant_lab; simplant_lab.ml_dataloop.dataframe_query.RrdDataframeQuery`
- THEN la clase existe
- AND `simplant_lab.ml_dataloop.DatasetSpec` y `DataSplit` están disponibles

### Requirement: Definir un `DataSplit` anti-leakage

El sistema MUST exponer `DataSplit(train, val, test)` (ventanas de tiempo) que MUST rechazar
splits con solapamiento temporal entre particiones, con accesores `train()`, `val()`,
`test()`, `windows()`.

#### Scenario: Split válido sin solapamiento

- GIVEN tres `TimeWindow` train/val/test disjuntas y ordenadas
- WHEN se construye `DataSplit(train, val, test)`
- THEN `windows()` devuelve las particiones nombradas

#### Scenario: Rechazar split con fuga temporal

- GIVEN ventanas train y test que se solapan
- WHEN se construye `DataSplit(...)`
- THEN se eleva una excepción Python

### Requirement: Definir un `DatasetSpec` versionado

El sistema MUST exponer la construcción de `DatasetSpec` a partir de features, targets, un
`DataSplit` y un `AssetCatalog`, validando que cada feature/target referencie un tag presente
en el catálogo, con accesores `id()`, `version()`, `features()`, `targets()`, `split()`.

#### Scenario: Definir dataset válido

- GIVEN features y targets cuyos tags existen en el catálogo y un `DataSplit` válido
- WHEN se define el `DatasetSpec`
- THEN se obtiene un `DatasetSpec` con `version()` ≥ 1

#### Scenario: Feature con tag ausente del catálogo

- GIVEN un `FeatureSpec` cuyo tag no existe en el catálogo
- WHEN se define el `DatasetSpec`
- THEN se eleva una excepción Python

### Requirement: Consultar un `.rrd` con `RrdDataframeQuery`

El sistema MUST exponer `RrdDataframeQuery.open(path)` y `query(window, tags)` que devuelva un
`QueryResult` con `series` (lista de `TagSeries` con `tag` y `measurements`), y MUST elevar
excepción Python si el archivo no existe o es ilegible.

#### Scenario: Consultar una ventana de tiempo

- GIVEN un `.rrd` con datos grabados y un `TimeWindow` con tags conocidos
- WHEN se invoca `RrdDataframeQuery.open(path).query(window, [tag_id])`
- THEN devuelve un `QueryResult`
- AND cada `series` tiene su `tag` y una lista de `measurements`

#### Scenario: Archivo .rrd inexistente

- GIVEN una ruta a un `.rrd` que no existe
- WHEN se invoca `RrdDataframeQuery.open(path)`
- THEN se eleva una excepción Python
