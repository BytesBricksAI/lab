//! `PidView`: the P&ID view class for the SimPlant Lab viewer.
//!
//! Renders every `PidSymbol` entity in the view on a [`crate::visualizer::PidCanvas`]
//! and wires hover/click into the viewer's global selection.

mod visualizer_system;

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

use crate::visualizer::{PidCanvas, PlacedSymbol};

/// The P&ID view: engineering symbols on an interactive canvas, with live
/// values from the store.
#[derive(Default)]
pub struct PidView;

impl ViewClass for PidView {
    fn identifier() -> ViewClassIdentifier {
        "SimPlantPid".into()
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
        system_registry.register_visualizer::<PidSymbolVisualizer>()
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
        if ctx
            .visualizable_entities_per_visualizer
            .get(&PidSymbolVisualizer::identifier())
            .is_some_and(|entities| entities.keys().any(include_entity))
        {
            ViewSpawnHeuristics::root()
        } else {
            ViewSpawnHeuristics::empty()
        }
    }

    fn recommended_visualizers_for_entity(
        &self,
        _entity_path: &EntityPath,
        visualizers: &[(ViewSystemIdentifier, &VisualizableReason)],
        _indicated_entities_per_visualizer: &PerVisualizerType<&IndicatedEntities>,
    ) -> RecommendedVisualizers {
        if visualizers
            .iter()
            .any(|(viz, _)| *viz == PidSymbolVisualizer::identifier())
        {
            RecommendedVisualizers::default(PidSymbolVisualizer::identifier())
        } else {
            RecommendedVisualizers::empty()
        }
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
        let empty = PidSymbolVisualizerOutput::new();
        let data = system_output
            .visualizer_data::<PidSymbolVisualizerOutput>(PidSymbolVisualizer::identifier())
            .unwrap_or(&empty);

        let placed: Vec<PlacedSymbol> = data.iter().map(|datum| datum.placed.clone()).collect();
        let canvas = PidCanvas::new(&placed).show(ui);

        // Wire every symbol into the viewer's global hover/selection.
        for (datum, response) in data.iter().zip(&canvas.symbol_responses) {
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
