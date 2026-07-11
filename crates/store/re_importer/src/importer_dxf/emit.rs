//! Emit a neutral [`Drawing`] as Rerun [`Chunk`]s.

use std::collections::BTreeMap;

use anyhow::Result;
use re_chunk::{Chunk, ChunkBuilder, ChunkId, EntityPath, RowId, TimePoint};
use re_log_types::EntityPathPart;
use re_sdk_types::AsComponents;
use re_sdk_types::archetypes::{LineStrips3D, Points3D};
use re_sdk_types::components::Color as RrColor;

use super::domain::{Color, Drawing, Entity, EntityKind, Point3};

/// Number of straight segments used to discretise a full circle (360°).
const CIRCLE_SEGMENTS: usize = 64;

pub fn emit_drawing(
    emit: &mut dyn FnMut(Chunk),
    drawing: &Drawing,
    root_path: EntityPath,
    timepoint: &TimePoint,
) -> Result<()> {
    let buckets = bucket_by_layer(&drawing.entities);

    for (layer, ents) in &buckets {
        let safe_layer = sanitize_path_part(layer);
        let layer_root =
            root_path.clone() / EntityPathPart::new("layers") / EntityPathPart::new(safe_layer);

        emit_line_like(emit, &layer_root, ents, timepoint)?;
        emit_points(emit, &layer_root, ents, timepoint)?;
        emit_text(emit, &layer_root, ents, timepoint)?;
    }

    Ok(())
}

fn bucket_by_layer(entities: &[Entity]) -> BTreeMap<&str, Vec<&Entity>> {
    let mut by_layer: BTreeMap<&str, Vec<&Entity>> = BTreeMap::new();
    for e in entities {
        by_layer.entry(e.layer.as_str()).or_default().push(e);
    }
    by_layer
}

/// Sanitize a path segment for Rerun entity paths.
pub(crate) fn sanitize_path_part(name: &str) -> String {
    let safe: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if safe.is_empty() {
        "default".to_owned()
    } else {
        safe
    }
}

fn emit_archetype(
    emit: &mut dyn FnMut(Chunk),
    entity_path: EntityPath,
    timepoint: &TimePoint,
    archetype: &impl AsComponents,
) -> Result<()> {
    emit(
        ChunkBuilder::new(ChunkId::new(), entity_path)
            .with_archetype(RowId::new(), timepoint.clone(), archetype)
            .build()?,
    );
    Ok(())
}

fn emit_line_like(
    emit: &mut dyn FnMut(Chunk),
    layer_root: &EntityPath,
    ents: &[&Entity],
    timepoint: &TimePoint,
) -> Result<()> {
    let mut strips: Vec<Vec<[f32; 3]>> = Vec::new();
    let mut colors: Vec<RrColor> = Vec::new();
    let mut has_any_color = false;

    for e in ents {
        let strip: Option<Vec<[f32; 3]>> = match &e.kind {
            EntityKind::Line(l) => Some(vec![l.start.as_array_f32(), l.end.as_array_f32()]),
            EntityKind::Polyline(p) => {
                if p.vertices.len() < 2 {
                    None
                } else {
                    let mut v: Vec<[f32; 3]> =
                        p.vertices.iter().map(|pt| pt.as_array_f32()).collect();
                    if p.closed
                        && let Some(first) = v.first().copied()
                    {
                        v.push(first);
                    }
                    Some(v)
                }
            }
            EntityKind::Circle(c) => Some(discretize_arc(
                c.center,
                c.radius,
                0.0,
                360.0,
                CIRCLE_SEGMENTS,
            )),
            EntityKind::Arc(a) => {
                let sweep = normalize_sweep(a.start_angle_deg, a.end_angle_deg);
                let segs = ((sweep / 360.0) * CIRCLE_SEGMENTS as f64).ceil().max(2.0) as usize;
                Some(discretize_arc(
                    a.center,
                    a.radius,
                    a.start_angle_deg,
                    a.end_angle_deg,
                    segs,
                ))
            }
            _ => None,
        };

        if let Some(s) = strip {
            strips.push(s);
            match e.color {
                Some(c) => {
                    colors.push(rr_color(c));
                    has_any_color = true;
                }
                None => colors.push(rr_color(Color::WHITE)),
            }
        }
    }

    if strips.is_empty() {
        return Ok(());
    }

    let mut arch = LineStrips3D::new(strips);
    if has_any_color {
        arch = arch.with_colors(colors);
    }
    emit_archetype(
        emit,
        layer_root.clone() / EntityPathPart::new("lines"),
        timepoint,
        &arch,
    )
}

fn emit_points(
    emit: &mut dyn FnMut(Chunk),
    layer_root: &EntityPath,
    ents: &[&Entity],
    timepoint: &TimePoint,
) -> Result<()> {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<RrColor> = Vec::new();
    let mut has_any_color = false;

    for e in ents {
        if let EntityKind::Point(p) = &e.kind {
            positions.push(p.as_array_f32());
            match e.color {
                Some(c) => {
                    colors.push(rr_color(c));
                    has_any_color = true;
                }
                None => colors.push(rr_color(Color::WHITE)),
            }
        }
    }

    if positions.is_empty() {
        return Ok(());
    }

    let n = positions.len();
    let mut arch = Points3D::new(positions).with_radii(std::iter::repeat_n(0.05, n));
    if has_any_color {
        arch = arch.with_colors(colors);
    }
    emit_archetype(
        emit,
        layer_root.clone() / EntityPathPart::new("points"),
        timepoint,
        &arch,
    )
}

fn emit_text(
    emit: &mut dyn FnMut(Chunk),
    layer_root: &EntityPath,
    ents: &[&Entity],
    timepoint: &TimePoint,
) -> Result<()> {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut labels: Vec<String> = Vec::new();
    let mut colors: Vec<RrColor> = Vec::new();
    let mut has_any_color = false;

    for e in ents {
        if let EntityKind::Text(t) = &e.kind {
            positions.push(t.position.as_array_f32());
            labels.push(t.text.clone());
            match e.color {
                Some(c) => {
                    colors.push(rr_color(c));
                    has_any_color = true;
                }
                None => colors.push(rr_color(Color::WHITE)),
            }
        }
    }

    if positions.is_empty() {
        return Ok(());
    }

    let n = positions.len();
    let mut arch = Points3D::new(positions)
        .with_labels(labels)
        .with_radii(std::iter::repeat_n(0.02, n));
    if has_any_color {
        arch = arch.with_colors(colors);
    }
    emit_archetype(
        emit,
        layer_root.clone() / EntityPathPart::new("text"),
        timepoint,
        &arch,
    )
}

fn rr_color(c: Color) -> RrColor {
    RrColor::from_rgb(c.r, c.g, c.b)
}

fn discretize_arc(
    center: Point3,
    radius: f64,
    start_deg: f64,
    end_deg: f64,
    segments: usize,
) -> Vec<[f32; 3]> {
    let sweep = normalize_sweep(start_deg, end_deg);
    let start = start_deg.to_radians();
    let sweep_rad = sweep.to_radians();
    let n = segments.max(2);

    (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let a = start + sweep_rad * t;
            let x = center.x + radius * a.cos();
            let y = center.y + radius * a.sin();
            let z = center.z;
            [x as f32, y as f32, z as f32]
        })
        .collect()
}

fn normalize_sweep(start_deg: f64, end_deg: f64) -> f64 {
    let mut s = end_deg - start_deg;
    while s <= 0.0 {
        s += 360.0;
    }
    while s > 360.0 {
        s -= 360.0;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_sweep_wraps() {
        assert!((normalize_sweep(350.0, 10.0) - 20.0).abs() < 1e-9);
        assert!((normalize_sweep(0.0, 90.0) - 90.0).abs() < 1e-9);
    }

    #[test]
    fn discretize_full_circle_first_equals_last() {
        let pts = discretize_arc(Point3::new(0.0, 0.0, 0.0), 1.0, 0.0, 360.0, 16);
        let first = pts.first().unwrap();
        let last = pts.last().unwrap();
        let d = ((first[0] - last[0]).powi(2) + (first[1] - last[1]).powi(2)).sqrt();
        assert!(d < 1e-5, "circle endpoints should meet, got {d}");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn emit_sample_dxf_produces_chunks() {
        use std::path::Path;

        use crate::importer_dxf::parse::load_dxf;

        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/dxf/sample.dxf");
        let drawing = load_dxf(&path).expect("sample.dxf should parse");

        let root = EntityPath::from_single_string("sample");
        let mut chunks = Vec::new();
        emit_drawing(
            &mut |chunk| chunks.push(chunk),
            &drawing,
            root,
            &TimePoint::STATIC,
        )
        .expect("emit should succeed");

        assert!(!chunks.is_empty(), "should emit at least one chunk");

        let paths: Vec<String> = chunks.iter().map(|c| c.entity_path().to_string()).collect();
        assert!(
            paths.iter().any(|p| p.contains("layers/walls/lines")),
            "expected walls layer lines, got {paths:?}"
        );
        assert!(
            paths.iter().any(|p| p.contains("layers/annotations/text")),
            "expected annotations text, got {paths:?}"
        );
    }
}
