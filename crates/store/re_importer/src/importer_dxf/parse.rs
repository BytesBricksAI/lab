//! DXF parser: reads a DXF file and produces a neutral [`Drawing`].

use std::io::Cursor;
use std::path::Path;

use anyhow::{Context as _, Result};
use dxf::Drawing as DxfDrawing;
use dxf::entities::EntityType;

use super::domain::{
    Arc, Circle, Color, Drawing, Entity, EntityKind, Line, Point3, Polyline, TextLabel,
};

/// Maximum recursion depth when expanding `INSERT` references.
pub(crate) const MAX_INSERT_DEPTH: usize = 8;

pub fn load_dxf(path: impl AsRef<Path>) -> Result<Drawing> {
    let path = path.as_ref();
    let dxf = DxfDrawing::load_file(path)
        .with_context(|| format!("failed to parse DXF file: {}", path.display()))?;
    parse_dxf_drawing(&dxf)
}

pub fn load_dxf_bytes(contents: &[u8]) -> Result<Drawing> {
    let mut reader = Cursor::new(contents);
    let dxf = DxfDrawing::load(&mut reader).context("failed to parse DXF bytes")?;
    parse_dxf_drawing(&dxf)
}

fn parse_dxf_drawing(dxf: &DxfDrawing) -> Result<Drawing> {
    let mut out = Drawing::new();
    let identity = Transform::identity();
    for ent in dxf.entities() {
        convert_entity(ent, dxf, &identity, 0, &mut out);
    }
    Ok(out)
}

/// Affine 2.5D transform applied when expanding an `INSERT`.
#[derive(Debug, Clone, Copy)]
struct Transform {
    tx: f64,
    ty: f64,
    tz: f64,
    sx: f64,
    sy: f64,
    sz: f64,
    cos_r: f64,
    sin_r: f64,
}

impl Transform {
    const fn identity() -> Self {
        Self {
            tx: 0.0,
            ty: 0.0,
            tz: 0.0,
            sx: 1.0,
            sy: 1.0,
            sz: 1.0,
            cos_r: 1.0,
            sin_r: 0.0,
        }
    }

    fn from_insert(ins: &dxf::entities::Insert) -> Self {
        let r = ins.rotation.to_radians();
        Self {
            tx: ins.location.x,
            ty: ins.location.y,
            tz: ins.location.z,
            sx: ins.x_scale_factor,
            sy: ins.y_scale_factor,
            sz: ins.z_scale_factor,
            cos_r: r.cos(),
            sin_r: r.sin(),
        }
    }

    fn apply(&self, p: Point3) -> Point3 {
        let sx = p.x * self.sx;
        let sy = p.y * self.sy;
        let sz = p.z * self.sz;
        Point3::new(
            self.cos_r * sx - self.sin_r * sy + self.tx,
            self.sin_r * sx + self.cos_r * sy + self.ty,
            sz + self.tz,
        )
    }

    fn compose(&self, other: &Transform) -> Transform {
        let origin = self.apply(Point3::new(other.tx, other.ty, other.tz));
        let cos_r = self.cos_r * other.cos_r - self.sin_r * other.sin_r;
        let sin_r = self.sin_r * other.cos_r + self.cos_r * other.sin_r;
        Transform {
            tx: origin.x,
            ty: origin.y,
            tz: origin.z,
            sx: self.sx * other.sx,
            sy: self.sy * other.sy,
            sz: self.sz * other.sz,
            cos_r,
            sin_r,
        }
    }
}

fn convert_entity(
    ent: &dxf::entities::Entity,
    dxf: &DxfDrawing,
    xform: &Transform,
    depth: usize,
    out: &mut Drawing,
) {
    let layer = ent.common.layer.clone();
    let color = map_color(&ent.common.color);

    match &ent.specific {
        EntityType::Line(l) => {
            out.push(Entity {
                layer,
                color,
                kind: EntityKind::Line(Line {
                    start: xform.apply(point(&l.p1)),
                    end: xform.apply(point(&l.p2)),
                }),
            });
        }
        EntityType::Circle(c) => {
            let avg_scale = (xform.sx.abs() + xform.sy.abs()) * 0.5;
            out.push(Entity {
                layer,
                color,
                kind: EntityKind::Circle(Circle {
                    center: xform.apply(point(&c.center)),
                    radius: c.radius * avg_scale,
                }),
            });
        }
        EntityType::Arc(a) => {
            let avg_scale = (xform.sx.abs() + xform.sy.abs()) * 0.5;
            let rot_deg = xform.sin_r.atan2(xform.cos_r).to_degrees();
            out.push(Entity {
                layer,
                color,
                kind: EntityKind::Arc(Arc {
                    center: xform.apply(point(&a.center)),
                    radius: a.radius * avg_scale,
                    start_angle_deg: a.start_angle + rot_deg,
                    end_angle_deg: a.end_angle + rot_deg,
                }),
            });
        }
        EntityType::LwPolyline(p) => {
            let vertices = p
                .vertices
                .iter()
                .map(|v| xform.apply(Point3::new(v.x, v.y, 0.0)))
                .collect();
            out.push(Entity {
                layer,
                color,
                kind: EntityKind::Polyline(Polyline {
                    vertices,
                    closed: p.is_closed(),
                }),
            });
        }
        EntityType::Polyline(p) => {
            let vertices = p
                .vertices()
                .map(|v| xform.apply(point(&v.location)))
                .collect();
            out.push(Entity {
                layer,
                color,
                kind: EntityKind::Polyline(Polyline {
                    vertices,
                    closed: p.is_closed(),
                }),
            });
        }
        EntityType::ModelPoint(p) => {
            out.push(Entity {
                layer,
                color,
                kind: EntityKind::Point(xform.apply(point(&p.location))),
            });
        }
        EntityType::Text(t) => {
            out.push(Entity {
                layer,
                color,
                kind: EntityKind::Text(TextLabel {
                    position: xform.apply(point(&t.location)),
                    text: t.value.clone(),
                    height: t.text_height * xform.sy.abs(),
                }),
            });
        }
        EntityType::MText(m) => {
            let cleaned = strip_mtext_codes(&m.text);
            out.push(Entity {
                layer,
                color,
                kind: EntityKind::Text(TextLabel {
                    position: xform.apply(point(&m.insertion_point)),
                    text: cleaned,
                    height: m.initial_text_height * xform.sy.abs(),
                }),
            });
        }
        EntityType::Spline(s) => {
            let source = if !s.fit_points.is_empty() {
                &s.fit_points
            } else {
                &s.control_points
            };
            if source.len() >= 2 {
                let vertices = source.iter().map(|p| xform.apply(point(p))).collect();
                out.push(Entity {
                    layer,
                    color,
                    kind: EntityKind::Polyline(Polyline {
                        vertices,
                        closed: s.is_closed(),
                    }),
                });
            }
        }
        EntityType::Insert(ins) => {
            if depth >= MAX_INSERT_DEPTH {
                return;
            }
            let inner_xform = xform.compose(&Transform::from_insert(ins));
            let cols = ins.column_count.max(1) as i32;
            let rows = ins.row_count.max(1) as i32;
            for col in 0..cols {
                for row in 0..rows {
                    let offset = inner_xform.apply(Point3::new(
                        col as f64 * ins.column_spacing,
                        row as f64 * ins.row_spacing,
                        0.0,
                    ));
                    let placed = Transform {
                        tx: offset.x,
                        ty: offset.y,
                        tz: offset.z,
                        ..inner_xform
                    };
                    if let Some(block) = find_block(dxf, &ins.name) {
                        for inner_ent in &block.entities {
                            convert_entity(inner_ent, dxf, &placed, depth + 1, out);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn find_block<'a>(dxf: &'a DxfDrawing, name: &str) -> Option<&'a dxf::Block> {
    dxf.blocks().find(|b| b.name == name)
}

fn point(p: &dxf::Point) -> Point3 {
    Point3::new(p.x, p.y, p.z)
}

fn strip_mtext_codes(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    let mut brace_depth: i32 = 0;
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('P') => out.push('\n'),
                Some('~') => out.push(' '),
                Some('\\') => out.push('\\'),
                Some(_) => {
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next == ';' || next.is_whitespace() {
                            break;
                        }
                    }
                }
                None => break,
            },
            '{' => brace_depth += 1,
            '}' => brace_depth = (brace_depth - 1).max(0),
            _ => out.push(c),
        }
    }
    out
}

fn map_color(c: &dxf::Color) -> Option<Color> {
    let idx = c.index()?;
    match idx {
        1 => Some(Color::new(0xE6, 0x33, 0x33)),
        2 => Some(Color::new(0xE6, 0xC9, 0x33)),
        3 => Some(Color::new(0x33, 0xC0, 0x4D)),
        4 => Some(Color::new(0x33, 0xC9, 0xE6)),
        5 => Some(Color::new(0x4D, 0x66, 0xE6)),
        6 => Some(Color::new(0xE6, 0x33, 0xC9)),
        7 => Some(Color::WHITE),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample_dxf() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/dxf/sample.dxf");
        let drawing = load_dxf(&path).expect("sample.dxf should parse");
        assert!(
            !drawing.entities.is_empty(),
            "sample.dxf should contain entities"
        );

        let layers: std::collections::BTreeSet<_> =
            drawing.entities.iter().map(|e| e.layer.as_str()).collect();
        assert!(layers.contains("walls"));
        assert!(layers.contains("doors"));
        assert!(layers.contains("furniture"));
        assert!(layers.contains("annotations"));
    }
}
