# Inc 2 — Panel de control en la GUI

- **Estado:** Diseño (se refina al llegar a su ciclo)
- **Depende de:** [`01-streaming-en-vivo.md`](01-streaming-en-vivo.md) (reusa `SimulationRunner`, `SimSink`, `RerunSimSink`, blueprint)
- **Entrega:** controlar la simulación **desde la GUI** — Start/Pause/Resume/Stop/Reset y velocidad — viendo los datos en vivo.

---

## 1. Objetivo y alcance

**Dentro:**
- Ejemplo `examples/simplant/sim_studio`: app `eframe` que envuelve `re_viewer::App` y le agrega un panel de control egui.
- La simulación corre en un **thread**; el panel la comanda por `mpsc<SimCommand>`; los datos llegan al viewer por un **log-channel in-process**.
- Comandos: `Start`, `Pause`, `Resume`, `Stop`, `Reset`, `SetSpeed(f64)`.

**Fuera:** setpoints/perturbaciones interactivas (Inc 3); integración al binario oficial.

## 2. Diseño

### 2.1 Estructura de la app (patrón `extend_viewer_ui` / `custom_callback`)

```rust
struct SimStudio {
    viewer: re_viewer::App,        // el viewer embebido
    tx_cmd: Sender<SimCommand>,    // panel → thread de sim
    state: PanelState,             // velocidad, corriendo/pausado, escenario
}

impl eframe::App for SimStudio {
    fn ui(&mut self, ui, frame) {
        egui::SidePanel::right("Simulación").show_inside(ui, |ui| self.control_ui(ui));
        self.viewer.ui(ui, frame);   // el viewer ocupa el resto
    }
}
```

Arranque (igual que `examples/rust/custom_callback/src/viewer.rs`):
- `re_viewer::App::new(main_thread_token, build_info, app_env, startup_options, cc, …, async_runtime)`.
- `app.add_log_receiver(rx_log)` — el viewer consume el log-channel donde escribe la sim.
- `eframe::run_native(title, options, |cc| SimStudio::new(app, tx_cmd))`.

### 2.2 Hilo de simulación

```
thread:
  loop {
    drenar rx_cmd → runner.apply(cmd)        // Start/Pause/SetSpeed/...
    if corriendo { runner.tick(engine, sink) }  // step + pacing + emit
    if Stop { break }
  }
```

- `engine`: `FirstOrderEngine` (o cualquier `SimulatorPort`).
- `sink`: `RerunSimSink` sobre un `RecordingStream` cuyo destino es el **log-channel** que el viewer recibe (no `spawn`, porque el viewer es local; se usa un sink de canal in-process).
- `Reset` reconstruye/`initialize` el engine y reinicia `sim_time`.

### 2.3 Comunicación (un proceso, sin red)

| Dirección | Mecanismo |
|---|---|
| Panel → sim | `std::sync::mpsc::Sender<SimCommand>` |
| Sim → viewer | `RecordingStream` → `LogReceiver` → `app.add_log_receiver` (in-process) |

El viewer recibe datos de **cualquier `LogReceiver`** (`custom_callback/src/viewer.rs:31`), así que no hace falta gRPC.

### 2.4 Panel (egui)

- Botones: ▶ Start · ⏸ Pause/Resume · ⏹ Stop · ↺ Reset.
- Slider de velocidad (`0.1×`–`50×`) → `SetSpeed`.
- Lectura: `sim_time` actual, estado (corriendo/pausado), último valor de cada variable.
- (Opcional) selector de escenario, si hay más de uno.

## 3. Decisiones

| Decisión | Elegido | Alternativa | Por qué |
|---|---|---|---|
| Proceso | uno solo (thread + canales) | 2 procesos + gRPC | Sim nativa liviana; sin red. |
| Transporte de datos | log-channel in-process | `serve_grpc` local | Más simple y directo en el mismo proceso. |
| `Reset` | `initialize()` de nuevo en el thread | recrear el thread | Menos churn; el runner ya soporta reinicio de `sim_time`. |

## 4. Testing

- **Lógica**: `SimulationRunner` con los nuevos comandos (`Pause`/`Resume`/`SetSpeed`/`Reset`) se testea headless con fakes (igual que Inc 1). Pausa ⇒ no emite estados; `SetSpeed` cambia el pacing; `Reset` vuelve `sim_time` a 0.
- **UI egui**: difícil de unit-testear; se valida a mano. La lógica testeada vive en el runner, no en el panel (el panel solo envía `SimCommand`).

## 5. Criterios de aceptación

- [ ] `cargo run -p sim_studio` abre SimPlant Lab con el panel a la derecha.
- [ ] Start arranca y los datos se pueblan en vivo; Pause congela; Resume sigue; Stop detiene; Reset reinicia desde `sim_time = 0`.
- [ ] El slider de velocidad cambia el ritmo en vivo.
- [ ] La sim corre en su thread sin congelar la UI.

## 6. Relación con otras specs

Reusa todo Inc 1 (runner, sink, blueprint) moviéndolo a un thread comandado. El contrato `SimCommand` se amplía aquí y se vuelve a ampliar en Inc 3 (setpoints). El esqueleto de app `eframe` es la base sobre la que Inc 3 agrega sliders de setpoint.
