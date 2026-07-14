"""Hand-written P&ID archetype and blueprint view for the SimPlant native viewer."""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np
import numpy.typing as npt

from ._baseclasses import (
    ComponentBatchMixin,
    ComponentDescriptor,
    DescribedComponentBatch,
)
from .blueprint.api import View, ViewContentsLike, VisualizerLike
from .components import (
    EntityPathBatch,
    HalfSize2DBatch,
    LineStrip2DBatch,
    Position2DBatch,
    TextBatch,
)

if TYPE_CHECKING:
    from collections.abc import Iterable, Mapping, Sequence

    from ._baseclasses import AsComponents
    from .datatypes import BoolLike, EntityPathLike, Utf8Like

    PidViewDefaultsLike = Iterable[AsComponents | Iterable[DescribedComponentBatch]]

_ARCHETYPE_PID_PIPE = "simplant.archetypes.PidPipe"
_ARCHETYPE_PID_SYMBOL = "simplant.archetypes.PidSymbol"
_VIEW_CLASS_IDENTIFIER = "SimPlantPid"
_PID_PIPE_KINDS = ("process", "signal")


def _described(
    batch: ComponentBatchMixin,
    archetype: str,
    field_name: str,
) -> DescribedComponentBatch:
    return batch.described(
        ComponentDescriptor(
            f"{archetype}:{field_name}",
            archetype=archetype,
            component_type=batch.component_type(),
        )
    )


def _validate_vec2(
    value: Sequence[float] | npt.ArrayLike,
    field_name: str,
) -> tuple[float, float]:
    array = np.asarray(value, dtype=np.float64)
    if array.shape != (2,):
        shape = array.shape
        msg = f"{field_name} must be a sequence of exactly 2 floats; got shape {shape}"
        raise ValueError(msg)
    return float(array[0]), float(array[1])


class PidSymbol:
    """
    One piece of equipment placed on a P&ID diagram.

    Mirror of `sp_types::PidSymbol`: center position, Equinor symbol id, optional label,
    half-extents, and linked process-variable entity path.
    """

    def __init__(
        self,
        position: Sequence[float] | npt.ArrayLike,
        symbol_id: str,
        *,
        label: str | None = None,
        half_size: Sequence[float] | npt.ArrayLike | None = None,
        linked_tag: str | None = None,
    ) -> None:
        """
        Create a P&ID symbol instance.

        Parameters
        ----------
        position:
            Center of the symbol in diagram coordinates (y down).
        symbol_id:
            Equinor `engineering-symbols` id, e.g. `"PP007A"`.
        label:
            Equipment tag shown under the symbol, e.g. `"P-101"`.
        half_size:
            Half-extents of the symbol in diagram units.
        linked_tag:
            Entity path of the linked process variable whose latest value is shown
            under the label, e.g. `"tags/P-101/pressure"`.

        """
        self._position = _validate_vec2(position, "position")
        self._symbol_id = symbol_id
        self._label = label
        self._half_size: tuple[float, float] | None
        if half_size is not None:
            self._half_size = _validate_vec2(half_size, "half_size")
        else:
            self._half_size = None
        self._linked_tag = linked_tag

    def as_component_batches(self) -> list[DescribedComponentBatch]:
        """
        Return component batches for logging this symbol.

        Optional fields with value `None` are omitted, matching the Rust `flatten()`
        behavior.
        """
        batches = [
            _described(
                Position2DBatch(list(self._position)),
                _ARCHETYPE_PID_SYMBOL,
                "position",
            ),
            _described(
                TextBatch(self._symbol_id),
                _ARCHETYPE_PID_SYMBOL,
                "symbol_id",
            ),
        ]
        if self._label is not None:
            batches.append(
                _described(
                    TextBatch(self._label),
                    _ARCHETYPE_PID_SYMBOL,
                    "label",
                ),
            )
        if self._half_size is not None:
            batches.append(
                _described(
                    HalfSize2DBatch(list(self._half_size)),
                    _ARCHETYPE_PID_SYMBOL,
                    "half_size",
                ),
            )
        if self._linked_tag is not None:
            batches.append(
                _described(
                    EntityPathBatch(self._linked_tag),
                    _ARCHETYPE_PID_SYMBOL,
                    "linked_tag",
                ),
            )
        return batches


class PidPipe:
    """
    A process line on the P&ID: a polyline through diagram points (y down).

    Mirror of `sp_types::PidPipe`: a single line strip in diagram coordinates.
    """

    def __init__(
        self,
        points: Sequence[Sequence[float]] | npt.ArrayLike,
        *,
        kind: str | None = None,
    ) -> None:
        """
        Create a P&ID pipe instance.

        Parameters
        ----------
        points:
            Polyline vertices in diagram coordinates (y down). At least two points,
            each a pair of floats.
        kind:
            Line kind: `"process"` (solid, the default) or `"signal"` (dashed,
            ISA-5.1 instrument signal line).

        """
        array = np.asarray(points, dtype=np.float64)
        if array.ndim != 2 or array.shape[1] != 2 or array.shape[0] < 2:
            shape = array.shape
            prefix = "points must be a sequence of at least 2 (x, y) pairs"
            msg = f"{prefix}; got shape {shape}"
            raise ValueError(msg)
        if kind is not None and kind not in _PID_PIPE_KINDS:
            msg = f"kind must be one of {_PID_PIPE_KINDS}; got {kind!r}"
            raise ValueError(msg)
        self._points = [[float(x), float(y)] for x, y in array]
        self._kind = kind

    def as_component_batches(self) -> list[DescribedComponentBatch]:
        """
        Return component batches for logging this pipe.

        One batch for the polyline points, plus the line kind when given.
        """
        batches = [
            _described(
                LineStrip2DBatch(self._points),
                _ARCHETYPE_PID_PIPE,
                "points",
            ),
        ]
        if self._kind is not None:
            batches.append(
                _described(
                    TextBatch(self._kind),
                    _ARCHETYPE_PID_PIPE,
                    "kind",
                ),
            )
        return batches


class PidView(View):
    """
    Native SimPlant P&ID view (`"SimPlantPid"`) for blueprint layouts.

    Displays entities logged with [`PidSymbol`][simplant_lab.PidSymbol] and
    [`PidPipe`][simplant_lab.PidPipe].
    """

    def __init__(
        self,
        *,
        origin: EntityPathLike = "/",
        contents: ViewContentsLike = "$origin/**",
        name: Utf8Like | None = None,
        visible: BoolLike | None = None,
        defaults: PidViewDefaultsLike | None = None,
        overrides: Mapping[
            EntityPathLike,
            VisualizerLike | Iterable[VisualizerLike],
        ]
        | None = None,
    ) -> None:
        """
        Construct a blueprint for a new PidView.

        Parameters
        ----------
        origin:
            The `EntityPath` to use as the origin of this view.
            All other entities will be transformed to be displayed relative to this
            origin.
        contents:
            The contents of the view specified as a query expression.
            This is either a single expression, or a list of multiple expressions.
            See [simplant_lab.blueprint.archetypes.ViewContents][].
        name:
            The display name of the view.
        visible:
            Whether this view is visible.

            Defaults to true if not specified.
        defaults:
            List of archetypes or (described) component batches to add to the
            view.
            When an archetype in the view is missing a component included in this set,
            the value of default will be used instead of the normal fallback for the
            visualizer.

            Note that an archetype's required components typically don't have any
            effect.
            It is recommended to use the archetype's `from_fields` method instead and
            only specify the fields that you need.
        overrides:
            Dictionary of visualizer overrides to apply to the view. The key is the path
            to the entity where the override should be applied. The value is a list of
            visualizers which should be enabled for that entity, or a single visualizer.

            Each visualizer can be configured with arbitrary overrides and mappings.

            For any entity mentioned in this map, visualizers are no longer added
            automatically based on the entity's components.

            Important note: the path must be a fully qualified entity path starting at
            the root. The override paths do not yet support `$origin` relative paths or
            glob expressions.
            This will be addressed in
            <https://github.com/rerun-io/rerun/issues/6673>.

        """
        super().__init__(
            class_identifier=_VIEW_CLASS_IDENTIFIER,
            origin=origin,
            contents=contents,
            name=name,
            visible=visible,
            defaults=defaults,
            overrides=overrides,
        )
