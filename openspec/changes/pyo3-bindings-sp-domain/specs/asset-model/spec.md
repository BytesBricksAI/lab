# asset-model (Python bindings) Specification

## Purpose

Exponer a Python el modelo de activos de `sp_asset_model` bajo `simplant_lab.asset_model`,
incluyendo el aggregate `AssetCatalog`, la jerarquía `Facility`/`Area`/`ProcessUnit`/
`Equipment`/`Tag`, los identificadores tipados, y el repositorio TOML para cargar catálogos
desde disco — el punto de entrada que las demás capacidades consumen.

## Requirements

### Requirement: Submódulo `simplant_lab.asset_model` accesible

El sistema MUST exponer un submódulo `simplant_lab.asset_model` con `AssetCatalog`,
`Facility`, `Area`, `ProcessUnit`, `TomlCatalogRepository` y los IDs `FacilityId`, `AreaId`,
`UnitId`, `EquipmentId` (reusando `TagId` de `simplant_lab.kernel`).

#### Scenario: Importar el submódulo

- GIVEN el paquete `simplant_lab` instalado
- WHEN se ejecuta `import simplant_lab; simplant_lab.asset_model`
- THEN expone `AssetCatalog`, `TomlCatalogRepository` y los IDs tipados

### Requirement: Cargar un catálogo desde TOML

El sistema MUST exponer `TomlCatalogRepository(path)` con `load_catalog()` que devuelva un
`AssetCatalog`, y MUST elevar una excepción Python si el archivo no existe o el contenido es
inválido.

#### Scenario: Cargar catálogo válido

- GIVEN una ruta a un archivo TOML de catálogo válido
- WHEN se invoca `TomlCatalogRepository(path).load_catalog()`
- THEN devuelve un `AssetCatalog`
- AND `catalog.tags()` contiene los tags definidos en el TOML

#### Scenario: Archivo inexistente

- GIVEN una ruta a un archivo que no existe
- WHEN se invoca `load_catalog()`
- THEN se eleva una excepción Python

### Requirement: Navegación del `AssetCatalog`

El sistema MUST exponer en `AssetCatalog` los accesos de lectura `facility()`, `equipment()`,
`tags()`, `tag(id)`, `equipment_by_id(id)` y `validate()`. Las colecciones MUST devolverse
como secuencias iterables desde Python.

#### Scenario: Iterar tags del catálogo

- GIVEN un `AssetCatalog` cargado con N tags
- WHEN se itera `for t in catalog.tags()`
- THEN se recorren los N tags
- AND `catalog.tag(some_tag_id)` devuelve el tag correspondiente o `None` si no existe

#### Scenario: Validar integridad del catálogo

- GIVEN un `AssetCatalog` consistente
- WHEN se invoca `validate()`
- THEN no se eleva excepción

### Requirement: Construcción de `Facility` y mutadores

El sistema MUST exponer la construcción de `Facility` y sus mutadores `add_area()` /
`add_unit()`, que MUST elevar excepción Python ante violaciones de invariante (ej. agregar
unidad a un área inexistente). Los IDs tipados MUST validar su valor al construirse.

#### Scenario: Agregar área y unidad

- GIVEN una `Facility` recién definida
- WHEN se invoca `add_area(area_id, "Área 1")` y luego `add_unit(area_id, unit_id, "U-1")`
- THEN `has_area(area_id)` y `has_unit(unit_id)` devuelven `True`

#### Scenario: Unidad en área inexistente

- GIVEN una `Facility` sin el área `A-99`
- WHEN se invoca `add_unit(AreaId("A-99"), unit_id, "U-1")`
- THEN se eleva una excepción Python
