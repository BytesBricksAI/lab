# Stress-testing (Python bindings) specification

## Purpose

Exponer a Python la capacidad de pruebas de estrés de `sp_stress_testing` bajo
`simplant_lab.stress_testing`: el aggregate `StressTest`, los value objects `LoadProfile`,
`LoadPoint`, `DesignLimit`, `SafetyFactor`, `AcceptanceCriterion`, `MeasuredOutcome`, y la
evaluación de resultados contra límites de diseño y criterios de aceptación.

## Requirements

### Requirement: submódulo `simplant_lab.stress_testing` accesible

El sistema MUST exponer `simplant_lab.stress_testing` con `StressTest`, `LoadProfile`,
`LoadPoint`, `DesignLimit`, `SafetyFactor`, `AcceptanceCriterion`, `MeasuredOutcome` y el
enum `StressTestState`.

#### Scenario: importar el submódulo

- GIVEN el paquete `simplant_lab` instalado
- WHEN se ejecuta `import simplant_lab; simplant_lab.stress_testing.StressTest`
- THEN la clase existe junto con sus value objects

### Requirement: construcción de value objects con validación

El sistema MUST exponer `LoadPoint(variable, value)`, `LoadProfile(points)`,
`DesignLimit(variable, max_value)`, `SafetyFactor(value)` (que MUST rechazar valores ≤0 o no
finitos), `AcceptanceCriterion(metric, max_value)` y `MeasuredOutcome(metric, value)`.

#### Scenario: crear un SafetyFactor válido

- GIVEN el valor `1.5`
- WHEN se construye `SafetyFactor(1.5)`
- THEN `value()` devuelve `1.5`

#### Scenario: rechazar SafetyFactor inválido

- GIVEN el valor `0.0`
- WHEN se construye `SafetyFactor(0.0)`
- THEN se eleva una excepción Python

### Requirement: planificar una prueba de estrés

El sistema MUST exponer `StressTest.plan(id, load_profile, safety_factor, design_limits,
acceptance_criteria)` que devuelva un `StressTest` en estado `Planned`, y MUST elevar excepción
Python ante una configuración inválida (ej. carga que excede `design_limit × safety_factor`).

#### Scenario: planificar prueba válida

- GIVEN un `LoadProfile`, un `SafetyFactor`, límites de diseño y criterios consistentes
- WHEN se invoca `StressTest.plan(...)`
- THEN se obtiene un `StressTest` con `state() == Planned`

#### Scenario: carga excede límite de diseño

- GIVEN un `LoadProfile` cuyo punto excede `design_limit × safety_factor`
- WHEN se invoca `StressTest.plan(...)`
- THEN se eleva una excepción Python

### Requirement: evaluar resultados medidos

El sistema MUST exponer `evaluate(outcomes)` sobre `StressTest`, mutándolo in-place a estado
`Completed`, comparando cada `MeasuredOutcome` contra los `AcceptanceCriterion`.

#### Scenario: evaluar resultados dentro de criterio

- GIVEN un `StressTest` en `Planned` y outcomes que cumplen los criterios
- WHEN se invoca `evaluate([outcome, ...])`
- THEN `state()` pasa a `Completed`
- AND el resultado de la evaluación indica aceptación

#### Scenario: evaluar dos veces

- GIVEN un `StressTest` ya en estado `Completed`
- WHEN se invoca `evaluate(...)` de nuevo
- THEN se eleva una excepción Python
