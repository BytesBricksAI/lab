//! Domain layer: aggregates, value objects, events, and errors.

pub mod catalog;
pub mod equipment;
pub mod error;
pub mod events;
pub mod facility;
pub mod ids;
pub mod tag;

pub use catalog::AssetCatalog;
pub use equipment::{DesignBound, DesignSpec, Equipment, EquipmentKind};
pub use error::{AssetError, Result};
pub use events::{
    AlarmLimitsChanged, AreaAdded, AssetEvent, DesignSpecRevised, EquipmentCommissioned,
    FacilityDefined, TagDefined, UnitAdded,
};
pub use facility::{Area, Facility, ProcessUnit};
pub use ids::{AreaId, EquipmentId, FacilityId, UnitId};
pub use tag::{Tag, TagSpec};
