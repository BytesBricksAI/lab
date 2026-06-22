//! Central branding constants for SimPlant-Lab (fork of Rerun).

/// User-visible product name (no "Viewer" suffix).
pub const PRODUCT_NAME: &str = "SimPlant-Lab";

/// Lowercase slug for paths, CLI, and technical identifiers.
pub const PRODUCT_NAME_LOWERCASE: &str = "simplant-lab";

/// Recording file extension (unchanged for compatibility with `.rrd`).
pub const FILE_EXTENSION_RRD: &str = "rrd";

/// Default documentation / help base URL.
pub const DEFAULT_DOCS_URL: &str = "https://github.com/SimPlant/SimPlant-v2";

/// Optional community chat URL shown in the UI.
pub const DISCORD_URL: Option<&str> = None;

/// Default homepage when no project-specific URL is configured.
pub const DEFAULT_HOMEPAGE_URL: &str = "https://github.com/SimPlant/SimPlant-v2";
