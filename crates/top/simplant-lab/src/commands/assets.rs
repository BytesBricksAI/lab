use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::Context as _;
use sp_asset_model::{AssetCatalog, AssetCatalogPort as _, TomlCatalogRepository};

/// Load a TOML asset catalog and print its facilities, equipment, and tags.
#[derive(Debug, Clone, clap::Parser)]
pub struct AssetsCommand {
    /// Path to the TOML asset catalog.
    #[arg(long = "catalog")]
    catalog: PathBuf,
}

impl AssetsCommand {
    pub fn run(self) -> anyhow::Result<()> {
        let catalog = load_catalog(&self.catalog)?;
        println!("{}", format_catalog(&catalog));
        Ok(())
    }
}

fn load_catalog(path: &std::path::Path) -> anyhow::Result<AssetCatalog> {
    let catalog = TomlCatalogRepository::new(path)
        .load_catalog()
        .map_err(|err| anyhow::anyhow!(err.to_string()))
        .with_context(|| format!("loading catalog from {}", path.display()))?;

    catalog
        .validate()
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    Ok(catalog)
}

/// Format an asset catalog for human-readable CLI output.
pub(crate) fn format_catalog(catalog: &AssetCatalog) -> String {
    let mut out = String::new();

    let facility = catalog.facility();
    writeln!(
        out,
        "Facility: {} ({})",
        facility.name(),
        facility.id().as_str()
    )
    .ok();

    writeln!(out).ok();
    writeln!(out, "Areas:").ok();
    for area in facility.areas() {
        writeln!(out, "  {} ({})", area.name(), area.id().as_str()).ok();
        for unit in area.units() {
            writeln!(out, "    Unit: {} ({})", unit.name(), unit.id().as_str()).ok();
        }
    }

    writeln!(out).ok();
    writeln!(out, "Equipment ({}):", catalog.equipment().len()).ok();
    for equipment in catalog.equipment() {
        writeln!(
            out,
            "  {} ({}) — unit {} — {:?}",
            equipment.name(),
            equipment.id().as_str(),
            equipment.unit().as_str(),
            equipment.kind()
        )
        .ok();
    }

    writeln!(out).ok();
    writeln!(out, "Tags ({}):", catalog.tags().len()).ok();
    for tag in catalog.tags() {
        writeln!(
            out,
            "  {} — equipment {} — {} [{:?}]",
            tag.id().as_str(),
            tag.equipment().as_str(),
            tag.description(),
            tag.unit()
        )
        .ok();
    }

    out
}

#[cfg(test)]
mod tests {
    use sp_asset_model::{
        AreaId, AssetCatalog, DesignSpec, Equipment, EquipmentId, EquipmentKind, Facility,
        FacilityId, Tag, TagSpec, UnitId,
    };
    use sp_kernel::{EngineeringRange, TagId, UnitOfMeasure};

    use super::format_catalog;

    fn sample_catalog() -> AssetCatalog {
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

        AssetCatalog::assemble(facility, vec![equipment], vec![tag]).unwrap()
    }

    #[test]
    fn format_catalog_includes_facility_equipment_and_tags() {
        let text = format_catalog(&sample_catalog());

        assert!(text.contains("Facility: Refinery (FAC-01)"));
        assert!(text.contains("Areas:"));
        assert!(text.contains("Crude (AREA-A)"));
        assert!(text.contains("Unit: CDU (UNIT-100)"));
        assert!(text.contains("Equipment (1):"));
        assert!(text.contains("Separator (EQ-101)"));
        assert!(text.contains("Tags (1):"));
        assert!(text.contains("PT-1101 — equipment EQ-101 — Pressure"));
    }
}
