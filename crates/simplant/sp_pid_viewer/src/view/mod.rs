//! `PidView`: the P&ID view class for the SimPlant Lab viewer.
//!
//! Renders every `PidSymbol` and `PidPipe` entity in the view on a
//! [`crate::visualizer::PidCanvas`] and wires symbol hover/click into the
//! viewer's global selection.

mod pipe_visualizer_system;
mod visualizer_system;

pub use pipe_visualizer_system::{PidPipeDatum, PidPipeVisualizer, PidPipeVisualizerOutput};
pub use visualizer_system::{PidSymbolDatum, PidSymbolVisualizer, PidSymbolVisualizerOutput};

use re_log_types::EntityPath;
use re_sdk_types::ViewClassIdentifier;
use re_ui::Help;
use re_viewer_context::external::re_entity_db::InstancePath;
use re_viewer_context::{
    DataResultInteractionAddress, IdentifiedViewSystem as _, IndicatedEntities, Item,
    MissingChunkReporter, PerVisualizerType, RecommendedVisualizers, SystemExecutionOutput,
    ViewClass, ViewClassLayoutPriority, ViewClassRegistryError, ViewQuery, ViewSpawnHeuristics,
    ViewState, ViewSystemExecutionError, ViewSystemIdentifier, ViewSystemRegistrator,
    ViewerContext, VisualizableReason,
};

use crate::visualizer::{PidCanvas, PlacedPipe, PlacedSymbol};

/// The P&ID view: engineering symbols on an interactive canvas, with live
/// values from the store.
#[derive(Default)]
pub struct PidView;

impl ViewClass for PidView {
    fn identifier() -> ViewClassIdentifier {
        crate::VIEW_CLASS_IDENTIFIER.into()
    }

    fn display_name(&self) -> &'static str {
        "P&ID"
    }

    fn icon(&self) -> &'static re_ui::Icon {
        &re_ui::icons::VIEW_GENERIC
    }

    fn help(&self, _os: egui::os::OperatingSystem) -> Help {
        Help::new("P&ID view").markdown(
            "Piping & instrumentation diagram.\n\n\
             Drag to pan, scroll to zoom, double-click the background to re-fit.\n\
             Click an equipment to select it.",
        )
    }

    fn on_register(
        &self,
        system_registry: &mut ViewSystemRegistrator<'_>,
    ) -> Result<(), ViewClassRegistryError> {
        system_registry.register_visualizer::<PidSymbolVisualizer>()?;
        system_registry.register_visualizer::<PidPipeVisualizer>()
    }

    fn new_state(&self) -> Box<dyn ViewState> {
        Box::new(())
    }

    fn layout_priority(&self) -> ViewClassLayoutPriority {
        Default::default()
    }

    fn spawn_heuristics(
        &self,
        ctx: &ViewerContext<'_>,
        include_entity: &dyn Fn(&EntityPath) -> bool,
    ) -> ViewSpawnHeuristics {
        let has_symbols = ctx
            .visualizable_entities_per_visualizer
            .get(&PidSymbolVisualizer::identifier())
            .is_some_and(|entities| entities.keys().any(include_entity));
        let has_pipes = ctx
            .visualizable_entities_per_visualizer
            .get(&PidPipeVisualizer::identifier())
            .is_some_and(|entities| entities.keys().any(include_entity));
        if has_symbols || has_pipes {
            ViewSpawnHeuristics::root()
        } else {
            ViewSpawnHeuristics::empty()
        }
    }

    fn recommended_visualizers_for_entity(
        &self,
        entity_path: &EntityPath,
        visualizers: &[(ViewSystemIdentifier, &VisualizableReason)],
        indicated_entities_per_visualizer: &PerVisualizerType<&IndicatedEntities>,
    ) -> RecommendedVisualizers {
        // Both visualizers report every entity in the view as visualizable, so
        // indication — which archetype the entity actually logged — is what
        // separates symbols from pipes (same pattern as the time series view).
        RecommendedVisualizers::default_many(visualizers.iter().filter_map(|(viz, _)| {
            indicated_entities_per_visualizer
                .get(viz)?
                .contains(entity_path)
                .then_some(*viz)
        }))
    }

    fn ui(
        &self,
        ctx: &ViewerContext<'_>,
        _missing_chunk_reporter: &MissingChunkReporter,
        ui: &mut egui::Ui,
        _state: &mut dyn ViewState,
        query: &ViewQuery<'_>,
        system_output: SystemExecutionOutput,
    ) -> Result<(), ViewSystemExecutionError> {
        let empty_symbols = PidSymbolVisualizerOutput::new();
        let symbol_data = system_output
            .visualizer_data::<PidSymbolVisualizerOutput>(PidSymbolVisualizer::identifier())
            .unwrap_or(&empty_symbols);

        let empty_pipes = PidPipeVisualizerOutput::new();
        let pipe_data = system_output
            .visualizer_data::<PidPipeVisualizerOutput>(PidPipeVisualizer::identifier())
            .unwrap_or(&empty_pipes);

        let placed: Vec<PlacedSymbol> = symbol_data
            .iter()
            .map(|datum| datum.placed.clone())
            .collect();
        let pipes: Vec<PlacedPipe> = pipe_data.iter().map(|datum| datum.placed.clone()).collect();
        let canvas = PidCanvas::new(&placed).with_pipes(&pipes).show(ui);

        // Wire every symbol into the viewer's global hover/selection.
        for (datum, response) in symbol_data.iter().zip(&canvas.symbol_responses) {
            ctx.handle_select_hover_drag_interactions(
                response,
                Item::DataResult(DataResultInteractionAddress {
                    view_id: query.view_id,
                    instance_path: InstancePath::entity_all(datum.entity_path.clone()),
                    visualizer: Some(datum.instruction_id),
                }),
                false,
            );
        }
        if canvas.hovered.is_none() {
            ctx.handle_select_hover_drag_interactions(
                &canvas.response,
                Item::View(query.view_id),
                false,
            );
        }

        Ok(())
    }
}
