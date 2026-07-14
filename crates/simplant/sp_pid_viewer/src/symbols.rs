//! Embedded catalogue of Equinor `engineering-symbols` P&ID icons.
//!
//! The SVG sources live in `assets/symbols/` and are embedded into the binary
//! at compile time, so the viewer needs neither network nor filesystem access
//! to render them. The symbols are MIT-licensed by Equinor — see
//! `assets/symbols/LICENSE`.
//!
//! Besides the raw SVG bytes, this module owns the *drawing metadata* the
//! Equinor sources embed: the viewBox extents and the connection points
//! (`annotation-connector-<index>-<degrees>` circles). [`glyph_rect`] and
//! [`connector_point`] are the single source of truth for mapping that
//! metadata into diagram coordinates — the canvas draws through them and the
//! Python bindings compute pipe anchors through them, so they cannot drift.
//!
//! This module is deliberately free of `egui`/`re_*` dependencies: it only
//! owns the catalogue and its geometry. Theming and rendering live in
//! [`crate::visualizer`].

use std::sync::OnceLock;

/// Whether a symbol depicts process equipment or an ISA-style instrument.
///
/// The P&ID canvas draws instrument tags *inside* the bubble (ISA-5.1 style)
/// and equipment tags under the symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// Vessels, tanks, pumps, valves, fittings…
    Equipment,

    /// Instrument bubbles and logic blocks (`IM*`, `LZ*`).
    Instrument,
}

/// One embedded P&ID symbol, identified by its Equinor id (e.g. `"PP007A"`).
#[derive(Debug, Clone, Copy)]
pub struct Symbol {
    /// Symbol id in the Equinor `engineering-symbols` library.
    pub id: &'static str,

    /// Equipment vs instrument, decides how the canvas places the tag.
    pub kind: SymbolKind,

    /// Raw SVG contents, embedded in the binary.
    pub svg: &'static [u8],
}

/// A connection point declared inside an Equinor SVG as
/// `<circle id="annotation-connector-<index>-<degrees>" cx=… cy=…/>`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Connector {
    /// 1-based index within the symbol, stable across the Equinor library.
    pub index: u8,

    /// Compass direction a line approaches from, in degrees
    /// (0 = top/north, 90 = right/east, 180 = bottom, 270 = left).
    pub direction_deg: u16,

    /// Position in viewBox coordinates (y down).
    pub pos: [f32; 2],
}

/// Parsed drawing metadata of one symbol.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolMeta {
    /// viewBox width/height; the vendored set always has a `0 0` origin
    /// (enforced by the parser).
    pub view_box: [f32; 2],

    /// Declared connection points. A few symbols have none.
    pub connectors: Vec<Connector>,
}

impl Symbol {
    /// Parsed viewBox + connectors, cached once for the whole catalogue.
    ///
    /// `None` if the SVG deviates from the Equinor format; the tests pin
    /// every vendored asset to parse.
    pub fn meta(&self) -> Option<&'static SymbolMeta> {
        static METAS: OnceLock<Vec<Option<SymbolMeta>>> = OnceLock::new();
        let metas = METAS.get_or_init(|| {
            SYMBOLS
                .iter()
                .map(|symbol| parse_meta(symbol.svg))
                .collect()
        });
        let index = SYMBOLS
            .binary_search_by(|symbol| symbol.id.cmp(self.id))
            .ok()?;
        metas[index].as_ref()
    }

    /// Diagram position of connector `index` when this symbol is drawn
    /// centered at `center` within a box of half-extents `half_size`.
    pub fn connector_point(
        &self,
        center: [f32; 2],
        half_size: [f32; 2],
        index: u8,
    ) -> Option<[f32; 2]> {
        connector_point(center, half_size, self.meta()?, index)
    }
}

/// Rect the glyph actually covers when drawn centered in a `2 × half_size` box.
///
/// The largest viewBox-aspect rect that fits, centered — any excess is
/// letterboxed. Returned as `(min, size)` in diagram coordinates.
pub fn glyph_rect(
    center: [f32; 2],
    half_size: [f32; 2],
    view_box: [f32; 2],
) -> ([f32; 2], [f32; 2]) {
    let scale = (half_size[0] * 2.0 / view_box[0]).min(half_size[1] * 2.0 / view_box[1]);
    let size = [view_box[0] * scale, view_box[1] * scale];
    ([center[0] - size[0] / 2.0, center[1] - size[1] / 2.0], size)
}

/// Diagram position of connector `index`, mapped through [`glyph_rect`] —
/// the exact point the canvas renders that connector at.
pub fn connector_point(
    center: [f32; 2],
    half_size: [f32; 2],
    meta: &SymbolMeta,
    index: u8,
) -> Option<[f32; 2]> {
    let connector = meta
        .connectors
        .iter()
        .find(|connector| connector.index == index)?;
    let (min, size) = glyph_rect(center, half_size, meta.view_box);
    Some([
        min[0] + connector.pos[0] / meta.view_box[0] * size[0],
        min[1] + connector.pos[1] / meta.view_box[1] * size[1],
    ])
}

/// Half-extents matching the symbol's native aspect ratio for a full width.
pub fn half_size_for_width(view_box: [f32; 2], width: f32) -> [f32; 2] {
    [width / 2.0, width * view_box[1] / view_box[0] / 2.0]
}

/// Half-extents matching the symbol's native aspect ratio for a full height.
pub fn half_size_for_height(view_box: [f32; 2], height: f32) -> [f32; 2] {
    [height * view_box[0] / view_box[1] / 2.0, height / 2.0]
}

/// Parses the Equinor SVG metadata with a tiny scanner — the vendored format
/// is stable (`viewBox="0 0 W H"` plus annotation-connector circles), and the
/// tests pin every asset to it, so a real XML parser would be dead weight.
fn parse_meta(svg: &[u8]) -> Option<SymbolMeta> {
    let svg = std::str::from_utf8(svg).ok()?;
    Some(SymbolMeta {
        view_box: parse_view_box(svg)?,
        connectors: parse_connectors(svg),
    })
}

fn parse_view_box(svg: &str) -> Option<[f32; 2]> {
    let start = svg.find("viewBox=\"")? + "viewBox=\"".len();
    let value = &svg[start..start + svg[start..].find('"')?];
    let mut numbers = value.split_ascii_whitespace().map(str::parse::<f32>);
    let min_x = numbers.next()?.ok()?;
    let min_y = numbers.next()?.ok()?;
    let width = numbers.next()?.ok()?;
    let height = numbers.next()?.ok()?;
    // The connector math assumes the vendored `0 0` origin.
    (min_x == 0.0 && min_y == 0.0 && width > 0.0 && height > 0.0).then_some([width, height])
}

fn parse_connectors(svg: &str) -> Vec<Connector> {
    const MARKER: &str = "id=\"annotation-connector-";
    let mut connectors = Vec::new();
    let mut cursor = 0;
    while let Some(offset) = svg[cursor..].find(MARKER) {
        let id_start = cursor + offset + MARKER.len();
        cursor = id_start;
        connectors.extend(parse_connector_at(svg, id_start));
    }
    connectors
}

/// Parses one connector whose id value starts at `id_start`; `None` drops
/// malformed entries instead of failing the whole symbol.
fn parse_connector_at(svg: &str, id_start: usize) -> Option<Connector> {
    let id_end = id_start + svg[id_start..].find('"')?;
    let (index, degrees) = svg[id_start..id_end].split_once('-')?;
    // Attribute order inside the element is not guaranteed; scan the whole
    // `<circle …>` tag for cx/cy.
    let tag_start = svg[..id_start].rfind('<')?;
    let tag_end = id_end + svg[id_end..].find('>')?;
    let tag = &svg[tag_start..tag_end];
    Some(Connector {
        index: index.parse().ok()?,
        direction_deg: degrees.parse().ok()?,
        pos: [attr_f32(tag, "cx=\"")?, attr_f32(tag, "cy=\"")?],
    })
}

fn attr_f32(tag: &str, marker: &str) -> Option<f32> {
    let start = tag.find(marker)? + marker.len();
    let value = &tag[start..start + tag[start..].find('"')?];
    value.trim().parse().ok()
}

macro_rules! symbol {
    ($id:literal) => {
        symbol!($id, Equipment)
    };
    ($id:literal, $kind:ident) => {
        Symbol {
            id: $id,
            kind: SymbolKind::$kind,
            svg: include_bytes!(concat!("../assets/symbols/", $id, ".svg")),
        }
    };
}

/// The full embedded catalogue, sorted by id (required by [`find`]).
pub const SYMBOLS: &[Symbol] = &[
    symbol!("IM005A", Instrument),
    symbol!("IM017A", Instrument),
    symbol!("IM017B", Instrument),
    symbol!("IM017C", Instrument),
    symbol!("LZ009A", Instrument),
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

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

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

    #[test]
    fn every_symbol_parses_metadata() {
        for symbol in SYMBOLS {
            let meta = symbol
                .meta()
                .unwrap_or_else(|| panic!("{}: metadata does not parse", symbol.id));
            assert!(
                meta.view_box[0] > 0.0 && meta.view_box[1] > 0.0,
                "{}: degenerate viewBox",
                symbol.id
            );
            // `connector_point` looks connectors up by index, so indices
            // must be unique within a symbol and sit inside the viewBox.
            let mut indices: Vec<u8> = meta.connectors.iter().map(|c| c.index).collect();
            indices.sort_unstable();
            indices.dedup();
            assert_eq!(
                indices.len(),
                meta.connectors.len(),
                "{}: duplicate connector index",
                symbol.id
            );
            for connector in &meta.connectors {
                assert!(
                    (0.0..=meta.view_box[0]).contains(&connector.pos[0])
                        && (0.0..=meta.view_box[1]).contains(&connector.pos[1]),
                    "{}: connector {} outside the viewBox",
                    symbol.id,
                    connector.index
                );
            }
        }
    }

    #[test]
    fn demo_symbols_expose_the_expected_connectors() {
        let tank = find("PT002A").expect("vendored").meta().expect("parses");
        assert_eq!(tank.view_box, [96.0, 216.0]);
        assert_eq!(tank.connectors.len(), 4);

        // Pump suction is the impeller eye at the center; discharge is the
        // top-right stub. Anchoring pipes there is the whole point.
        let pump = find("PP007A").expect("vendored").meta().expect("parses");
        let suction = pump.connectors.iter().find(|c| c.index == 2).expect("2");
        let discharge = pump.connectors.iter().find(|c| c.index == 1).expect("1");
        assert_eq!(suction.pos, [48.0, 40.5]);
        assert_eq!(suction.direction_deg, 270);
        assert_eq!(discharge.pos, [87.5, 13.0]);
        assert_eq!(discharge.direction_deg, 90);

        let bubble = find("IM005A").expect("vendored").meta().expect("parses");
        assert_eq!(bubble.view_box, [48.0, 48.0]);
        assert_eq!(bubble.connectors.len(), 4);
    }

    #[test]
    fn glyph_rect_letterboxes_non_native_aspect() {
        // ND0001 (72×24) in a square box: full width, centered height.
        let (min, size) = glyph_rect([430.0, 125.0], [36.0, 36.0], [72.0, 24.0]);
        assert!(close(size[0], 72.0) && close(size[1], 24.0));
        assert!(close(min[0], 394.0) && close(min[1], 113.0));

        // Native aspect: the glyph fills the box exactly.
        let (min, size) = glyph_rect([0.0, 0.0], [40.0, 90.0], [96.0, 216.0]);
        assert!(close(min[0], -40.0) && close(min[1], -90.0));
        assert!(close(size[0], 80.0) && close(size[1], 180.0));
    }

    #[test]
    fn connector_point_maps_into_diagram_coordinates() {
        let tank = find("PT002A").expect("vendored");
        // Right-side nozzle (87.5, 108) of the 96×216 viewBox on an 80×180
        // glyph centered at the origin: x scales, y lands dead center.
        let point = tank
            .connector_point([0.0, 0.0], [40.0, 90.0], 1)
            .expect("connector 1");
        assert!(close(point[0], 87.5 / 96.0 * 80.0 - 40.0));
        assert!(close(point[1], 0.0));

        assert_eq!(tank.connector_point([0.0, 0.0], [40.0, 90.0], 9), None);
    }

    #[test]
    fn half_size_helpers_preserve_native_aspect() {
        let tank_box = [96.0, 216.0];
        let by_width = half_size_for_width(tank_box, 80.0);
        let by_height = half_size_for_height(tank_box, 180.0);
        assert!(close(by_width[0], 40.0) && close(by_width[1], 90.0));
        assert!(close(by_height[0], 40.0) && close(by_height[1], 90.0));

        let valve = half_size_for_width([72.0, 24.0], 72.0);
        assert!(close(valve[0], 36.0) && close(valve[1], 12.0));
    }

    #[test]
    fn instrument_symbols_are_flagged() {
        assert_eq!(
            find("IM005A").expect("vendored").kind,
            SymbolKind::Instrument
        );
        assert_eq!(
            find("LZ009A").expect("vendored").kind,
            SymbolKind::Instrument
        );
        assert_eq!(
            find("PP007A").expect("vendored").kind,
            SymbolKind::Equipment
        );
        assert_eq!(
            find("ND0001").expect("vendored").kind,
            SymbolKind::Equipment
        );
    }
}
