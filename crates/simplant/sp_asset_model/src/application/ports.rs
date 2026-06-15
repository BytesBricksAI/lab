use crate::domain::catalog::AssetCatalog;
use crate::domain::error::Result;

/// Driven port: load/save the asset catalog from a persistent, version-controlled source.
pub trait AssetCatalogPort {
    /// Loads the asset catalog from persistent storage.
    fn load_catalog(&self) -> Result<AssetCatalog>;

    /// Persists the asset catalog.
    fn save_catalog(&self, catalog: &AssetCatalog) -> Result<()>;
}
