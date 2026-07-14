//! `PidPipe` archetype: a process line (pipe) on a P&ID diagram.
//!
//! Logged per pipe entity (e.g. `pid/pipes/TK-101-P-101`); the P&ID view
//! renders it as a polyline through the given diagram points.

use re_sdk_types::{
    Archetype, ArchetypeName, AsComponents, Component as _, ComponentDescriptor,
    SerializedComponentBatch, components, datatypes, try_serialize_field,
};

use crate::namespace::{ARCHETYPE_PID_PIPE, field};

/// A process line on the P&ID: a polyline through diagram points (y down).
#[derive(Default)]
pub struct PidPipe {
    /// Polyline vertices, in diagram coordinates (y down).
    pub points: Option<SerializedComponentBatch>,

    /// Line kind: `"process"` (solid, default) or `"signal"` (dashed,
    /// ISA-5.1 instrument signal).
    pub kind: Option<SerializedComponentBatch>,
}

impl Archetype for PidPipe {
    fn name() -> ArchetypeName {
        ARCHETYPE_PID_PIPE.into()
    }

    fn display_name() -> &'static str {
        "P&ID pipe"
    }

    fn required_components() -> std::borrow::Cow<'static, [ComponentDescriptor]> {
        vec![Self::descriptor_points()].into()
    }

    fn optional_components() -> std::borrow::Cow<'static, [ComponentDescriptor]> {
        vec![Self::descriptor_kind()].into()
    }
}

impl PidPipe {
    /// Descriptor for [`Self::points`].
    #[inline]
    pub fn descriptor_points() -> ComponentDescriptor {
        ComponentDescriptor {
            archetype: Some(ARCHETYPE_PID_PIPE.into()),
            component: field(ARCHETYPE_PID_PIPE, "points").into(),
            component_type: Some(components::LineStrip2D::name()),
        }
    }

    /// Descriptor for [`Self::kind`].
    #[inline]
    pub fn descriptor_kind() -> ComponentDescriptor {
        ComponentDescriptor {
            archetype: Some(ARCHETYPE_PID_PIPE.into()),
            component: field(ARCHETYPE_PID_PIPE, "kind").into(),
            component_type: Some(components::Text::name()),
        }
    }

    /// A pipe polyline through the given diagram points (y down).
    #[inline]
    pub fn new(points: impl IntoIterator<Item = impl Into<datatypes::Vec2D>>) -> Self {
        let strip = components::LineStrip2D::from_iter(points);
        Self {
            points: try_serialize_field::<components::LineStrip2D>(
                Self::descriptor_points(),
                [strip],
            ),
            kind: None,
        }
    }

    /// Sets the line kind: `"process"` (solid) or `"signal"` (dashed).
    #[inline]
    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = try_serialize_field::<components::Text>(
            Self::descriptor_kind(),
            [components::Text(kind.into().into())],
        );
        self
    }
}

impl AsComponents for PidPipe {
    #[inline]
    fn as_serialized_batches(&self) -> Vec<SerializedComponentBatch> {
        [self.points.clone(), self.kind.clone()]
            .into_iter()
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_serializes_one_batch() {
        let pipe = PidPipe::new([[0.0, 0.0], [10.0, 0.0], [10.0, 5.0]]);
        assert_eq!(pipe.as_serialized_batches().len(), 1);
    }

    #[test]
    fn builder_with_kind_serializes_two_batches() {
        let pipe = PidPipe::new([[0.0, 0.0], [10.0, 0.0]]).with_kind("signal");
        assert_eq!(pipe.as_serialized_batches().len(), 2);
    }

    #[test]
    fn descriptors_are_namespaced() {
        assert_eq!(
            PidPipe::descriptor_points().component.as_str(),
            "simplant.archetypes.PidPipe:points"
        );
        assert_eq!(
            PidPipe::descriptor_kind().component.as_str(),
            "simplant.archetypes.PidPipe:kind"
        );
    }
}
