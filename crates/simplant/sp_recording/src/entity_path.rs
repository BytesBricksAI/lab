//! Entity path conventions for `SimPlant` recordings.

use sp_kernel::TagId;

/// Returns the flat entity path for a process tag: `tags/<id>`.
///
/// Leading slashes on the tag id are stripped so `/PT-1101` and `PT-1101` map to the same path.
///
/// F1 uses this flat layout. A full hierarchical path (`/site/area/unit/equipment/tag`) is a
/// future improvement that requires the asset catalog to be available inside the recorder.
pub fn tag_entity_path(tag: &TagId) -> String {
    format!("tags/{}", tag.as_str().trim_start_matches('/'))
}

/// Timeline name for plant-side timestamps (distinct from the recording wall-clock).
pub const PLANT_TIME: &str = "plant_time";

/// Entity path for acquisition control-plane events.
pub const EVENTS_PATH: &str = "events/acquisition";
