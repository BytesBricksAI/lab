# Inc 1 — streaming en vivo (headless)

- **Estado:** Diseño listo para implementar
- **Depende de:** [`00-overview.md`](00-overview.md)
- **Entrega:** ejecutar la simulación nativa al ritmo del reloj y verla evolucionar en vivo en el viewer, con la tabla/tendencias que ya existen. Sin controles en la GUI todavía.

---

## 1. Objetivo y alcance

**Dentro:**
- Crate `sp_sim_runtime` con `SimulationRunner`, `SimCommand` (v1: `Start`/`Stop`), `SimSink`.
- Ejemplo `examples/simplant/sim_live`: arma escenario (reusa el de `sim_demo`), corre el runner con *pacing*, streamea en vivo al viewer.
- Adapter `SimSink → re_sdk::RecordingStream`.
- Pacing con factor de velocidad y duración configurables por CLI.

**Fuera:** panel GUI (Inc 2), interactividad (Inc 3).

## 2. Diseño

### 2.1 `sp_sim_runtime` (agnóstico, sin `re_*`)

Dependencias: `sp_simulation` (para `SimulatorPort`, `SimState`, `Result`). Nada más del store.

- `SimCommand` — solo `Start`, `Stop` en este incremento (el enum completo vive en el overview; se agrega por incremento).
- `SimSink` trait — `on_state(sim_time_secs, &SimState)`.
- `SimulationRunner` (contrato del overview: `tick` + `apply`):
  - `new(dt_secs)`, `apply(SimCommand)` — `Start` arranca, `Stop` marca fin.
  - `tick(engine, sink)` — **un paso** (el primitivo): `engine.step(dt)` → `sim_time += dt` → `sink.on_state(sim_time, &state)` → **pacing**. Es lo que Inc 2 invoca desde su thread.
  - `run(engine, sink, stop_after_secs)` — helper headless de este incremento: hace el loop de `tick()` hasta `stop_after_secs` (o `0` = indefinido) o hasta recibir `Stop`.
  - **Pacing**: target `dt_real = dt / speed`. Acumula deuda: mide con `Instant`, duerme `dt_real - elapsed` (saturado a 0). Evita drift acumulado.

### 2.2 adapter de salida

`RerunSimSink` sobre `re_sdk::RecordingStream`. Replica el patrón de `sim_demo`:
- `set_duration_secs("sim_time", sim_time)`,
- por cada `(var, value)` en `SimState`: `rec.log("sim/{var}", &Scalars::single(value))`.

Ubicación: en el ejemplo `sim_live` (es código de integración). Si Inc 2 lo reusa, se promueve a un crate adapter `sp_sim_recording`.

### 2.3 ejemplo `sim_live`

```
flowsheet/scenario (igual que sim_demo)
  → FirstOrderEngine::new(dt) + initialize(scenario)
  → RecordingStream con .spawn() + with_blueprint(blueprint)   // viewer en vivo
  → RerunSimSink(stream)
  → SimulationRunner::new(dt).run(engine, sink, duration)
```

- **Sink en vivo**: `RecordingStreamBuilder::new(app).with_blueprint(bp).spawn()` lanza el viewer nativo y streamea. El blueprint reutiliza el de `tanque_demo` adaptado a `/sim/**` con timeline `sim_time` (tabla + tendencias).
- **CLI**: `--speed <f64>` (default 1.0), `--duration <secs>` (default = `scenario.duration_secs()`; `0` = indefinido hasta Ctrl-C).

### 2.4 data flow

```
FirstOrderEngine.step(dt) → SimState
        → SimulationRunner (sim_time, pacing)
        → RerunSimSink → RecordingStream(spawn) → viewer nativo EN VIVO
```

## 3. Decisiones

| Decisión | Elegido | Alternativa | Por qué |
|---|---|---|---|
| Sink en vivo | `spawn()` (lanza viewer y streamea) | `serve_grpc()` (viewer se conecta) / `connect_grpc()` (viewer ya abierto) | `spawn` es el demo más simple de un comando. `connect` se documenta como variante para un viewer ya abierto. |
| Tiempo de sim | runner acumula `sim_time += dt` | exponer `current_time()` en `SimulatorPort` | No ensuciar el puerto; el concreto ya lo tiene pero el contrato no debe exigirlo. |
| Pacing | deuda temporal con `Instant` | `sleep(dt/speed)` fijo | El sleep fijo acumula drift; medir y compensar mantiene el ritmo. |

## 4. Testing (TDD)

`sp_sim_runtime` es testeable sin `re_*`:
- **Fake `SimulatorPort`** que devuelve estados conocidos por paso, y **fake `SimSink`** que acumula lo recibido.
- Test: `run` con `Stop` tras N pasos emite exactamente N estados con `sim_time` = `N·dt`.
- Test: `SetSpeed`/pacing — con velocidad alta el wall-clock total es < umbral (verificación grosera del pacing, tolerante para no ser flaky).
- Test: el orden de emisión y los valores coinciden con los del fake engine.

El ejemplo `sim_live` se valida E2E a mano (abre viewer) + `rerun rrd verify` si además se guarda a archivo opcionalmente.

## 5. Criterios de aceptación

- [ ] `cargo run -p sim_live` abre el viewer y los valores de `/sim/**` avanzan **al ritmo del reloj** (no instantáneo).
- [ ] `--speed 10` acelera ~10×; `--speed 1` ≈ tiempo real.
- [ ] La tabla y las tendencias (blueprint) se ven poblarse en vivo.
- [ ] `sp_sim_runtime` no depende de ningún crate `re_*` (verificable en su `Cargo.toml`).
- [ ] Tests del runner verdes (fakes), clippy `--no-deps` limpio.

## 6. Relación con otras specs

`SimulationRunner`/`SimCommand`/`SimSink` definidos acá son la base de Inc 2 (los mueve a un thread, agrega comandos) e Inc 3 (amplía el contrato). El `RerunSimSink` y el blueprint se reutilizan.
