# Types (Python bindings) specification

## Purpose

Exponer a Python la anti-corruption layer de `sp_types` (componentes y archetypes
`simplant.*`) bajo `simplant_lab.types`, para que el código Python pueda referenciar los
nombres canónicos de componentes/archetypes y construir muestras de variable de proceso.

## Requirements

### Requirement: submódulo `simplant_lab.types` accesible

El sistema MUST exponer un submódulo `simplant_lab.types` con `ProcessVariableSample`,
`TagMetadata`, `Quality` y las constantes de namespace (`ARCHETYPE_PROCESS_VARIABLE`,
`ARCHETYPE_TAG_METADATA`, `COMPONENT_QUALITY`).

#### Scenario: importar el submódulo y leer constantes

- GIVEN el paquete `simplant_lab` instalado
- WHEN se ejecuta `import simplant_lab; simplant_lab.types.ARCHETYPE_PROCESS_VARIABLE`
- THEN la constante existe y es un string no vacío con el namespace `simplant.*`

### Requirement: construcción de `ProcessVariableSample`

El sistema MUST exponer `ProcessVariableSample` y `TagMetadata` de forma que Python pueda
construirlos a partir de tipos primitivos y de `simplant_lab.kernel`.

#### Scenario: crear una ProcessVariableSample

- GIVEN un valor numérico y una `Quality`
- WHEN se construye una `ProcessVariableSample`
- THEN sus campos son legibles desde Python y consistentes con los valores provistos

### Requirement: helper de campo de archetype

El sistema MUST exponer un helper equivalente a `field(archetype, field_name) -> str` que
componga el nombre calificado de un campo dentro de un archetype.

#### Scenario: componer nombre de campo

- GIVEN un archetype `ARCHETYPE_PROCESS_VARIABLE` y el campo `"value"`
- WHEN se invoca el helper de campo
- THEN devuelve el nombre calificado esperado del campo
