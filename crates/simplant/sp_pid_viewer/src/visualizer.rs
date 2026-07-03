//! Interactive P&ID canvas: draws embedded symbols with pan + zoom and
//! reports hover/click so the host can wire selection and live data.
//!
//! This is the only module in the crate that touches `egui`. The host app is
//! responsible for having egui's image loaders installed (the SimPlant Lab
//! viewer already does this via `re_ui`).

use crate::symbols;

/// Fill color used by the Equinor SVG sources; replaced at load time so the
/// symbols stay visible in both light and dark themes.
const EQUINOR_FILL: &str = "#231f20";

/// A symbol instance placed on the diagram, in diagram coordinates (y down).
#[derive(Debug, Clone)]
pub struct PlacedSymbol {
    /// Equinor symbol id, e.g. `"PP007A"`. Unknown ids get a placeholder box.
    pub symbol_id: String,

    /// Equipment tag shown under the symbol, e.g. `"P-101"`.
    pub label: String,

    /// Center of the symbol in diagram coordinates.
    pub center: egui::Pos2,

    /// Size of the symbol in diagram units.
    pub size: egui::Vec2,

    /// Latest streamed value to display under the tag, e.g. `"12.3 bar"`.
    pub live_value: Option<String>,
}

impl PlacedSymbol {
    /// A symbol instance with no live value attached yet.
    pub fn new(
        symbol_id: impl Into<String>,
        label: impl Into<String>,
        center: egui::Pos2,
        size: egui::Vec2,
    ) -> Self {
        Self {
            symbol_id: symbol_id.into(),
            label: label.into(),
            center,
            size,
            live_value: None,
        }
    }

    /// Attaches the latest streamed value, shown under the equipment tag.
    #[inline]
    pub fn with_live_value(mut self, value: impl Into<String>) -> Self {
        self.live_value = Some(value.into());
        self
    }
}

/// What happened on the canvas this frame.
pub struct PidCanvasResponse {
    /// Index into the `symbols` slice of the symbol clicked this frame.
    pub clicked: Option<usize>,

    /// Index into the `symbols` slice of the symbol under the pointer.
    pub hovered: Option<usize>,

    /// The response covering the whole canvas area.
    pub response: egui::Response,
}

/// P&ID canvas widget with pan + zoom (drag to pan, scroll to zoom,
/// double-click the background to re-fit the diagram).
pub struct PidCanvas<'a> {
    placed: &'a [PlacedSymbol],
    id_salt: egui::Id,
}

impl<'a> PidCanvas<'a> {
    /// A canvas over the given placed symbols.
    pub fn new(placed: &'a [PlacedSymbol]) -> Self {
        Self {
            placed,
            id_salt: egui::Id::new("sp_pid_canvas"),
        }
    }

    /// Distinguishes multiple canvases living in the same `Ui`.
    #[inline]
    pub fn with_id_salt(mut self, salt: impl std::hash::Hash) -> Self {
        self.id_salt = egui::Id::new(salt);
        self
    }

    /// Shows the canvas and reports interactions.
    pub fn show(&self, ui: &mut egui::Ui) -> PidCanvasResponse {
        let scene_id = ui.make_persistent_id(self.id_salt);
        let fit_rect = content_bounds(self.placed)
            .unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1.0, 1.0)))
            .expand(24.0);
        let mut scene_rect: egui::Rect = ui
            .ctx()
            .data(|data| data.get_temp(scene_id))
            .unwrap_or(fit_rect);

        let symbol_color = ui.visuals().text_color();
        let mut clicked = None;
        let mut hovered = None;

        let response = egui::Scene::new()
            .show(ui, &mut scene_rect, |ui| {
                for (index, placed) in self.placed.iter().enumerate() {
                    let interaction = draw_symbol(ui, placed, symbol_color);
                    if interaction.hovered() {
                        hovered = Some(index);
                    }
                    if interaction.clicked() {
                        clicked = Some(index);
                    }
                }
            })
            .response;

        if response.double_clicked() {
            scene_rect = fit_rect;
        }
        ui.ctx()
            .data_mut(|data| data.insert_temp(scene_id, scene_rect));

        PidCanvasResponse {
            clicked,
            hovered,
            response,
        }
    }
}

/// Draws one symbol (or its placeholder) and returns its interaction response.
fn draw_symbol(
    ui: &mut egui::Ui,
    placed: &PlacedSymbol,
    symbol_color: egui::Color32,
) -> egui::Response {
    let rect = egui::Rect::from_center_size(placed.center, placed.size);
    let response = ui.allocate_rect(rect, egui::Sense::click());

    if let Some(symbol) = symbols::find(&placed.symbol_id) {
        let uri = symbol_uri(symbol.id, symbol_color);
        ensure_svg_registered(ui.ctx(), &uri, symbol.svg, symbol_color);
        egui::Image::from_uri(uri).paint_at(ui, rect);
    } else {
        // Unknown or unmapped symbol: honest placeholder instead of a
        // wrong icon on a P&ID.
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::same(2),
            egui::Stroke::new(1.0, symbol_color),
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "?",
            egui::FontId::proportional(rect.height() * 0.4),
            symbol_color,
        );
    }

    let font = egui::FontId::proportional((placed.size.y * 0.16).clamp(6.0, 14.0));
    let label_pos = rect.center_bottom() + egui::vec2(0.0, 2.0);
    ui.painter().text(
        label_pos,
        egui::Align2::CENTER_TOP,
        &placed.label,
        font.clone(),
        ui.visuals().text_color(),
    );
    if let Some(value) = &placed.live_value {
        ui.painter().text(
            label_pos + egui::vec2(0.0, font.size * 1.3),
            egui::Align2::CENTER_TOP,
            value,
            font,
            ui.visuals().strong_text_color(),
        );
    }

    let tooltip = match &placed.live_value {
        Some(value) => format!("{}\n{}", placed.label, value),
        None => placed.label.clone(),
    };
    response.on_hover_text(tooltip)
}

/// Registers the themed SVG bytes under `uri` once per egui context.
fn ensure_svg_registered(
    ctx: &egui::Context,
    uri: &str,
    svg: &'static [u8],
    color: egui::Color32,
) {
    let flag = egui::Id::new(uri);
    let already = ctx.data(|data| data.get_temp::<bool>(flag)).unwrap_or(false);
    if !already {
        ctx.include_bytes(uri.to_owned(), themed_svg(svg, color));
        ctx.data_mut(|data| data.insert_temp(flag, true));
    }
}

/// URI under which a symbol's SVG is registered; one per (symbol, color) so
/// theme changes load a freshly tinted copy.
fn symbol_uri(id: &str, color: egui::Color32) -> String {
    format!(
        "bytes://sp_pid_viewer/{}/{id}.svg",
        color_hex(color)
    )
}

/// Replaces the Equinor fill color with the theme's text color so dark themes
/// don't render near-black symbols on a near-black background.
fn themed_svg(svg: &'static [u8], color: egui::Color32) -> Vec<u8> {
    let hex = color_hex(color);
    String::from_utf8_lossy(svg)
        .replacen(EQUINOR_FILL, &format!("#{hex}"), 1)
        .into_bytes()
}

fn color_hex(color: egui::Color32) -> String {
    format!("{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

/// Bounding box of all placed symbols, in diagram coordinates.
fn content_bounds(placed: &[PlacedSymbol]) -> Option<egui::Rect> {
    placed
        .iter()
        .map(|symbol| egui::Rect::from_center_size(symbol.center, symbol.size))
        .reduce(|acc, rect| acc.union(rect))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn themed_svg_replaces_the_equinor_fill() {
        let symbol = symbols::find("PP007A").expect("PP007A is vendored");
        let themed = themed_svg(symbol.svg, egui::Color32::WHITE);
        let themed = String::from_utf8(themed).expect("svg stays utf-8");
        assert!(themed.contains("#ffffff"));
        assert!(!themed.contains(EQUINOR_FILL));
    }

    #[test]
    fn symbol_uris_differ_per_color() {
        let light = symbol_uri("PP007A", egui::Color32::BLACK);
        let dark = symbol_uri("PP007A", egui::Color32::WHITE);
        assert_ne!(light, dark);
    }

    #[test]
    fn content_bounds_covers_all_symbols() {
        let placed = [
            PlacedSymbol::new("PP007A", "P-101", egui::pos2(0.0, 0.0), egui::vec2(10.0, 10.0)),
            PlacedSymbol::new("PT002A", "T-201", egui::pos2(100.0, 50.0), egui::vec2(20.0, 40.0)),
        ];
        let bounds = content_bounds(&placed).expect("non-empty input");
        assert_eq!(bounds.min, egui::pos2(-5.0, -5.0));
        assert_eq!(bounds.max, egui::pos2(110.0, 70.0));
        assert!(content_bounds(&[]).is_none());
    }
}
