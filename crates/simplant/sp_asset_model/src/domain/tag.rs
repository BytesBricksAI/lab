//! Tag aggregate: process variable definition with engineering range and alarms.

use serde::{Deserialize, Serialize};
use sp_kernel::{AlarmLimits, EngineeringRange, TagId, UnitOfMeasure};

use crate::domain::error::{AssetError, Result};
use crate::domain::events::{AlarmLimitsChanged, TagDefined};
use crate::domain::ids::EquipmentId;

/// Specification used to define a new process tag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagSpec {
    /// Tag identifier.
    pub id: TagId,
    /// Host equipment identifier.
    pub equipment: EquipmentId,
    /// Human-readable description.
    pub description: String,
    /// Engineering unit of measure.
    pub unit: UnitOfMeasure,
    /// Inclusive engineering range.
    pub range: EngineeringRange,
    /// Optional alarm limits.
    pub alarms: Option<AlarmLimits>,
}

/// A defined process tag on equipment.
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    id: TagId,
    equipment: EquipmentId,
    description: String,
    unit: UnitOfMeasure,
    range: EngineeringRange,
    alarms: Option<AlarmLimits>,
}

impl Tag {
    /// Defines a new tag, enforcing unit consistency and alarm range invariants.
    pub fn define(spec: TagSpec) -> Result<(Self, TagDefined)> {
        if spec.range.unit() != spec.unit {
            return Err(AssetError::UnitMismatch {
                expected: format!("{:?}", spec.unit),
                found: format!("{:?}", spec.range.unit()),
            });
        }
        if let Some(alarms) = spec.alarms {
            if alarms.unit() != spec.unit {
                return Err(AssetError::UnitMismatch {
                    expected: format!("{:?}", spec.unit),
                    found: format!("{:?}", alarms.unit()),
                });
            }
            if !alarms.within(&spec.range) {
                return Err(AssetError::AlarmsOutOfRange);
            }
        }
        let tag = Self {
            id: spec.id.clone(),
            equipment: spec.equipment.clone(),
            description: spec.description,
            unit: spec.unit,
            range: spec.range,
            alarms: spec.alarms,
        };
        let event = TagDefined {
            tag: spec.id,
            equipment: spec.equipment,
        };
        Ok((tag, event))
    }

    /// Changes alarm limits, revalidating unit and range invariants.
    pub fn change_alarm_limits(&mut self, alarms: AlarmLimits) -> Result<AlarmLimitsChanged> {
        if alarms.unit() != self.unit {
            return Err(AssetError::UnitMismatch {
                expected: format!("{:?}", self.unit),
                found: format!("{:?}", alarms.unit()),
            });
        }
        if !alarms.within(&self.range) {
            return Err(AssetError::AlarmsOutOfRange);
        }
        self.alarms = Some(alarms);
        Ok(AlarmLimitsChanged {
            tag: self.id.clone(),
        })
    }

    /// Returns the tag identifier.
    pub fn id(&self) -> &TagId {
        &self.id
    }

    /// Returns the host equipment identifier.
    pub fn equipment(&self) -> &EquipmentId {
        &self.equipment
    }

    /// Returns the engineering unit.
    pub fn unit(&self) -> UnitOfMeasure {
        self.unit
    }

    /// Returns the engineering range.
    pub fn range(&self) -> EngineeringRange {
        self.range
    }

    /// Returns the alarm limits, if set.
    pub fn alarms(&self) -> Option<AlarmLimits> {
        self.alarms
    }

    /// Returns the tag description.
    pub fn description(&self) -> &str {
        &self.description
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ids::EquipmentId;

    fn base_spec(unit: UnitOfMeasure, range_unit: UnitOfMeasure) -> TagSpec {
        TagSpec {
            id: TagId::new("PT-1101").unwrap(),
            equipment: EquipmentId::new("EQ-101").unwrap(),
            description: "Column pressure".to_owned(),
            unit,
            range: EngineeringRange::new(0.0, 100.0, range_unit).unwrap(),
            alarms: None,
        }
    }

    #[test]
    fn define_rejects_unit_mismatch_with_range() {
        let err = Tag::define(base_spec(UnitOfMeasure::Bar, UnitOfMeasure::Psi)).unwrap_err();
        assert!(matches!(err, AssetError::UnitMismatch { .. }));
    }

    #[test]
    fn define_rejects_alarms_out_of_range() {
        let mut spec = base_spec(UnitOfMeasure::Bar, UnitOfMeasure::Bar);
        spec.alarms = Some(
            AlarmLimits::new(
                Some(10.0),
                Some(20.0),
                Some(80.0),
                Some(110.0),
                UnitOfMeasure::Bar,
            )
            .unwrap(),
        );
        let err = Tag::define(spec).unwrap_err();
        assert_eq!(err, AssetError::AlarmsOutOfRange);
    }

    #[test]
    fn define_happy_path() {
        let mut spec = base_spec(UnitOfMeasure::Bar, UnitOfMeasure::Bar);
        spec.alarms = Some(
            AlarmLimits::new(
                Some(10.0),
                Some(20.0),
                Some(80.0),
                Some(90.0),
                UnitOfMeasure::Bar,
            )
            .unwrap(),
        );
        let (tag, event) = Tag::define(spec).unwrap();
        assert_eq!(tag.id().as_str(), "PT-1101");
        assert_eq!(event.tag.as_str(), "PT-1101");
        assert!(tag.alarms().is_some());
    }

    #[test]
    fn change_alarm_limits_revalidates() {
        let spec = base_spec(UnitOfMeasure::Bar, UnitOfMeasure::Bar);
        let (mut tag, _) = Tag::define(spec).unwrap();
        let bad = AlarmLimits::new(
            Some(10.0),
            Some(20.0),
            Some(80.0),
            Some(110.0),
            UnitOfMeasure::Bar,
        )
        .unwrap();
        let err = tag.change_alarm_limits(bad).unwrap_err();
        assert_eq!(err, AssetError::AlarmsOutOfRange);

        let good = AlarmLimits::new(
            Some(10.0),
            Some(20.0),
            Some(80.0),
            Some(90.0),
            UnitOfMeasure::Bar,
        )
        .unwrap();
        tag.change_alarm_limits(good).unwrap();
        assert!(tag.alarms().is_some());
    }
}
