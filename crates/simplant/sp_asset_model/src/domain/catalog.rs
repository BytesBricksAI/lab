//! Asset catalog read model with referential-integrity boundary.

use sp_kernel::TagId;

use crate::domain::equipment::Equipment;
use crate::domain::error::{AssetError, Result};
use crate::domain::facility::Facility;
use crate::domain::ids::EquipmentId;
use crate::domain::tag::Tag;

/// Complete asset catalog: facility hierarchy, equipment, and tags.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetCatalog {
    facility: Facility,
    equipment: Vec<Equipment>,
    tags: Vec<Tag>,
}

impl AssetCatalog {
    /// Assembles a catalog, validating referential integrity and uniqueness.
    pub fn assemble(facility: Facility, equipment: Vec<Equipment>, tags: Vec<Tag>) -> Result<Self> {
        let catalog = Self {
            facility,
            equipment,
            tags,
        };
        catalog.validate()?;
        Ok(catalog)
    }

    /// Re-checks referential integrity (e.g. for CLI validation).
    pub fn validate(&self) -> Result<()> {
        let mut seen_equipment = std::collections::HashSet::new();
        for eq in &self.equipment {
            if !seen_equipment.insert(eq.id().clone()) {
                return Err(AssetError::DuplicateEquipment(eq.id().as_str().to_owned()));
            }
            if !self.facility.has_unit(eq.unit()) {
                return Err(AssetError::UnitNotFound(eq.unit().as_str().to_owned()));
            }
        }

        let mut seen_tags = std::collections::HashSet::new();
        for tag in &self.tags {
            if !seen_tags.insert(tag.id().clone()) {
                return Err(AssetError::DuplicateTag(tag.id().as_str().to_owned()));
            }
            if self.equipment_by_id(tag.equipment()).is_none() {
                return Err(AssetError::EquipmentNotFound(
                    tag.equipment().as_str().to_owned(),
                ));
            }
        }

        Ok(())
    }

    /// Returns the facility hierarchy.
    pub fn facility(&self) -> &Facility {
        &self.facility
    }

    /// Returns all commissioned equipment.
    pub fn equipment(&self) -> &[Equipment] {
        &self.equipment
    }

    /// Returns all defined tags.
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Looks up a tag by identifier.
    pub fn tag(&self, id: &TagId) -> Option<&Tag> {
        self.tags.iter().find(|t| t.id() == id)
    }

    /// Looks up equipment by identifier.
    pub fn equipment_by_id(&self, id: &EquipmentId) -> Option<&Equipment> {
        self.equipment.iter().find(|e| e.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::equipment::{DesignSpec, Equipment, EquipmentKind};
    use crate::domain::facility::Facility;
    use crate::domain::ids::{AreaId, EquipmentId, FacilityId, UnitId};
    use crate::domain::tag::{Tag, TagSpec};
    use sp_kernel::{EngineeringRange, TagId, UnitOfMeasure};

    fn sample_facility_with_unit() -> Facility {
        let (mut facility, _) = Facility::define(FacilityId::new("FAC-01").unwrap(), "Refinery");
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
        facility
    }

    #[test]
    fn assemble_rejects_equipment_with_unknown_unit() {
        let facility = sample_facility_with_unit();
        let (equipment, _) = Equipment::commission(
            EquipmentId::new("EQ-101").unwrap(),
            UnitId::new("UNIT-999").unwrap(),
            "Separator",
            EquipmentKind::Vessel,
            DesignSpec::new(None, None).unwrap(),
        )
        .unwrap();
        let err = AssetCatalog::assemble(facility, vec![equipment], vec![]).unwrap_err();
        assert!(matches!(err, AssetError::UnitNotFound(_)));
    }

    #[test]
    fn assemble_rejects_tag_with_unknown_equipment() {
        let facility = sample_facility_with_unit();
        let spec = TagSpec {
            id: TagId::new("PT-1101").unwrap(),
            equipment: EquipmentId::new("EQ-MISSING").unwrap(),
            description: "Pressure".to_owned(),
            unit: UnitOfMeasure::Bar,
            range: EngineeringRange::new(0.0, 100.0, UnitOfMeasure::Bar).unwrap(),
            alarms: None,
        };
        let (tag, _) = Tag::define(spec).unwrap();
        let err = AssetCatalog::assemble(facility, vec![], vec![tag]).unwrap_err();
        assert!(matches!(err, AssetError::EquipmentNotFound(_)));
    }

    #[test]
    fn assemble_happy_path() {
        let facility = sample_facility_with_unit();
        let (equipment, _) = Equipment::commission(
            EquipmentId::new("EQ-101").unwrap(),
            UnitId::new("UNIT-100").unwrap(),
            "Separator",
            EquipmentKind::Vessel,
            DesignSpec::new(None, None).unwrap(),
        )
        .unwrap();
        let spec = TagSpec {
            id: TagId::new("PT-1101").unwrap(),
            equipment: EquipmentId::new("EQ-101").unwrap(),
            description: "Pressure".to_owned(),
            unit: UnitOfMeasure::Bar,
            range: EngineeringRange::new(0.0, 100.0, UnitOfMeasure::Bar).unwrap(),
            alarms: None,
        };
        let (tag, _) = Tag::define(spec).unwrap();
        let catalog = AssetCatalog::assemble(facility, vec![equipment], vec![tag]).unwrap();
        assert_eq!(catalog.equipment().len(), 1);
        assert_eq!(catalog.tags().len(), 1);
    }
}
