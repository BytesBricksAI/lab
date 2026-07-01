# simulation (Python bindings) Specification

## Purpose

Exponer a Python la capacidad de simulación de procesos de `sp_simulation` bajo
`simplant_lab.simulation`: el aggregate `FlowsheetSpec` con análisis de grados de libertad,
el `Scenario`, los value objects de modelado (`UnitOp`, `MaterialStream`, `Specification`,
`BoundaryCondition`, `ChemicalComponent`, `Composition`) y enums, más el motor nativo
`FirstOrderEngine` bajo `simplant_lab.simulation.engine`. Es la cadena que ejercita `sim_demo`.

## Requirements

### Requirement: Submódulo `simplant_lab.simulation` accesible

El sistema MUST exponer `simplant_lab.simulation` con `FlowsheetSpec`, `Scenario`, `UnitOp`,
`MaterialStream`, `Specification`, `BoundaryCondition`, `ChemicalComponent`, `Composition`,
los enums `UnitOpKind`, `ThermoPackage`, `EngineCapability`, los IDs (`FlowsheetId`,
`UnitOpId`, `StreamId`, `ScenarioId`), y el submódulo `simplant_lab.simulation.engine` con
`FirstOrderEngine`.

#### Scenario: Importar el submódulo y el motor

- GIVEN el paquete `simplant_lab` instalado
- WHEN se ejecuta `import simplant_lab; simplant_lab.simulation.engine.FirstOrderEngine`
- THEN la clase existe
- AND `simplant_lab.simulation.FlowsheetSpec` y los enums están disponibles

### Requirement: Construir un flowsheet en borrador y analizar DOF

El sistema MUST exponer `FlowsheetSpec.draft(id, components, unit_ops, streams, specs,
thermo)` y `degrees_of_freedom()`, y MUST elevar excepción Python ante un flowsheet mal
especificado.

#### Scenario: Draft válido y cálculo de DOF

- GIVEN componentes, unit ops, streams y specs consistentes
- WHEN se invoca `FlowsheetSpec.draft(...)`
- THEN se obtiene un flowsheet en estado `Draft`
- AND `degrees_of_freedom()` devuelve un entero

#### Scenario: Draft inválido

- GIVEN unit ops o specs inconsistentes
- WHEN se invoca `FlowsheetSpec.draft(...)`
- THEN se eleva una excepción Python

### Requirement: Aprobar un flowsheet (gate de DOF=0)

El sistema MUST exponer `approve()` sobre `FlowsheetSpec`, mutándolo in-place, que MUST
elevar excepción Python si `degrees_of_freedom() != 0` (solo flowsheets cuadrados se aprueban).

#### Scenario: Aprobar flowsheet cuadrado

- GIVEN un `FlowsheetSpec` en `Draft` con `degrees_of_freedom() == 0`
- WHEN se invoca `approve()`
- THEN `is_approved()` devuelve `True`
- AND `state()` es `Approved`

#### Scenario: Rechazar aprobación con DOF≠0

- GIVEN un `FlowsheetSpec` con `degrees_of_freedom() != 0`
- WHEN se invoca `approve()`
- THEN se eleva una excepción Python

### Requirement: Aprobar un escenario sobre un flowsheet

El sistema MUST exponer `Scenario.approve(id, flowsheet, boundary_conditions, duration_secs,
required_capability)` que MUST validar contra el flowsheet aprobado y devolver el escenario.

#### Scenario: Aprobar escenario válido

- GIVEN un `FlowsheetSpec` aprobado y condiciones de borde válidas
- WHEN se invoca `Scenario.approve("SC-1", flowsheet, [bc], 120.0, EngineCapability.Dynamic)`
- THEN se obtiene un `Scenario` con `is_approved() == True`
- AND `duration_secs() == 120.0`

### Requirement: Motor `FirstOrderEngine` ejecuta el escenario

El sistema MUST exponer `FirstOrderEngine(tau_secs)` con `initialize(scenario)`, `step(dt_secs)`
y los accesores `current_time()` y `value_of(variable)`. `step` MUST devolver el estado de la
simulación (pares variable→valor) y avanzar el tiempo interno.

#### Scenario: Inicializar y avanzar la simulación

- GIVEN un `Scenario` aprobado y `FirstOrderEngine(20.0)`
- WHEN se invoca `initialize(scenario)` y luego `step(2.0)` repetidamente
- THEN cada `step` devuelve el estado con pares `(variable, valor)`
- AND `current_time()` avanza en múltiplos de `dt_secs`

#### Scenario: Step sin initialize

- GIVEN un `FirstOrderEngine` recién creado sin `initialize`
- WHEN se invoca `step(2.0)`
- THEN se eleva una excepción Python

### Requirement: Unit ops steady-state expuestas

El sistema MUST exponer las funciones de unit op steady-state (`mix`, `split`, `valve`,
`pump`, `pipe`) y `StreamState`, para cálculos sin termodinámica.

#### Scenario: Calcular una operación de mezcla

- GIVEN estados de stream de entrada válidos
- WHEN se invoca la función `mix(...)`
- THEN devuelve un `StreamState` resultante consistente
