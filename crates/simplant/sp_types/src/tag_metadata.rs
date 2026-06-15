//! `TagMetadata` archetype: static engineering metadata for a process tag.

use re_sdk_types::{AsComponents, DynamicArchetype, SerializedComponentBatch, components};

use crate::namespace::ARCHETYPE_TAG_METADATA;

/// Static metadata for a process tag (unit, range, optional alarm limits).
pub struct TagMetadata {
    /// Conventional unit symbol (e.g. `"bar"`, `"°C"`).
    pub unit_symbol: String,
    /// Lower end of the normal engineering range.
    pub range_low: f64,
    /// Upper end of the normal engineering range.
    pub range_high: f64,
    /// Optional low alarm limit.
    pub alarm_low: Option<f64>,
    /// Optional high alarm limit.
    pub alarm_high: Option<f64>,
}

impl TagMetadata {
    /// Creates tag metadata with unit symbol taken from `unit.symbol()`.
    pub fn new(unit: sp_kernel::UnitOfMeasure, range_low: f64, range_high: f64) -> Self {
        Self {
            unit_symbol: unit.symbol().to_owned(),
            range_low,
            range_high,
            alarm_low: None,
            alarm_high: None,
        }
    }

    /// Sets the low alarm limit.
    #[must_use]
    pub fn with_alarm_low(mut self, limit: f64) -> Self {
        self.alarm_low = Some(limit);
        self
    }

    /// Sets the high alarm limit.
    #[must_use]
    pub fn with_alarm_high(mut self, limit: f64) -> Self {
        self.alarm_high = Some(limit);
        self
    }

    fn as_dynamic_archetype(&self) -> DynamicArchetype {
        let mut archetype = DynamicArchetype::new(ARCHETYPE_TAG_METADATA)
            .with_component::<components::Text>(
                "unit_symbol",
                std::iter::once(self.unit_symbol.as_str()),
            )
            .with_component::<components::Scalar>("range_low", [self.range_low])
            .with_component::<components::Scalar>("range_high", [self.range_high]);

        if let Some(alarm_low) = self.alarm_low {
            archetype = archetype.with_component::<components::Scalar>("alarm_low", [alarm_low]);
        }
        if let Some(alarm_high) = self.alarm_high {
            archetype = archetype.with_component::<components::Scalar>("alarm_high", [alarm_high]);
        }

        archetype
    }
}

impl AsComponents for TagMetadata {
    fn as_serialized_batches(&self) -> Vec<SerializedComponentBatch> {
        self.as_dynamic_archetype().as_serialized_batches()
    }
}

#[cfg(test)]
mod tests {
    use re_sdk_types::AsComponents as _;

    use super::*;

    #[test]
    fn tag_metadata_batches() {
        let metadata = TagMetadata::new(sp_kernel::UnitOfMeasure::Bar, 0.0, 100.0);
        let batches = metadata.as_serialized_batches();
        assert!(!batches.is_empty());
    }
}
