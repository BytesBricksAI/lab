//! Public API surface for `sp_asset_model`.

pub use crate::application::ports::AssetCatalogPort;
pub use crate::domain::{
    AlarmLimitsChanged, Area, AreaAdded, AreaId, AssetCatalog, AssetError, AssetEvent, DesignBound,
    DesignSpec, DesignSpecRevised, Equipment, EquipmentCommissioned, EquipmentId, EquipmentKind,
    Facility, FacilityDefined, FacilityId, ProcessUnit, Result, Tag, TagDefined, TagSpec,
    UnitAdded, UnitId,
};
pub use crate::infrastructure::toml_catalog::TomlCatalogRepository;
