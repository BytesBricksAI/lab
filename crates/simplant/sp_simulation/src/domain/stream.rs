//! Material streams connecting unit operations.

use serde::{Deserialize, Serialize};

use crate::domain::component::Composition;
use crate::domain::ids::{StreamId, UnitOpId};

/// A material stream with optional upstream and downstream unit operations.
///
/// A stream with `from = None` is a feed; a stream with `to = None` is a product.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialStream {
    id: StreamId,
    from: Option<UnitOpId>,
    to: Option<UnitOpId>,
    composition: Composition,
}

impl MaterialStream {
    /// Creates a material stream.
    pub fn new(
        id: StreamId,
        from: Option<UnitOpId>,
        to: Option<UnitOpId>,
        composition: Composition,
    ) -> Self {
        Self {
            id,
            from,
            to,
            composition,
        }
    }

    /// Stream identifier.
    pub fn id(&self) -> &StreamId {
        &self.id
    }

    /// Upstream unit operation, if any.
    pub fn from(&self) -> Option<&UnitOpId> {
        self.from.as_ref()
    }

    /// Downstream unit operation, if any.
    pub fn to(&self) -> Option<&UnitOpId> {
        self.to.as_ref()
    }

    /// Stream composition.
    pub fn composition(&self) -> &Composition {
        &self.composition
    }

    /// Returns `true` when this stream is a feed (no upstream unit operation).
    pub fn is_feed(&self) -> bool {
        self.from.is_none()
    }

    /// Returns `true` when this stream is a product (no downstream unit operation).
    pub fn is_product(&self) -> bool {
        self.to.is_none()
    }
}
