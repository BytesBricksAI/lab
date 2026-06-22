//! Neutral CAD domain model for DXF import.
//!
//! These types describe CAD geometry without knowing anything about the
//! source format (DXF) or the target viewer (Rerun).

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn as_array_f32(self) -> [f32; 3] {
        [self.x as f32, self.y as f32, self.z as f32]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const WHITE: Self = Self::new(0xE0, 0xE0, 0xE0);
}

#[derive(Debug, Clone)]
pub struct Line {
    pub start: Point3,
    pub end: Point3,
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub center: Point3,
    pub radius: f64,
}

/// Counter-clockwise arc in its local plane (z = center.z).
/// Angles in degrees following DXF convention.
#[derive(Debug, Clone)]
pub struct Arc {
    pub center: Point3,
    pub radius: f64,
    pub start_angle_deg: f64,
    pub end_angle_deg: f64,
}

#[derive(Debug, Clone)]
pub struct Polyline {
    pub vertices: Vec<Point3>,
    pub closed: bool,
}

#[derive(Debug, Clone)]
pub struct TextLabel {
    pub position: Point3,
    pub text: String,
    #[allow(dead_code)]
    pub height: f64,
}

/// Geometric entity tagged with the layer it belongs to and an optional color.
#[derive(Debug, Clone)]
pub struct Entity {
    pub layer: String,
    pub color: Option<Color>,
    pub kind: EntityKind,
}

#[derive(Debug, Clone)]
pub enum EntityKind {
    Line(Line),
    Circle(Circle),
    Arc(Arc),
    Polyline(Polyline),
    Point(Point3),
    Text(TextLabel),
}

/// A drawing is just a flat collection of entities.
///
/// Block references (DXF `INSERT`) are expanded by the loader at parse time.
#[derive(Debug, Clone, Default)]
pub struct Drawing {
    pub entities: Vec<Entity>,
}

impl Drawing {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, entity: Entity) {
        self.entities.push(entity);
    }
}
