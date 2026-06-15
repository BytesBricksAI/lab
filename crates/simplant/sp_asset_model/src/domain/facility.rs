//! Facility aggregate: plant hierarchy (facility → area → process unit).

use crate::domain::error::{AssetError, Result};
use crate::domain::events::{AreaAdded, FacilityDefined, UnitAdded};
use crate::domain::ids::{AreaId, FacilityId, UnitId};

/// A process unit within an area.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessUnit {
    id: UnitId,
    name: String,
}

impl ProcessUnit {
    /// Returns the process unit identifier.
    pub fn id(&self) -> &UnitId {
        &self.id
    }

    /// Returns the process unit name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// An operational area within a facility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Area {
    id: AreaId,
    name: String,
    units: Vec<ProcessUnit>,
}

impl Area {
    /// Returns the area identifier.
    pub fn id(&self) -> &AreaId {
        &self.id
    }

    /// Returns the area name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the process units in this area.
    pub fn units(&self) -> &[ProcessUnit] {
        &self.units
    }
}

/// Root aggregate modeling the plant hierarchy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facility {
    id: FacilityId,
    name: String,
    areas: Vec<Area>,
}

impl Facility {
    /// Defines a new facility with no areas.
    pub fn define(id: FacilityId, name: impl Into<String>) -> (Self, FacilityDefined) {
        let name = name.into();
        let event = FacilityDefined {
            facility: id.clone(),
            name: name.clone(),
        };
        let facility = Self {
            id,
            name,
            areas: Vec::new(),
        };
        (facility, event)
    }

    /// Adds an area to the facility, rejecting duplicate area identifiers.
    pub fn add_area(&mut self, id: AreaId, name: impl Into<String>) -> Result<AreaAdded> {
        if self.has_area(&id) {
            return Err(AssetError::DuplicateArea(id.as_str().to_owned()));
        }
        self.areas.push(Area {
            id: id.clone(),
            name: name.into(),
            units: Vec::new(),
        });
        Ok(AreaAdded {
            facility: self.id.clone(),
            area: id,
        })
    }

    /// Adds a process unit to an existing area; unit identifiers must be unique facility-wide.
    pub fn add_unit(
        &mut self,
        area: &AreaId,
        unit: UnitId,
        name: impl Into<String>,
    ) -> Result<UnitAdded> {
        if self.has_unit(&unit) {
            return Err(AssetError::DuplicateUnit(unit.as_str().to_owned()));
        }
        let area_entry = self
            .areas
            .iter_mut()
            .find(|a| a.id == *area)
            .ok_or_else(|| AssetError::AreaNotFound(area.as_str().to_owned()))?;
        area_entry.units.push(ProcessUnit {
            id: unit.clone(),
            name: name.into(),
        });
        Ok(UnitAdded {
            area: area.clone(),
            unit,
        })
    }

    /// Returns `true` if the facility contains the given process unit.
    pub fn has_unit(&self, unit: &UnitId) -> bool {
        self.areas
            .iter()
            .flat_map(|a| a.units.iter())
            .any(|u| &u.id == unit)
    }

    /// Returns `true` if the facility contains the given area.
    pub fn has_area(&self, area: &AreaId) -> bool {
        self.areas.iter().any(|a| &a.id == area)
    }

    /// Returns all areas in the facility.
    pub fn areas(&self) -> &[Area] {
        &self.areas
    }

    /// Returns the facility identifier.
    pub fn id(&self) -> &FacilityId {
        &self.id
    }

    /// Returns the facility name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_facility() -> Facility {
        let (facility, _) = Facility::define(FacilityId::new("FAC-01").unwrap(), "Refinery");
        facility
    }

    #[test]
    fn happy_path_add_area_and_unit() {
        let mut facility = sample_facility();
        facility
            .add_area(AreaId::new("AREA-A").unwrap(), "Crude")
            .unwrap();
        facility
            .add_unit(
                &AreaId::new("AREA-A").unwrap(),
                UnitId::new("UNIT-100").unwrap(),
                "CDU",
            )
            .unwrap();
        assert!(facility.has_area(&AreaId::new("AREA-A").unwrap()));
        assert!(facility.has_unit(&UnitId::new("UNIT-100").unwrap()));
    }

    #[test]
    fn duplicate_area_is_rejected() {
        let mut facility = sample_facility();
        facility
            .add_area(AreaId::new("AREA-A").unwrap(), "Crude")
            .unwrap();
        let err = facility
            .add_area(AreaId::new("AREA-A").unwrap(), "Duplicate")
            .unwrap_err();
        assert!(matches!(err, AssetError::DuplicateArea(_)));
    }

    #[test]
    fn add_unit_to_missing_area_returns_not_found() {
        let mut facility = sample_facility();
        let err = facility
            .add_unit(
                &AreaId::new("MISSING").unwrap(),
                UnitId::new("UNIT-100").unwrap(),
                "CDU",
            )
            .unwrap_err();
        assert!(matches!(err, AssetError::AreaNotFound(_)));
    }

    #[test]
    fn duplicate_unit_globally_is_rejected() {
        let mut facility = sample_facility();
        facility
            .add_area(AreaId::new("AREA-A").unwrap(), "Crude")
            .unwrap();
        facility
            .add_area(AreaId::new("AREA-B").unwrap(), "Reform")
            .unwrap();
        facility
            .add_unit(
                &AreaId::new("AREA-A").unwrap(),
                UnitId::new("UNIT-100").unwrap(),
                "CDU",
            )
            .unwrap();
        let err = facility
            .add_unit(
                &AreaId::new("AREA-B").unwrap(),
                UnitId::new("UNIT-100").unwrap(),
                "Reformer",
            )
            .unwrap_err();
        assert!(matches!(err, AssetError::DuplicateUnit(_)));
    }
}
