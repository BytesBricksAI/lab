//! Domain events for the asset-model control plane.

use serde::{Deserialize, Serialize};

use crate::domain::ids::{AreaId, EquipmentId, FacilityId, UnitId};
use sp_kernel::TagId;

/// A facility was defined.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityDefined {
    /// Facility identifier.
    pub facility: FacilityId,
    /// Human-readable facility name.
    pub name: String,
}

/// An area was added to a facility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AreaAdded {
    /// Parent facility identifier.
    pub facility: FacilityId,
    /// New area identifier.
    pub area: AreaId,
}

/// A process unit was added to an area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitAdded {
    /// Parent area identifier.
    pub area: AreaId,
    /// New process unit identifier.
    pub unit: UnitId,
}

/// Equipment was commissioned on a process unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentCommissioned {
    /// Commissioned equipment identifier.
    pub equipment: EquipmentId,
    /// Host process unit identifier.
    pub unit: UnitId,
}

/// Equipment design specification was revised.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignSpecRevised {
    /// Equipment whose design was revised.
    pub equipment: EquipmentId,
}

/// A process tag was defined on equipment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagDefined {
    /// Tag identifier.
    pub tag: TagId,
    /// Host equipment identifier.
    pub equipment: EquipmentId,
}

/// Alarm limits were changed on a tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlarmLimitsChanged {
    /// Tag whose alarm limits changed.
    pub tag: TagId,
}

/// All asset-model domain events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetEvent {
    /// Facility defined.
    FacilityDefined(FacilityDefined),
    /// Area added.
    AreaAdded(AreaAdded),
    /// Process unit added.
    UnitAdded(UnitAdded),
    /// Equipment commissioned.
    EquipmentCommissioned(EquipmentCommissioned),
    /// Design specification revised.
    DesignSpecRevised(DesignSpecRevised),
    /// Tag defined.
    TagDefined(TagDefined),
    /// Alarm limits changed.
    AlarmLimitsChanged(AlarmLimitsChanged),
}
