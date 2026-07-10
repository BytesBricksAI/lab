//! `ProcessVariableSample` archetype: scalar value plus quality.

use re_sdk_types::{
    AsComponents, ComponentBatch as _, ComponentDescriptor, SerializedComponentBatch, archetypes,
};

use crate::namespace::{ARCHETYPE_PROCESS_VARIABLE, field};
use crate::quality::Quality;

/// A single process-variable sample: engineering value and OPC UA-style quality.
pub struct ProcessVariableSample {
    /// Scalar engineering value (logged via builtin `Scalars` for time-series plotting).
    pub value: f64,

    /// Sample quality code.
    pub quality: Quality,
}

impl AsComponents for ProcessVariableSample {
    fn as_serialized_batches(&self) -> Vec<SerializedComponentBatch> {
        let mut batches = archetypes::Scalars::single(self.value).as_serialized_batches();

        let quality_descriptor = ComponentDescriptor {
            archetype: Some(ARCHETYPE_PROCESS_VARIABLE.into()),
            component: field(ARCHETYPE_PROCESS_VARIABLE, "quality").into(),
            component_type: Some(Quality::component_type()),
        };

        if let Some(quality_batch) = [self.quality.to_text()].serialized(quality_descriptor) {
            batches.push(quality_batch);
        }

        batches
    }
}

#[cfg(test)]
mod tests {
    use re_sdk_types::AsComponents as _;

    use super::*;

    #[test]
    fn process_variable_sample_batches() {
        let sample = ProcessVariableSample {
            value: 42.0,
            quality: Quality::from(sp_kernel::Quality::Good),
        };

        let batches = sample.as_serialized_batches();
        assert!(
            batches.len() >= 2,
            "expected scalar + quality batches, got {}",
            batches.len()
        );
    }
}
