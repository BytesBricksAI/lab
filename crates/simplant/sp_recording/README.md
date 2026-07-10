# sp_recording

SimPlant Lab recording adapter: implements [`RecorderPort`] from `sp_acquisition` over [`re_sdk::RecordingStream`].

This is the **only** crate that translates domain types (`MeasurementBatch`, `Tag`, `AcquisitionEvent`) into Rerun store primitives (`ProcessVariableSample`, `TagMetadata`, `TextLog`).

## Entity paths

| Constant / function | Path |
|---------------------|------|
| `tag_entity_path` | `tags/<tag_id>` (flat path for F1) |
| `PLANT_TIME` | Timeline name for plant timestamps |
| `EVENTS_PATH` | `events/acquisition` |

Full hierarchical paths (`/site/area/unit/equipment/tag`) are deferred to a future milestone that requires the asset catalog inside the recorder.

## Usage

```rust
use sp_recording::RerunRecorder;
use sp_acquisition::RecorderPort;

let recorder = RerunRecorder::to_file("simplant_demo", "recording.rrd")?;
recorder.record_batch(&batch)?;
recorder.flush();
```

[`RecorderPort`]: https://docs.rs/sp_acquisition/latest/sp_acquisition/trait.RecorderPort.html
[`re_sdk::RecordingStream`]: https://docs.rs/re_sdk/latest/re_sdk/struct.RecordingStream.html
