//! Visualizer that collects every `PidSymbol` entity visible in the view.

use re_log_types::EntityPath;
use re_sdk_types::blueprint::components::VisualizerInstructionId;
use re_sdk_types::{Archetype as _, components};
use re_viewer_context::{
    AppOptions, IdentifiedViewSystem, SingleRequiredComponentConstraint, ViewContext,
    ViewContextCollection, ViewQuery, ViewSystemExecutionError, ViewSystemIdentifier,
    VisualizerExecutionOutput, VisualizerQueryInfo, VisualizerSystem,
};
use sp_types::PidSymbol;

use crate::visualizer::PlacedSymbol;

/// One `PidSymbol` entity, resolved into a drawable [`PlacedSymbol`].
pub struct PidSymbolDatum {
    /// Entity that logged the `PidSymbol` (e.g. `pid/P-101`).
    pub entity_path: EntityPath,

    /// Which visualizer instruction produced this datum.
    pub instruction_id: VisualizerInstructionId,

    /// The drawable symbol, in diagram coordinates.
    pub placed: PlacedSymbol,

    /// Entity whose latest `Scalars` value feeds the live-value text shown
    /// under the label.
    pub linked_tag: Option<EntityPath>,
}

/// Output of [`PidSymbolVisualizer`], consumed by the view's `ui`.
pub type PidSymbolVisualizerOutput = Vec<PidSymbolDatum>;

/// Collects `PidSymbol` entities and resolves their latest values.
#[derive(Default)]
pub struct PidSymbolVisualizer;

impl IdentifiedViewSystem for PidSymbolVisualizer {
    fn identifier() -> ViewSystemIdentifier {
        "PidSymbolVisualizer".into()
    }
}

impl VisualizerSystem for PidSymbolVisualizer {
    fn visualizer_query_info(&self, _app_options: &AppOptions) -> VisualizerQueryInfo {
        VisualizerQueryInfo {
            relevant_archetype: Some(PidSymbol::name()),
            constraints: SingleRequiredComponentConstraint::new::<components::Position2D>(
                &PidSymbol::descriptor_position(),
            )
            .into(),
            queried: [
                PidSymbol::descriptor_position(),
                PidSymbol::descriptor_symbol_id(),
                PidSymbol::descriptor_label(),
                PidSymbol::descriptor_half_size(),
                PidSymbol::descriptor_linked_tag(),
            ]
            .into_iter()
            .collect(),
        }
    }

    fn execute(
        &self,
        ctx: &ViewContext<'_>,
        query: &ViewQuery<'_>,
        _context_systems: &ViewContextCollection,
    ) -> Result<VisualizerExecutionOutput, ViewSystemExecutionError> {
        let output = VisualizerExecutionOutput::default();
        let latest_at = re_chunk_store::LatestAtQuery::new(query.timeline, query.latest_at);
        let mut data: PidSymbolVisualizerOutput = Vec::new();

        for (data_result, instruction) in query.iter_visualizer_instruction_for(Self::identifier())
        {
            let results = ctx.recording_engine().cache().latest_at(
                &latest_at,
                &data_result.entity_path,
                [
                    PidSymbol::descriptor_position().component,
                    PidSymbol::descriptor_symbol_id().component,
                    PidSymbol::descriptor_label().component,
                    PidSymbol::descriptor_half_size().component,
                    PidSymbol::descriptor_linked_tag().component,
                ],
            );

            let Some(position) = results.component_mono::<components::Position2D>(
                PidSymbol::descriptor_position().component,
            ) else {
                continue;
            };
            let Some(symbol_id) = results
                .component_mono::<components::Text>(PidSymbol::descriptor_symbol_id().component)
            else {
                continue;
            };

            let label = results
                .component_mono::<components::Text>(PidSymbol::descriptor_label().component)
                .map_or_else(
                    || {
                        data_result
                            .entity_path
                            .last()
                            .map(|part| part.ui_string())
                            .unwrap_or_default()
                    },
                    |text| text.as_str().to_owned(),
                );
            let half_size = results
                .component_mono::<components::HalfSize2D>(
                    PidSymbol::descriptor_half_size().component,
                )
                .map_or(egui::vec2(96.0, 96.0), |half| {
                    egui::vec2(half.x() * 2.0, half.y() * 2.0)
                });
            let linked_tag = results
                .component_mono::<components::EntityPath>(
                    PidSymbol::descriptor_linked_tag().component,
                )
                .map(|path| EntityPath::from(path.as_str()));

            // Latest value of the linked tag, shown under the equipment label.
            let live_value = linked_tag.as_ref().and_then(|tag| {
                ctx.recording_engine()
                    .cache()
                    .latest_at(
                        &latest_at,
                        tag,
                        [re_sdk_types::archetypes::Scalars::descriptor_scalars().component],
                    )
                    .component_mono::<components::Scalar>(
                        re_sdk_types::archetypes::Scalars::descriptor_scalars().component,
                    )
                    .map(|scalar| format!("{:.2}", scalar.0.0))
            });

            let mut placed = PlacedSymbol::new(
                symbol_id.as_str(),
                label,
                egui::pos2(position.x(), position.y()),
                half_size,
            );
            if let Some(live_value) = live_value {
                placed = placed.with_live_value(live_value);
            }

            data.push(PidSymbolDatum {
                entity_path: data_result.entity_path.clone(),
                instruction_id: instruction.id,
                placed,
                linked_tag,
            });
        }

        Ok(output.with_visualizer_data(data))
    }
}
