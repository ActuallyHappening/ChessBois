use crate::{ProgramState, ChessPoint, solver::{pieces::StandardKnight, algs::Computation, CellOption}, GroundClicked};

use super::{
	manual::{add_default_manual_viz_colour, add_empty_manual_moves, ManualNextCell},
	*, visualization::{sys_despawn_visualization, VisualizationComponent, despawn_visualization, spawn_visualization}, cells::{sys_despawn_markers, CellMarker, MarkerMarker, despawn_markers, spawn_markers, despawn_cells, spawn_cells}, viz_colours::VizColour,
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
			.add_event::<NewOptions>()
			.add_event::<ComputationResult>()
			// update
			.add_systems(
				(
					handle_automatic_computation,
					update_cache_from_computation,
					handle_spawning_visualization,
					handle_new_options,
					handle_plane_clicked,
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
	mut options_events: EventReader<NewOptions>,
	old_options: Res<CurrentOptions>,

	cells: Query<Entity, (With<CellMarker>, With<ChessPoint>)>,
	viz: Query<Entity, With<VisualizationComponent>>,
	markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,

	mut commands: Commands,
	mut mma: ResSpawning,
) {
	if let Some(options) = options_events.iter().next() {
		let options = options.clone().into_options();
		let old_options = old_options.clone().into_options();

		if options.force_update {
			info!("Force updating ...")
		}

		if options == old_options && !options.force_update {
			// info!("Ignoring update, options are the same");
			return;
		}

		despawn_visualization(&mut commands, viz);

		// markers
		despawn_markers(&mut commands, markers);
		spawn_markers(&options, &mut commands, &mut mma);

		// if BoardOptions changed, despawn + re-spawn cells
		if options.options != old_options.options || options.force_update {
			// info!("BoardOptions changed, de-spawning + re-spawning cells & markers");
			despawn_cells(&mut commands, cells);

			spawn_cells(&options, &mut commands, &mut mma);
		}

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

		options_events.clear();
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
	options: Res<CurrentOptions>,
	mut update_board: EventWriter<NewOptions>,
) {
	if click.iter().next().is_some() {
		debug!("Plane clicked");

		update_board.send(NewOptions::from_options(Options {
			selected_start: None,
			..options.clone().into_options()
		}));
	}
}

fn toggle_cell_availability(
	In(event): In<ListenedEvent<Click>>,
	cells: Query<(&Handle<StandardMaterial>, &ChessPoint)>,
	options: ResMut<CurrentOptions>,

	mut update_board: EventWriter<NewOptions>,
	mut update_manual: EventWriter<ManualNextCell>,
) -> Bubble {
	let (_mat, point) = cells.get(event.target).unwrap();

	let mut options = options.current.clone();
	match options.options.get(point) {
		Some(CellOption::Available) => {
			// let material = materials.get_mut(mat).unwrap();
			// material.base_color = CELL_DISABLED_COLOUR;

			options.options.rm(*point);
			update_board.send(NewOptions::from_options(options));
			update_manual.send(ManualNextCell::from(*point));
		}
		Some(CellOption::Unavailable) => {
			// let material = materials.get_mut(mat).unwrap();
			// material.base_color = point.get_standard_colour();

			options.options.add(*point);
			update_board.send(NewOptions::from_options(options));
		}
		None => panic!("Tried to change availability of cell that doesn't exist"),
	}
	Bubble::Up
}
