# Inc 3 — interactividad (setpoints y perturbaciones en vivo)

- **Estado:** Diseño (se refina al llegar a su ciclo)
- **Depende de:** [`02-panel-de-control.md`](02-panel-de-control.md)
- **Entrega:** cambiar setpoints y meter perturbaciones **mientras la simulación corre**, y ver el lazo de control responder.

---

## 1. Objetivo y alcance

**Dentro:**
- Sub-trait `InteractiveSimulator: SimulatorPort` con `set_input(var, value)`.
- `FirstOrderEngine` implementa `InteractiveSimulator`.
- `SimCommand` amplía: `SetSetpoint { var, value }`, `Disturb { var, value }`.
- Panel: sliders/inputs por variable de control + botón de perturbación.

**Fuera:** UI para *crear* procesos (evolución futura, ver overview §7).

## 2. Diseño

### 2.1 capacidad en caliente — sub-trait opt-in

`SimulatorPort` **no** se modifica (no todos los motores son interactivos; forzarlo viola ARL — acoplamiento innecesario). En su lugar, en `sp_simulation`:

```rust
pub trait InteractiveSimulator: SimulatorPort {
    /// Cambia una entrada (setpoint, boundary condition) en caliente.
    fn set_input(&mut self, var: &str, value: f64) -> Result<()>;
}
```

`FirstOrderEngine` (`sp_sim_engine`) lo implementa actualizando el setpoint/boundary correspondiente que ya usa internamente para integrar hacia el target.

### 2.2 cómo aplica el runner los comandos interactivos

El runner necesita poder invocar `set_input` cuando llega `SetSetpoint`/`Disturb`. Como no todo `SimulatorPort` es interactivo, dos variantes (a decidir en el ciclo):

- **A (recomendada):** el thread de sim conoce el tipo concreto (o un `&mut dyn InteractiveSimulator`) y aplica los comandos interactivos antes del `tick`. El `SimulationRunner` sigue siendo agnóstico; la *aplicación* del comando interactivo vive en el bucle del thread, que sí sabe si el motor es interactivo.
- **B:** el runner recibe `Option<&mut dyn InteractiveSimulator>` y descarta comandos interactivos si es `None` (con `on_event` avisando "motor no interactivo").

`Disturb` = un `set_input` puntual sumado al estado (escalón); `SetSetpoint` = cambio persistente del target.

### 2.3 panel (egui) — añadidos

- Por cada variable de control declarada en el `Scenario`: un slider/`DragValue` → `SetSetpoint { var, value }` (envía on-change).
- Botón "Perturbar" con selector de variable y magnitud → `Disturb`.
- (Opcional) marcar en el `TextLog`/eventos cada cambio para trazabilidad en la tabla.

## 3. Decisiones

| Decisión | Elegido | Alternativa | Por qué |
|---|---|---|---|
| Capacidad interactiva | sub-trait `InteractiveSimulator` | método en `SimulatorPort` | No fuerza a todos los motores (ARL: coupling intencional y mínimo). |
| Aplicar comando | en el bucle del thread (variante A) | dentro del runner con `Option<>` | Mantiene el runner agnóstico; quien conoce la interactividad es el thread. |
| Perturbación | `set_input` puntual (escalón) | tipo de evento separado | Reusa el mismo mecanismo; menos superficie. |

## 4. Testing (TDD)

- **`FirstOrderEngine::set_input`**: tras `set_input("setpoint", X)`, los `step` siguientes convergen hacia `X` (test de dominio en `sp_sim_engine`).
- **Runner/thread**: `SetSetpoint` aplicado entre pasos cambia la trayectoria posterior (fake `InteractiveSimulator`).
- **Motor no interactivo**: comando interactivo se descarta sin romper (si se adopta variante B).

## 5. Criterios de aceptación

- [ ] Con la sim corriendo, mover un setpoint hace que la variable converja al nuevo valor (visible en la tendencia).
- [ ] "Perturbar" produce un escalón visible y el lazo lo corrige.
- [ ] Un motor que no implementa `InteractiveSimulator` sigue funcionando (sin setpoints).
- [ ] Tests de dominio (`set_input`) verdes; clippy limpio.

## 6. Relación con otras specs

Cierra el contrato `SimCommand` del overview. Reusa el panel y el thread de Inc 2 y el runner/sink de Inc 1. Deja el terreno listo para la evolución futura (UI para crear procesos), porque la interacción ya está modelada como comandos sobre un `SimulatorPort`/`Scenario` agnóstico.
