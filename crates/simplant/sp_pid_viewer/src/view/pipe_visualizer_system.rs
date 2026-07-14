//! Visualizer that collects every `PidPipe` entity visible in the view.

use re_log_types::EntityPath;
use re_sdk_types::blueprint::components::VisualizerInstructionId;
use re_sdk_types::{Archetype as _, components};
use re_viewer_context::{
    AppOptions, IdentifiedViewSystem, SingleRequiredComponentConstraint, ViewContext,
    ViewContextCollection, ViewQuery, ViewSystemExecutionError, ViewSystemIdentifier,
    VisualizerExecutionOutput, VisualizerQueryInfo, VisualizerSystem,
};
use sp_types::PidPipe;

use crate::visualizer::{PipeKind, PlacedPipe};

/// One `PidPipe` entity, resolved into a drawable [`PlacedPipe`].
pub struct PidPipeDatum {
    /// Entity that logged the `PidPipe` (e.g. `pid/pipes/TK-101-P-101`).
    pub entity_path: EntityPath,

    /// Which visualizer instruction produced this datum.
    pub instruction_id: VisualizerInstructionId,

    /// The drawable pipe polyline, in diagram coordinates.
    pub placed: PlacedPipe,
}

/// Output of [`PidPipeVisualizer`], consumed by the view's `ui`.
pub type PidPipeVisualizerOutput = Vec<PidPipeDatum>;

/// Collects `PidPipe` entities into drawable polylines.
#[derive(Default)]
pub struct PidPipeVisualizer;

impl IdentifiedViewSystem for PidPipeVisualizer {
    fn identifier() -> ViewSystemIdentifier {
        "PidPipeVisualizer".into()
    }
}

impl VisualizerSystem for PidPipeVisualizer {
    fn visualizer_query_info(&self, _app_options: &AppOptions) -> VisualizerQueryInfo {
        VisualizerQueryInfo {
            relevant_archetype: Some(PidPipe::name()),
            constraints: SingleRequiredComponentConstraint::new::<components::LineStrip2D>(
                &PidPipe::descriptor_points(),
            )
            .into(),
            queried: [PidPipe::descriptor_points(), PidPipe::descriptor_kind()]
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
        let mut data: PidPipeVisualizerOutput = Vec::new();

        for (data_result, instruction) in query.iter_visualizer_instruction_for(Self::identifier())
        {
            let results = ctx.recording_engine().cache().latest_at(
                &latest_at,
                &data_result.entity_path,
                [
                    PidPipe::descriptor_points().component,
                    PidPipe::descriptor_kind().component,
                ],
            );

            let Some(strip) = results
                .component_mono::<components::LineStrip2D>(PidPipe::descriptor_points().component)
            else {
                continue;
            };

            let points: Vec<egui::Pos2> =
                strip.0.iter().map(|v| egui::pos2(v.x(), v.y())).collect();
            if points.len() < 2 {
                continue;
            }

            // Anything other than an explicit "signal" draws as a process line.
            let kind = results
                .component_mono::<components::Text>(PidPipe::descriptor_kind().component)
                .map_or(PipeKind::Process, |text| match text.as_str() {
                    "signal" => PipeKind::Signal,
                    _ => PipeKind::Process,
                });

            data.push(PidPipeDatum {
                entity_path: data_result.entity_path.clone(),
                instruction_id: instruction.id,
                placed: PlacedPipe::new(points).with_kind(kind),
            });
        }

        Ok(output.with_visualizer_data(data))
    }
}
