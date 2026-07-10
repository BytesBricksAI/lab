//! `PidSymbol` archetype: one piece of equipment placed on a P&ID diagram.
//!
//! Logged per equipment entity (e.g. `pid/P-101`); the P&ID view renders it
//! with the matching Equinor engineering symbol and the live value of its
//! linked process variable.

use re_sdk_types::{
    Archetype, ArchetypeName, AsComponents, Component as _, ComponentDescriptor,
    SerializedComponentBatch, components, try_serialize_field,
};

use crate::namespace::{ARCHETYPE_PID_SYMBOL, field};

/// A symbol instance on the P&ID: where it sits, which icon renders it, and
/// which process-variable entity it is linked to.
#[derive(Default)]
pub struct PidSymbol {
    /// Center of the symbol, in diagram coordinates (y down).
    pub position: Option<SerializedComponentBatch>,

    /// Equinor `engineering-symbols` id, e.g. `"PP007A"`.
    pub symbol_id: Option<SerializedComponentBatch>,

    /// Equipment tag shown under the symbol, e.g. `"P-101"`.
    pub label: Option<SerializedComponentBatch>,

    /// Half-extents of the symbol, in diagram units.
    pub half_size: Option<SerializedComponentBatch>,

    /// Entity path of the linked process variable, whose latest value is
    /// shown under the symbol's label, e.g. `"tags/P-101/pressure"`.
    pub linked_tag: Option<SerializedComponentBatch>,
}

impl Archetype for PidSymbol {
    fn name() -> ArchetypeName {
        ARCHETYPE_PID_SYMBOL.into()
    }

    fn display_name() -> &'static str {
        "P&ID symbol"
    }

    fn required_components() -> std::borrow::Cow<'static, [ComponentDescriptor]> {
        vec![Self::descriptor_position(), Self::descriptor_symbol_id()].into()
    }

    fn optional_components() -> std::borrow::Cow<'static, [ComponentDescriptor]> {
        vec![
            Self::descriptor_label(),
            Self::descriptor_half_size(),
            Self::descriptor_linked_tag(),
        ]
        .into()
    }
}

impl PidSymbol {
    /// Descriptor for [`Self::position`].
    #[inline]
    pub fn descriptor_position() -> ComponentDescriptor {
        ComponentDescriptor {
            archetype: Some(ARCHETYPE_PID_SYMBOL.into()),
            component: field(ARCHETYPE_PID_SYMBOL, "position").into(),
            component_type: Some(components::Position2D::name()),
        }
    }

    /// Descriptor for [`Self::symbol_id`].
    #[inline]
    pub fn descriptor_symbol_id() -> ComponentDescriptor {
        ComponentDescriptor {
            archetype: Some(ARCHETYPE_PID_SYMBOL.into()),
            component: field(ARCHETYPE_PID_SYMBOL, "symbol_id").into(),
            component_type: Some(components::Text::name()),
        }
    }

    /// Descriptor for [`Self::label`].
    #[inline]
    pub fn descriptor_label() -> ComponentDescriptor {
        ComponentDescriptor {
            archetype: Some(ARCHETYPE_PID_SYMBOL.into()),
            component: field(ARCHETYPE_PID_SYMBOL, "label").into(),
            component_type: Some(components::Text::name()),
        }
    }

    /// Descriptor for [`Self::half_size`].
    #[inline]
    pub fn descriptor_half_size() -> ComponentDescriptor {
        ComponentDescriptor {
            archetype: Some(ARCHETYPE_PID_SYMBOL.into()),
            component: field(ARCHETYPE_PID_SYMBOL, "half_size").into(),
            component_type: Some(components::HalfSize2D::name()),
        }
    }

    /// Descriptor for [`Self::linked_tag`].
    #[inline]
    pub fn descriptor_linked_tag() -> ComponentDescriptor {
        ComponentDescriptor {
            archetype: Some(ARCHETYPE_PID_SYMBOL.into()),
            component: field(ARCHETYPE_PID_SYMBOL, "linked_tag").into(),
            component_type: Some(components::EntityPath::name()),
        }
    }

    /// A symbol at `position` rendered with the given Equinor symbol id.
    #[inline]
    pub fn new(position: impl Into<components::Position2D>, symbol_id: impl Into<String>) -> Self {
        Self::default()
            .with_position(position)
            .with_symbol_id(symbol_id)
    }

    /// Sets the symbol center, in diagram coordinates.
    #[inline]
    pub fn with_position(mut self, position: impl Into<components::Position2D>) -> Self {
        self.position = try_serialize_field::<components::Position2D>(
            Self::descriptor_position(),
            [position.into()],
        );
        self
    }

    /// Sets the Equinor symbol id (e.g. `"PP007A"`).
    #[inline]
    pub fn with_symbol_id(mut self, symbol_id: impl Into<String>) -> Self {
        self.symbol_id = try_serialize_field::<components::Text>(
            Self::descriptor_symbol_id(),
            [components::Text(symbol_id.into().into())],
        );
        self
    }

    /// Sets the equipment tag shown under the symbol.
    #[inline]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = try_serialize_field::<components::Text>(
            Self::descriptor_label(),
            [components::Text(label.into().into())],
        );
        self
    }

    /// Sets the symbol half-extents, in diagram units.
    #[inline]
    pub fn with_half_size(mut self, half_size: impl Into<components::HalfSize2D>) -> Self {
        self.half_size = try_serialize_field::<components::HalfSize2D>(
            Self::descriptor_half_size(),
            [half_size.into()],
        );
        self
    }

    /// Links the equipment to the process-variable entity whose latest value
    /// is shown under the label.
    #[inline]
    pub fn with_linked_tag(mut self, entity_path: impl Into<String>) -> Self {
        self.linked_tag = try_serialize_field::<components::EntityPath>(
            Self::descriptor_linked_tag(),
            [components::EntityPath(entity_path.into().into())],
        );
        self
    }
}

impl AsComponents for PidSymbol {
    #[inline]
    fn as_serialized_batches(&self) -> Vec<SerializedComponentBatch> {
        [
            self.position.clone(),
            self.symbol_id.clone(),
            self.label.clone(),
            self.half_size.clone(),
            self.linked_tag.clone(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_serializes_all_fields() {
        let symbol = PidSymbol::new([10.0, 20.0], "PP007A")
            .with_label("P-101")
            .with_half_size([48.0, 48.0])
            .with_linked_tag("tags/P-101/pressure");
        assert_eq!(symbol.as_serialized_batches().len(), 5);
    }

    #[test]
    fn descriptors_are_namespaced() {
        assert_eq!(
            PidSymbol::descriptor_position().component.as_str(),
            "simplant.archetypes.PidSymbol:position"
        );
    }
}
