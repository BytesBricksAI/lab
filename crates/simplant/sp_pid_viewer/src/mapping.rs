//! Maps the SimPlant asset model onto P&ID symbols.
//!
//! This table is *domain* knowledge — which icon represents which kind of
//! equipment — and it is expected to evolve: refine ids by inspecting the
//! SVGs in `assets/symbols/`, and extend it with DEXPI class names once the
//! DEXPI importer lands.

use sp_asset_model::EquipmentKind;

use crate::symbols::{self, Symbol};

/// Default Equinor symbol id for an equipment kind.
///
/// `None` means the kind has no dedicated icon: pipes are drawn as polylines,
/// and unknown equipment gets a labeled placeholder box in the canvas.
pub fn symbol_id_for(kind: EquipmentKind) -> Option<&'static str> {
    match kind {
        EquipmentKind::Vessel => Some("PV005A"),
        EquipmentKind::Tank => Some("PT002A"),
        EquipmentKind::Pump => Some("PP007A"),
        // Provisional: ND* is the fittings/valves family — review visually.
        EquipmentKind::Valve => Some("ND0001"),
        // No icon: piping is rendered as polylines; the vendored set has no
        // confidently-identified heat exchanger yet, and better an honest
        // placeholder than a wrong symbol on a P&ID.
        EquipmentKind::HeatExchanger | EquipmentKind::Pipe | EquipmentKind::Other => None,
    }
}

/// Resolves the embedded [`Symbol`] for an equipment kind, if it has one.
pub fn symbol_for(kind: EquipmentKind) -> Option<&'static Symbol> {
    symbol_id_for(kind).and_then(symbols::find)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_KINDS: [EquipmentKind; 7] = [
        EquipmentKind::Vessel,
        EquipmentKind::Tank,
        EquipmentKind::Pump,
        EquipmentKind::HeatExchanger,
        EquipmentKind::Valve,
        EquipmentKind::Pipe,
        EquipmentKind::Other,
    ];

    #[test]
    fn every_mapped_id_exists_in_the_catalogue() {
        for kind in ALL_KINDS {
            if let Some(id) = symbol_id_for(kind) {
                assert!(
                    symbols::find(id).is_some(),
                    "{kind:?} maps to {id:?}, which is not in the catalogue"
                );
            }
        }
    }

    #[test]
    fn symbol_for_agrees_with_symbol_id_for() {
        for kind in ALL_KINDS {
            assert_eq!(
                symbol_for(kind).map(|symbol| symbol.id),
                symbol_id_for(kind)
            );
        }
    }
}
