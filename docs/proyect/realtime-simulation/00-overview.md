# Simulación en tiempo real — overview y roadmap

- **Fecha:** 2026-06-22
- **Estado:** Diseño aprobado (pendiente de specs por incremento)
- **Alcance:** ejecutar simulaciones de proceso *en vivo* y, eventualmente, controlarlas desde la GUI de SimPlant Lab.

---

## 1. Problema

Hoy la simulación nativa (`examples/simplant/sim_demo`) corre el motor lo más rápido posible y escribe toda la trayectoria a un `.rrd` con `RecordingStreamBuilder::save()`. El usuario abre el archivo *después*. No hay "tiempo real": ni ritmo de reloj, ni streaming en vivo, ni control desde la interfaz.

El objetivo es **generar y ver los datos en vivo**, y a futuro **controlar y crear** las simulaciones desde la propia GUI.

## 2. Visión

> Abrir SimPlant Lab, arrancar una simulación desde un panel, verla evolucionar al ritmo del reloj, y ajustar setpoints/perturbaciones mientras corre — con un motor **agnóstico del proceso**, de modo que más adelante una UI permita *crear* los procesos a simular.

Restricción de diseño derivada de esa visión: **nada se casa con un modelo físico concreto**. Todo cuelga de `SimulatorPort` + `Scenario` (ya existentes en `sp_simulation`). El modelo de hoy (`FirstOrderEngine`, un heater) es solo el primer adapter.

## 3. Columna vertebral compartida

Crate nuevo **`crates/simplant/sp_sim_runtime`** — dominio/aplicación puro, **sin dependencias `re_*`** (anti-corrupción: el runtime no conoce el store ni la GUI).

```
┌──────────────────────── sp_sim_runtime (agnóstico) ───────────────────────┐
│                                                                            │
│   SimCommand  ──►  SimulationRunner  ──(SimState)──►  dyn SimSink          │
│   (mpsc)            • loop port.step(dt)                  (puerto salida)   │
│                     • lleva sim_time acumulado                              │
│                     • pacing: dt_real = dt / velocidad                      │
│                     • aplica comandos entre pasos                           │
│                            │                                                │
│                            ▼                                                │
│                    &mut dyn SimulatorPort   (sp_simulation, ya existe)      │
└────────────────────────────────────────────────────────────────────────────┘
        ▲ comandos                                   │ estados
   (panel GUI / CLI)                          (adapter re_sdk → store/viewer)
```

### 3.1 contratos (el "pegamento" entre las 3 specs)

**`SimCommand`** — enum que cada incremento amplía (ésta es la relación entre specs):

```rust
pub enum SimCommand {
    // Inc 1
    Start,
    Stop,
    // Inc 2
    Pause,
    Resume,
    SetSpeed(f64),   // factor: 1.0 = tiempo real, 10.0 = 10×, 0 = pausa
    Reset,
    // Inc 3
    SetSetpoint { var: String, value: f64 },
    Disturb     { var: String, value: f64 },
}
```

**`SimSink`** — puerto de salida (mantiene `sp_sim_runtime` libre de `re_*`):

```rust
pub trait SimSink {
    fn on_state(&mut self, sim_time_secs: f64, state: &SimState);
    fn on_event(&mut self, event: &str) {}   // start/stop/reset, opcional
}
```

**`SimulationRunner`** — orquestador agnóstico:

```rust
pub struct SimulationRunner { /* velocidad, sim_time, paused, dt */ }

impl SimulationRunner {
    pub fn new(dt_secs: f64) -> Self;
    // Avanza un paso: aplica comandos pendientes, step(dt), pacing, emite estado.
    pub fn tick(&mut self, engine: &mut dyn SimulatorPort, sink: &mut dyn SimSink) -> Result<()>;
    pub fn apply(&mut self, cmd: SimCommand);
}
```

`SimulatorPort` no expone el tiempo, así que el runner acumula `sim_time += dt` por paso.

## 4. Decisiones de arquitectura

| Decisión | Elegido | Alternativa | Por qué |
|---|---|---|---|
| Transporte panel↔motor | **Un proceso** (canales `mpsc` + log-channel in-process) | 2 procesos + gRPC (patrón `examples/rust/custom_callback`) | La sim nativa es liviana; no necesita red ni binario extra. El runner no cambia si mañana se quiere gRPC (solo el adapter). |
| Dónde vive | **Ejemplos** en `examples/simplant/` | Integrar al binario `simplant-lab` oficial | No toca producto ni zona upstream. Integrar es un paso posterior, cuando madure. |
| Salida de datos | **`SimSink` puerto** + adapter `re_sdk` | Loguear `re_sdk` directo desde el runner | Mantiene `sp_sim_runtime` agnóstico (ARL: el runtime no conoce el store). |

## 5. Roadmap (3 incrementos · specs separadas)

| # | Spec | Entrega | Toca | Estado |
|---|---|---|---|---|
| 1 | [`01-streaming-en-vivo.md`](01-streaming-en-vivo.md) | Ver la sim en vivo al ritmo real (headless, sin GUI) | ejemplo `sim_live` + `sp_sim_runtime` | Listo para implementar |
| 2 | [`02-panel-de-control.md`](02-panel-de-control.md) | Controlar la sim desde la GUI (Start/Pause/Stop/velocidad) | + envoltura de `re_viewer` en `eframe` | Diseño |
| 3 | [`03-interactividad.md`](03-interactividad.md) | Cambiar setpoints/perturbaciones en vivo, el lazo responde | + sub-trait `InteractiveSimulator` | Diseño |

## 6. Cómo se relacionan las specs

Las tres comparten `SimulationRunner` + `SimCommand` + `SimSink` (sección 3). Cada incremento **amplía** sin reescribir:

- Inc 1 ejercita el runner en modo headless con `SimCommand::{Start, Stop}`.
- Inc 2 mueve el runner a un thread y lo comanda desde un panel egui (agrega `Pause/Resume/SetSpeed/Reset`).
- Inc 3 amplía el contrato (`SetSetpoint/Disturb`) y el motor (sub-trait opt-in).

El adapter `SimSink → re_sdk` y el blueprint de tabla/tendencias (ya construido en `tanque_demo`) se reutilizan en los tres.

## 7. Evolución futura (fuera de alcance ahora)

- **UI para crear procesos**: un constructor de `Scenario`/`FlowsheetSpec` en la GUI. Encaja sin fricción: el runner ya ejecuta cualquier `SimulatorPort` inicializado con un `Scenario`.
- **Integración al binario oficial** `simplant-lab`.
- **Motores pesados/remotos** (p. ej. sidecar DWSIM): se cambia el adapter de transporte a gRPC; el runner no cambia.

## 8. Constraints y riesgos

- **Web viewer**: las extensiones de GUI (Inc 2/3) corren en el **viewer nativo**; el web viewer requiere build propio (ver `examples/rust/custom_callback` README). Alcance: nativo.
- **Zona upstream**: Inc 2/3 usan `re_viewer` como librería (no lo modifican). No agregan diff upstream nuevo; si en algún punto hiciera falta, se registra en [`../UPSTREAM_DIFF.md`](../UPSTREAM_DIFF.md).
- **Pacing**: usa `std::thread::sleep` + `Instant`; en `dt` chicos el jitter del scheduler es visible. Mitigación: acumular deuda temporal y dormir por diferencia, no por `dt` fijo.
