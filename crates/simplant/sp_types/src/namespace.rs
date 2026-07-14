//! `simplant.*` namespace constants for archetypes and components.

/// Archetype name for time-series process variable samples.
pub const ARCHETYPE_PROCESS_VARIABLE: &str = "simplant.archetypes.ProcessVariable";

/// Archetype name for static tag metadata.
pub const ARCHETYPE_TAG_METADATA: &str = "simplant.archetypes.TagMetadata";

/// Archetype name for equipment symbols placed on a P&ID diagram.
pub const ARCHETYPE_PID_SYMBOL: &str = "simplant.archetypes.PidSymbol";

/// Archetype name for process lines (pipes) on a P&ID diagram.
pub const ARCHETYPE_PID_PIPE: &str = "simplant.archetypes.PidPipe";

/// Component type name for OPC UA-style quality codes.
pub const COMPONENT_QUALITY: &str = "simplant.components.Quality";

/// Builds a fully-qualified component field name within an archetype.
pub fn field(archetype: &str, field: &str) -> String {
    format!("{archetype}:{field}")
}
