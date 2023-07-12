use crate::{ProgramState, ChessPoint, solver::{pieces::StandardKnight, algs::Computation, CellOption}, GroundClicked};

use super::{
	manual::{add_default_manual_viz_colour, add_empty_manual_moves, ManualNextCell},
	*, visualization::{sys_despawn_visualization, VisualizationComponent, despawn_visualization, spawn_visualization}, cells::{sys_despawn_markers, CellMarker, MarkerMarker, despawn_markers, spawn_markers, despawn_cells, spawn_cells, CellClicked}, viz_colours::VizColour,
};
use cached_info::*;
use compute::*;

pub use compute::ComputationResult;

pub mod cached_info;
mod compute;

pub struct AutomaticState;
impl Plugin for AutomaticState {
	fn build(&self, app: &mut App) {
		app
			.add_event::<ComputationResult>()
			// update
			.add_systems(
				(
					handle_automatic_computation,
					update_cache_from_computation,
					handle_spawning_visualization,
					handle_new_options,
					handle_plane_clicked,
					handle_cell_clicked,
				)
					.in_set(OnUpdate(ProgramState::Automatic)),
			)
			// enter
			.add_systems(
				(
					sys_despawn_visualization,
					sys_despawn_markers,
					add_empty_manual_moves,
					add_default_manual_viz_colour,
				)
					.in_schedule(OnEnter(ProgramState::Automatic)),
			)
			// exit
			.add_systems(
				(
					sys_despawn_visualization,
					sys_despawn_markers,
					add_empty_manual_moves,
				)
					.in_schedule(OnExit(ProgramState::Automatic)),
			);
	}
}

/// Decides what happens when [NewOptions] event comes in
fn handle_new_options(
	options: ResMut<CurrentOptions>,

	cells: Query<Entity, (With<CellMarker>, With<ChessPoint>)>,
	viz: Query<Entity, With<VisualizationComponent>>,
	markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,

	mut commands: Commands,
	mut mma: ResSpawning,
) {
	if options.is_changed() {
		let options = &mut options.into_inner().current;

		if options.force_update {
			info!("Force updating ...")
		}

		despawn_visualization(&mut commands, viz);

		// markers
		despawn_markers(&mut commands, markers);
		spawn_markers(&options, &mut commands, &mut mma);

		// cells
		despawn_cells(&mut commands, cells);
		spawn_cells(&options, &mut commands, &mut mma);

		// begin recomputing visualization
		begin_background_compute(
			options.selected_algorithm,
			&StandardKnight {},
			options.clone(),
			&mut commands,
		);

		// add new options as current
		commands.insert_resource(CurrentOptions::from_options(Options {
			force_update: false,
			..options.clone()
		}));
	}
}

/// Consumes [EventReader<ComputationResult>] and actually spawns concrete visualization if state is correct.
/// ONLY in AUTOMATIC state!
pub fn handle_spawning_visualization(
	mut commands: Commands,
	mut solutions: EventReader<ComputationResult>,
	current_options: Res<CurrentOptions>,

	_viz: Query<Entity, With<VisualizationComponent>>,
	viz_col: Res<VizColour>,

	mut mma: ResSpawning,
) {
	if let Some(solution) = solutions.iter().next() {
		let (solution, options) = solution.clone().get();
		if &options != current_options.as_options() {
			// warn!("Not rendering visualization for computation of non-valid state");
			return;
		}

		if let Computation::Successful {
			solution: moves, ..
		} = solution
		{
			spawn_visualization(
				moves.clone(),
				options.options,
				&mut commands,
				&mut mma,
				vec![*viz_col.into_inner(); moves.len()],
			);
		}

		solutions.clear()
	}
}

fn handle_plane_clicked(
	mut click: EventReader<GroundClicked>,
	options: ResMut<CurrentOptions>,
) {
	if click.iter().next().is_some() {
		debug!("Plane clicked");

		let options = options.into_inner();
		options.current.selected_start = None;
	}
}

fn handle_cell_clicked(
	mut event: EventReader<CellClicked>,

	options: ResMut<CurrentOptions>,
) {
	if let Some(CellClicked(point)) = event.iter().next() {
		debug!("Cell clicked in auto mode, disabling: {:?}", point);

		let options = options.into_inner();
		options.current.options.rm(*point);
	}	
}
