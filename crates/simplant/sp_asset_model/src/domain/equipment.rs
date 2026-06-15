//! Equipment aggregate: commissioned plant equipment with design limits.

use serde::{Deserialize, Serialize};
use sp_kernel::{Dimension, UnitOfMeasure};

use crate::domain::error::{AssetError, Result};
use crate::domain::events::{DesignSpecRevised, EquipmentCommissioned};
use crate::domain::ids::{EquipmentId, UnitId};

/// Kind of process equipment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentKind {
    /// Pressure vessel.
    Vessel,
    /// Storage tank.
    Tank,
    /// Rotating pump.
    Pump,
    /// Heat exchanger.
    HeatExchanger,
    /// Control or isolation valve.
    Valve,
    /// Piping segment.
    Pipe,
    /// Other equipment type.
    Other,
}

/// A single design-bound value with unit.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DesignBound {
    value: f64,
    unit: UnitOfMeasure,
}

impl DesignBound {
    /// Creates a design bound, rejecting non-finite values.
    pub fn new(value: f64, unit: UnitOfMeasure) -> Result<Self> {
        if !value.is_finite() {
            return Err(AssetError::InvalidDesignSpec(
                "design bound value must be finite".to_owned(),
            ));
        }
        Ok(Self { value, unit })
    }

    /// Returns the bound value.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Returns the bound unit.
    pub fn unit(&self) -> UnitOfMeasure {
        self.unit
    }
}

/// Equipment design specification (maximum pressure and temperature).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DesignSpec {
    max_pressure: Option<DesignBound>,
    max_temperature: Option<DesignBound>,
}

impl DesignSpec {
    /// Creates a design spec, validating dimensional consistency of each bound.
    pub fn new(
        max_pressure: Option<DesignBound>,
        max_temperature: Option<DesignBound>,
    ) -> Result<Self> {
        if let Some(bound) = max_pressure
            && bound.unit().dimension() != Dimension::Pressure
        {
            return Err(AssetError::InvalidDesignSpec(
                "max_pressure unit must be a pressure dimension".to_owned(),
            ));
        }
        if let Some(bound) = max_temperature
            && bound.unit().dimension() != Dimension::Temperature
        {
            return Err(AssetError::InvalidDesignSpec(
                "max_temperature unit must be a temperature dimension".to_owned(),
            ));
        }
        Ok(Self {
            max_pressure,
            max_temperature,
        })
    }

    /// Returns the maximum pressure design bound, if set.
    pub fn max_pressure(&self) -> Option<DesignBound> {
        self.max_pressure
    }

    /// Returns the maximum temperature design bound, if set.
    pub fn max_temperature(&self) -> Option<DesignBound> {
        self.max_temperature
    }
}

/// Commissioned equipment on a process unit.
#[derive(Debug, Clone, PartialEq)]
pub struct Equipment {
    id: EquipmentId,
    unit: UnitId,
    name: String,
    kind: EquipmentKind,
    design: DesignSpec,
}

impl Equipment {
    /// Commissions new equipment on a process unit.
    #[expect(clippy::unnecessary_wraps)]
    pub fn commission(
        id: EquipmentId,
        unit: UnitId,
        name: impl Into<String>,
        kind: EquipmentKind,
        design: DesignSpec,
    ) -> Result<(Self, EquipmentCommissioned)> {
        let equipment = Self {
            id: id.clone(),
            unit: unit.clone(),
            name: name.into(),
            kind,
            design,
        };
        let event = EquipmentCommissioned {
            equipment: id,
            unit,
        };
        Ok((equipment, event))
    }

    /// Revises the equipment design specification.
    pub fn revise_design(&mut self, design: DesignSpec) -> DesignSpecRevised {
        self.design = design;
        DesignSpecRevised {
            equipment: self.id.clone(),
        }
    }

    /// Returns the equipment identifier.
    pub fn id(&self) -> &EquipmentId {
        &self.id
    }

    /// Returns the host process unit identifier.
    pub fn unit(&self) -> &UnitId {
        &self.unit
    }

    /// Returns the equipment name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the equipment kind.
    pub fn kind(&self) -> EquipmentKind {
        self.kind
    }

    /// Returns the design specification.
    pub fn design(&self) -> DesignSpec {
        self.design
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn design_spec_rejects_pressure_with_temperature_unit() {
        let bound = DesignBound::new(100.0, UnitOfMeasure::DegreeCelsius).unwrap();
        let err = DesignSpec::new(Some(bound), None).unwrap_err();
        assert!(matches!(err, AssetError::InvalidDesignSpec(_)));
    }

    #[test]
    fn commission_happy_path() {
        let design = DesignSpec::new(
            Some(DesignBound::new(10.0, UnitOfMeasure::Bar).unwrap()),
            None,
        )
        .unwrap();
        let (equipment, event) = Equipment::commission(
            EquipmentId::new("EQ-101").unwrap(),
            UnitId::new("UNIT-100").unwrap(),
            "Separator",
            EquipmentKind::Vessel,
            design,
        )
        .unwrap();
        assert_eq!(equipment.id().as_str(), "EQ-101");
        assert_eq!(event.equipment.as_str(), "EQ-101");
    }
}
