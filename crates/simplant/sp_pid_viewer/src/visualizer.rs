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

/// Line style of a placed pipe (ISA-5.1): process lines are solid,
/// instrument signal lines are dashed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PipeKind {
    /// Solid process line.
    #[default]
    Process,

    /// Dashed instrument signal line.
    Signal,
}

/// A pipe polyline placed on the diagram, in diagram coordinates (y down).
#[derive(Debug, Clone)]
pub struct PlacedPipe {
    /// Polyline vertices in diagram coordinates.
    pub points: Vec<egui::Pos2>,

    /// Line style (process = solid, signal = dashed).
    pub kind: PipeKind,
}

impl PlacedPipe {
    /// A process-line polyline through the given diagram points.
    pub fn new(points: Vec<egui::Pos2>) -> Self {
        Self {
            points,
            kind: PipeKind::Process,
        }
    }

    /// Sets the line style.
    #[inline]
    pub fn with_kind(mut self, kind: PipeKind) -> Self {
        self.kind = kind;
        self
    }
}

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

    /// Per-symbol responses, same order as the input slice (for hosts that
    /// wire selection/hover into their own state, e.g. the viewer).
    pub symbol_responses: Vec<egui::Response>,

    /// The region of diagram coordinates visible after pan/zoom; together
    /// with [`Self::response`].rect it maps diagram → screen coordinates.
    pub scene_rect: egui::Rect,
}

impl PidCanvasResponse {
    /// Maps a position in diagram coordinates to screen coordinates.
    pub fn screen_from_diagram(&self, pos: egui::Pos2) -> egui::Pos2 {
        let screen = self.response.rect;
        let scene = self.scene_rect;
        egui::pos2(
            screen.min.x + (pos.x - scene.min.x) * screen.width() / scene.width().max(f32::EPSILON),
            screen.min.y
                + (pos.y - scene.min.y) * screen.height() / scene.height().max(f32::EPSILON),
        )
    }
}

/// P&ID canvas widget with pan + zoom (drag to pan, scroll to zoom,
/// double-click the background to re-fit the diagram).
pub struct PidCanvas<'a> {
    placed: &'a [PlacedSymbol],
    pipes: &'a [PlacedPipe],
    id_salt: egui::Id,
}

impl<'a> PidCanvas<'a> {
    /// A canvas over the given placed symbols.
    pub fn new(placed: &'a [PlacedSymbol]) -> Self {
        Self {
            placed,
            pipes: &[],
            id_salt: egui::Id::new("sp_pid_canvas"),
        }
    }

    /// Adds process-line polylines drawn beneath the symbols.
    #[inline]
    pub fn with_pipes(mut self, pipes: &'a [PlacedPipe]) -> Self {
        self.pipes = pipes;
        self
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
        let fit_rect = content_bounds(self.placed, self.pipes)
            .unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1.0, 1.0)))
            .expand(24.0);
        let mut scene_rect: egui::Rect = ui
            .ctx()
            .data(|data| data.get_temp(scene_id))
            .unwrap_or(fit_rect);

        let symbol_color = ui.visuals().text_color();
        let mut clicked = None;
        let mut hovered = None;
        let mut symbol_responses = Vec::with_capacity(self.placed.len());

        let response = egui::Scene::new()
            .show(ui, &mut scene_rect, |ui| {
                for pipe in self.pipes {
                    if pipe.points.len() < 2 {
                        continue;
                    }
                    match pipe.kind {
                        PipeKind::Process => {
                            ui.painter().add(egui::Shape::line(
                                pipe.points.clone(),
                                egui::Stroke::new(1.5, symbol_color),
                            ));
                        }
                        PipeKind::Signal => {
                            ui.painter().extend(egui::Shape::dashed_line(
                                &pipe.points,
                                egui::Stroke::new(1.0, symbol_color),
                                5.0,
                                4.0,
                            ));
                        }
                    }
                }
                for (index, placed) in self.placed.iter().enumerate() {
                    let interaction = draw_symbol(ui, placed, symbol_color);
                    if interaction.hovered() {
                        hovered = Some(index);
                    }
                    if interaction.clicked() {
                        clicked = Some(index);
                    }
                    symbol_responses.push(interaction);
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
            symbol_responses,
            scene_rect,
        }
    }
}

/// Draws one symbol (or its placeholder) and returns its interaction response.
fn draw_symbol(
    ui: &mut egui::Ui,
    placed: &PlacedSymbol,
    symbol_color: egui::Color32,
) -> egui::Response {
    let bounds = egui::Rect::from_center_size(placed.center, placed.size);
    let response = ui.allocate_rect(bounds, egui::Sense::click());

    let symbol = symbols::find(&placed.symbol_id);

    // The glyph is painted in the aspect-preserving rect that
    // `symbols::connector_point` maps into, so pipes anchored to connectors
    // meet the artwork exactly (and non-native boxes no longer distort it).
    let glyph = symbol
        .and_then(|symbol| symbol.meta())
        .map_or(bounds, |meta| {
            let (min, size) = symbols::glyph_rect(
                [placed.center.x, placed.center.y],
                [placed.size.x / 2.0, placed.size.y / 2.0],
                meta.view_box,
            );
            egui::Rect::from_min_size(egui::pos2(min[0], min[1]), egui::vec2(size[0], size[1]))
        });

    if let Some(symbol) = symbol {
        let uri = symbol_uri(symbol.id, symbol_color);
        ensure_svg_registered(ui.ctx(), &uri, symbol.svg, symbol_color);
        egui::Image::from_uri(uri).paint_at(ui, glyph);
    } else {
        // Unknown or unmapped symbol: honest placeholder instead of a
        // wrong icon on a P&ID.
        ui.painter().rect_stroke(
            bounds,
            egui::CornerRadius::same(2),
            egui::Stroke::new(1.0, symbol_color),
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            bounds.center(),
            egui::Align2::CENTER_CENTER,
            "?",
            egui::FontId::proportional(bounds.height() * 0.4),
            symbol_color,
        );
    }

    if symbol.is_some_and(|symbol| symbol.kind == symbols::SymbolKind::Instrument) {
        draw_instrument_tag(ui, placed, glyph);
    } else {
        draw_equipment_tag(ui, placed, glyph);
    }

    let tooltip = match &placed.live_value {
        Some(value) => format!("{}\n{}", placed.label, value),
        None => placed.label.clone(),
    };
    response.on_hover_text(tooltip)
}

/// Equipment tag (and live value) under the glyph.
fn draw_equipment_tag(ui: &egui::Ui, placed: &PlacedSymbol, glyph: egui::Rect) {
    let font = egui::FontId::proportional((glyph.height() * 0.16).clamp(6.0, 14.0));
    let label_pos = glyph.center_bottom() + egui::vec2(0.0, 2.0);
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
}

/// ISA-5.1 instrument tag: identification letters and loop number go
/// *inside* the bubble (split at the first `-`). The live value sits beside
/// the bubble — leader lines usually arrive from below, so text under the
/// bubble would collide with them.
fn draw_instrument_tag(ui: &egui::Ui, placed: &PlacedSymbol, glyph: egui::Rect) {
    let font = egui::FontId::proportional((glyph.height() * 0.26).clamp(5.0, 14.0));
    let color = ui.visuals().text_color();
    match placed.label.split_once('-') {
        Some((letters, number)) => {
            let offset = egui::vec2(0.0, font.size * 0.55);
            ui.painter().text(
                glyph.center() - offset,
                egui::Align2::CENTER_CENTER,
                letters,
                font.clone(),
                color,
            );
            ui.painter().text(
                glyph.center() + offset,
                egui::Align2::CENTER_CENTER,
                number,
                font.clone(),
                color,
            );
        }
        None => {
            ui.painter().text(
                glyph.center(),
                egui::Align2::CENTER_CENTER,
                &placed.label,
                font.clone(),
                color,
            );
        }
    }
    if let Some(value) = &placed.live_value {
        ui.painter().text(
            glyph.right_center() + egui::vec2(4.0, 0.0),
            egui::Align2::LEFT_CENTER,
            value,
            font,
            ui.visuals().strong_text_color(),
        );
    }
}

/// Registers the themed SVG bytes under `uri` once per egui context.
fn ensure_svg_registered(ctx: &egui::Context, uri: &str, svg: &'static [u8], color: egui::Color32) {
    let flag = egui::Id::new(uri);
    let already = ctx
        .data(|data| data.get_temp::<bool>(flag))
        .unwrap_or(false);
    if !already {
        ctx.include_bytes(uri.to_owned(), themed_svg(svg, color));
        ctx.data_mut(|data| data.insert_temp(flag, true));
    }
}

/// URI under which a symbol's SVG is registered; one per (symbol, color) so
/// theme changes load a freshly tinted copy.
fn symbol_uri(id: &str, color: egui::Color32) -> String {
    format!("bytes://sp_pid_viewer/{}/{id}.svg", color_hex(color))
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

/// Bounding box of all placed symbols and pipe points, in diagram coordinates.
fn content_bounds(placed: &[PlacedSymbol], pipes: &[PlacedPipe]) -> Option<egui::Rect> {
    let symbol_bounds = placed
        .iter()
        .map(|symbol| egui::Rect::from_center_size(symbol.center, symbol.size));
    let pipe_bounds = pipes.iter().filter_map(|pipe| {
        if pipe.points.is_empty() {
            None
        } else {
            let mut iter = pipe.points.iter();
            let first = *iter.next()?;
            let mut rect = egui::Rect::from_min_max(first, first);
            for point in iter {
                rect.extend_with(*point);
            }
            Some(rect)
        }
    });
    symbol_bounds
        .chain(pipe_bounds)
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
            PlacedSymbol::new(
                "PP007A",
                "P-101",
                egui::pos2(0.0, 0.0),
                egui::vec2(10.0, 10.0),
            ),
            PlacedSymbol::new(
                "PT002A",
                "T-201",
                egui::pos2(100.0, 50.0),
                egui::vec2(20.0, 40.0),
            ),
        ];
        let bounds = content_bounds(&placed, &[]).expect("non-empty input");
        assert_eq!(bounds.min, egui::pos2(-5.0, -5.0));
        assert_eq!(bounds.max, egui::pos2(110.0, 70.0));
        assert!(content_bounds(&[], &[]).is_none());
    }

    #[test]
    fn content_bounds_covers_pipes_beyond_symbols() {
        let placed = [PlacedSymbol::new(
            "PP007A",
            "P-101",
            egui::pos2(0.0, 0.0),
            egui::vec2(10.0, 10.0),
        )];
        let pipes = [PlacedPipe::new(vec![
            egui::pos2(200.0, 0.0),
            egui::pos2(300.0, 50.0),
        ])];
        let bounds = content_bounds(&placed, &pipes).expect("non-empty input");
        assert_eq!(bounds.min, egui::pos2(-5.0, -5.0));
        assert_eq!(bounds.max, egui::pos2(300.0, 50.0));
    }
}
