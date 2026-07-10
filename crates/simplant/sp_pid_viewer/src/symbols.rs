//! Embedded catalogue of Equinor `engineering-symbols` P&ID icons.
//!
//! The SVG sources live in `assets/symbols/` and are embedded into the binary
//! at compile time, so the viewer needs neither network nor filesystem access
//! to render them. The symbols are MIT-licensed by Equinor — see
//! `assets/symbols/LICENSE`.
//!
//! This module is deliberately free of `egui`/`re_*` dependencies: it only
//! owns the raw catalogue. Theming and rendering live in
//! [`crate::visualizer`].

/// One embedded P&ID symbol, identified by its Equinor id (e.g. `"PP007A"`).
#[derive(Debug, Clone, Copy)]
pub struct Symbol {
    /// Symbol id in the Equinor `engineering-symbols` library.
    pub id: &'static str,

    /// Raw SVG contents, embedded in the binary.
    pub svg: &'static [u8],
}

macro_rules! symbol {
    ($id:literal) => {
        Symbol {
            id: $id,
            svg: include_bytes!(concat!("../assets/symbols/", $id, ".svg")),
        }
    };
}

/// The full embedded catalogue, sorted by id (required by [`find`]).
pub const SYMBOLS: &[Symbol] = &[
    symbol!("IM005A"),
    symbol!("IM017A"),
    symbol!("IM017B"),
    symbol!("IM017C"),
    symbol!("LZ009A"),
    symbol!("ND0001"),
    symbol!("ND0002"),
    symbol!("ND0005"),
    symbol!("ND0005Option1"),
    symbol!("ND0011"),
    symbol!("ND0011Option1"),
    symbol!("ND0012"),
    symbol!("ND0012Option1"),
    symbol!("ND0014"),
    symbol!("ND0015"),
    symbol!("ND0016"),
    symbol!("ND0020"),
    symbol!("ND0020Option1"),
    symbol!("ND0023"),
    symbol!("ND0024"),
    symbol!("ND0025"),
    symbol!("PA007A"),
    symbol!("PA008A"),
    symbol!("PD001A"),
    symbol!("PP001A"),
    symbol!("PP002A"),
    symbol!("PP003A"),
    symbol!("PP004A"),
    symbol!("PP005A"),
    symbol!("PP006A"),
    symbol!("PP007A"),
    symbol!("PS005A"),
    symbol!("PS006A"),
    symbol!("PS007A"),
    symbol!("PT002A"),
    symbol!("PT002AOption1"),
    symbol!("PT005A"),
    symbol!("PT006A"),
    symbol!("PV005A"),
    symbol!("PV005AOption1"),
    symbol!("PV005AUmbrella"),
    symbol!("PV005AUmbrellaOption1"),
    symbol!("PV007B"),
    symbol!("PV007BOption1"),
    symbol!("PV016A"),
    symbol!("PV016AOption1"),
    symbol!("PV018A"),
    symbol!("PV018AOption1"),
    symbol!("PV019B"),
    symbol!("PV019BOption1"),
    symbol!("PV021A"),
    symbol!("PV022A"),
    symbol!("PV023A"),
    symbol!("PV023AOption1"),
    symbol!("STPL008"),
];

/// Looks up a symbol by its Equinor id.
pub fn find(id: &str) -> Option<&'static Symbol> {
    SYMBOLS
        .binary_search_by(|symbol| symbol.id.cmp(id))
        .ok()
        .map(|index| &SYMBOLS[index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_is_sorted_by_id() {
        // `find` relies on binary search; a mis-sorted entry would silently
        // make some symbols unreachable.
        assert!(SYMBOLS.windows(2).all(|pair| pair[0].id < pair[1].id));
    }

    #[test]
    fn every_asset_file_has_an_entry() {
        let assets_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/symbols");
        let mut missing = Vec::new();
        for entry in std::fs::read_dir(assets_dir).expect("assets/symbols must exist") {
            let path = entry.expect("readable dir entry").path();
            if path.extension().is_some_and(|ext| ext == "svg") {
                let id = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or_default()
                    .to_owned();
                if find(&id).is_none() {
                    missing.push(id);
                }
            }
        }
        assert!(
            missing.is_empty(),
            "SVG files without a catalogue entry: {missing:?}"
        );
    }

    #[test]
    fn entries_contain_svg_markup() {
        for symbol in SYMBOLS {
            let head = &symbol.svg[..symbol.svg.len().min(64)];
            assert!(
                head.starts_with(b"<svg"),
                "symbol {} does not start with <svg",
                symbol.id
            );
        }
    }

    #[test]
    fn find_misses_unknown_ids() {
        assert!(find("NOPE").is_none());
        assert!(find("").is_none());
    }
}
