use crate::{
	solver::{algs::Computation, pieces::StandardKnight, CellOption},
	ChessPoint, GroundClicked, ProgramState, errors::Error,
};

use super::{
	cells::{
		despawn_cells, despawn_markers, spawn_cells, spawn_markers, sys_despawn_markers, CellClicked,
		CellMarker, MarkerMarker,
	},
	manual::{add_default_manual_viz_colour, add_empty_manual_moves},
	visualization::{
		despawn_visualization, spawn_visualization, sys_despawn_visualization, VisualizationComponent,
	},
	viz_colours::VizColour,
	*,
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
	options: Res<CurrentOptions>,

	cells: Query<Entity, (With<CellMarker>, With<ChessPoint>)>,
	viz: Query<Entity, With<VisualizationComponent>>,
	markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,

	mut commands: Commands,
	mut mma: ResSpawning,
) {
	if options.is_changed() && options.requires_updating {
		let options = &options.into_inner().current;

		trace!("Automatic updating ...");

		despawn_visualization(&mut commands, viz);

		// markers
		despawn_markers(&mut commands, markers);
		spawn_markers(options, &mut commands, &mut mma);

		// cells
		despawn_cells(&mut commands, cells);
		spawn_cells(options, &mut commands, None, &mut mma);

		// begin recomputing visualization
		if options.selected_start.is_some() {
			begin_background_compute(options.selected_algorithm, &StandardKnight, options.clone());
		} else {
			warn!("Not beginning background compute")
		}

		// add new options as current
		commands.insert_resource(CurrentOptions::from(Options {
			requires_updating: false,
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

fn handle_plane_clicked(mut click: EventReader<GroundClicked>, options: ResMut<CurrentOptions>) {
	if click.iter().next().is_some() {
		debug!("Plane clicked");

		let options = options.into_inner();
		options.current.selected_start = None;
	}
}

fn handle_cell_clicked(mut event: EventReader<CellClicked>, options: ResMut<CurrentOptions>, mut commands: Commands) {
	if let Some(CellClicked(point)) = event.iter().next() {
		debug!("Cell clicked in auto mode, toggling: {:?}", point);

		let options = options.into_inner();
		match options.get(point) {
			Some(CellOption::Available) => {
				options.rm(*point);
				if options.current.selected_start == Some(*point) {
					options.current.selected_start = None;
				}
			}
			Some(CellOption::Unavailable) => {
				options.add(*point);
			}
			None => {
				let err_msg = format!("Cell {:?} is out of bounds", point);
				warn!("{}", err_msg);
				commands.insert_resource(Error::new(err_msg));
			}
		}
		options.requires_updating();
	}
}
