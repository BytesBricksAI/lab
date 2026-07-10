# sp_types

SimPlant Lab store types: `simplant.*` components and archetypes (anti-corruption layer over `re_types_core`).

## Overview

`sp_types` defines manual Rerun components and archetypes for logging industrial process data:

| Type | Rerun namespace | Purpose |
|------|-----------------|---------|
| `Quality` | `simplant.components.Quality` | OPC UA-style quality (`u8`: Bad=0, Uncertain=1, Good=2) |
| `ProcessVariableSample` | `simplant.archetypes.ProcessVariable` | Time-series sample: builtin `Scalars` value + quality |
| `TagMetadata` | `simplant.archetypes.TagMetadata` | Static tag metadata (unit, range, alarm limits) |

## Quality component (Text fallback)

A custom `u8` `Loggable` component was the preferred design (see `docs/snippets/all/tutorials/custom_data.rs`), but `re_byte_size::SizeBytes` is a sealed supertrait of `Loggable` and is not re-exported by `re_sdk_types`. Downstream crates therefore cannot implement custom components.

`Quality` is represented in the store as builtin [`components::Text`](https://docs.rs/re_sdk_types/latest/re_sdk_types/components/struct.Text.html) with values `"Bad"`, `"Uncertain"`, or `"Good"`. The `ComponentDescriptor` still tags the field with `simplant.components.Quality`. Domain code uses `Quality::to_u8()` and `From<sp_kernel::Quality>`.

## Tag metadata

`TagMetadata` uses [`DynamicArchetype`](https://docs.rs/re_types_core/latest/re_types_core/struct.DynamicArchetype.html) (the archetype-aware counterpart to `AnyValues` in this Rerun version). `AnyValues::default()` does not accept a custom archetype name; `DynamicArchetype::new("simplant.archetypes.TagMetadata")` is used instead so descriptors are tagged correctly.

## Example

```rust
use sp_types::{ProcessVariableSample, Quality, TagMetadata};
use sp_kernel::{Quality as KernelQuality, UnitOfMeasure};
use re_sdk_types::AsComponents as _;

let sample = ProcessVariableSample {
    value: 42.0,
    quality: Quality::from(KernelQuality::Good),
};
let _batches = sample.as_serialized_batches();

let metadata = TagMetadata::new(UnitOfMeasure::Bar, 0.0, 100.0)
    .with_alarm_high(95.0);
let _batches = metadata.as_serialized_batches();
```
