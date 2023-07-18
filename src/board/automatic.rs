use crate::{
	errors::Error,
	solver::{algs::Computation, pieces::StandardKnight, CellOption},
	ChessPoint, GroundClicked, ProgramState,
};

use super::{
	cells::{
		despawn_cells, despawn_markers, spawn_cells, spawn_markers, sys_despawn_markers, CellClicked,
		CellMarker, MarkerMarker,
	},
	manual::{add_default_manual_viz_colour, add_empty_manual_moves},
	visualization::{
		despawn_visualization, spawn_visualization, sys_despawn_visualization, VisualizationComponent,
		VizOptions,
	},
	viz_colours::VizColour,
	*,
};
use bevy_egui::egui::Ui;
use cached_info::*;
use compute::*;

pub use compute::ComputationResult;

use strum::{EnumIs, EnumIter, IntoEnumIterator};

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
					VizOptions::sys_with_numbers,
					// ToggleAction::sys_toggle_enabled,
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
			)
			// startup
			.add_startup_system(ToggleAction::sys_toggle_enabled);
	}
}

/// WHat happens when you click on a cell.
/// Specific to **automatic** mode.
#[derive(Resource, Clone, Copy, PartialEq, Eq, EnumIs, strum::Display, EnumIter)]
pub enum ToggleAction {
	#[strum(serialize = "Enable / Disable")]
	ToggleCellEnabled,
	#[strum(serialize = "Target / Untarget")]
	TargetCell,
}

impl ToggleAction {
	pub fn sys_toggle_enabled(mut commands: Commands) {
		commands.insert_resource(ToggleAction::ToggleCellEnabled);
	}
	pub fn sys_toggle_targets(mut commands: Commands) {
		commands.insert_resource(ToggleAction::TargetCell);
	}

	pub fn render(&mut self, ui: &mut Ui) {
		use bevy_egui::egui::*;
		ui.horizontal_wrapped(|ui| {
			for action in ToggleAction::iter() {
				let mut text = RichText::new(action.to_string());
				if action == *self {
					text = text.color(Color32::GREEN);
				}
				if ui.button(text).clicked() {
					*self = action;
				}
			}
		});
	}
}

/// Decides what happens when [NewOptions] event comes in.
/// Triggers computation
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
		spawn_cells(options, &mut commands, &mut mma);

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
	viz_options: Res<VizOptions>,

	mut mma: ResSpawning,
) {
	if let Some(solution) = solutions.iter().next() {
		let (solution, options) = solution.clone().get();
		if &options != current_options.as_options() {
			trace!("Not rendering visualization for computation of non-valid state");
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
				&viz_options,
			);
		}

		solutions.clear()
	}
}

fn handle_plane_clicked(
	mut click: EventReader<GroundClicked>,
	options: ResMut<CurrentOptions>,
	mut commands: Commands,
	visualization: Query<Entity, With<VisualizationComponent>>,
) {
	if click.iter().next().is_some() {
		debug!("Plane clicked");

		let options = options.into_inner();
		options.current.selected_start = None;

		despawn_visualization(&mut commands, visualization)
	}
}

/// When cell clicked
fn handle_cell_clicked(
	mut event: EventReader<CellClicked>,
	options: ResMut<CurrentOptions>,
	selected_action: Res<ToggleAction>,

	mut commands: Commands,
) {
	if let Some(CellClicked(point)) = event.iter().next() {
		debug!("Cell clicked in auto mode, toggling: {:?}", point);

		let options = options.into_inner();
		match options.get(point) {
			Some(current_point) => match selected_action.into_inner() {
				ToggleAction::ToggleCellEnabled => match current_point {
					CellOption::Available { .. } => {
						options.rm(*point);
						options.current.selected_start = None;
					}
					CellOption::Unavailable => {
						options.add(*point);
					}
				},
				ToggleAction::TargetCell => {
					match current_point {
						CellOption::Available { .. } => {
							info!("Targetting point {}", *point);
							options.toggle_target(*point);
						}
						CellOption::Unavailable => {
							//
						}
					}
				}
			},
			None => {
				let err_msg = format!("Cell {:?} is out of bounds", point);
				warn!("{}", err_msg);
				commands.insert_resource(Error::new(err_msg));
			}
		}

		options.requires_updating();
	}
}
