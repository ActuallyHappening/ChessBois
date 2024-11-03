use crate::{errors::Error, solver::CellOption, ProgramState};

use super::{
	compute::compute_from_state,
	hotkeys::Hotkeyable,
	squares::{CellClicked, CellHovered},
	*,
};
use bevy_egui_controls::ControlPanel;
use strum::{EnumIs, EnumIter};

mod summary;

pub struct AutomaticPlugin;
impl Plugin for AutomaticPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			(
				compute_from_state,
				handle_cell_clicked,
				highlight_hovered_cell,
			)
				.in_set(OnUpdate(ProgramState::Automatic)),
		);
	}
}

/// WHat happens when you click on a cell.
/// Specific to **automatic** mode.
#[derive(
	Resource,
	Clone,
	Copy,
	Default,
	PartialEq,
	Eq,
	EnumIs,
	strum::Display,
	EnumIter,
	Debug,
	ControlPanel,
	Reflect,
	FromReflect,
)]
pub enum ToggleAction {
	#[strum(serialize = "Enable / Disable [d]")]
	#[default]
	ToggleCellEnabled,

	#[strum(serialize = "Target / Untarget [t]")]
	TargetCell,

	#[strum(serialize = "Eliminate cell [e]")]
	EliminateCell,

	#[strum(serialize = "Recommend move [r]")]
	RecommendMove,
}
impl Hotkeyable for ToggleAction {}

impl From<ToggleAction> for KeyCode {
	fn from(value: ToggleAction) -> Self {
		match value {
			ToggleAction::TargetCell => KeyCode::T,
			ToggleAction::ToggleCellEnabled => KeyCode::D,
			ToggleAction::EliminateCell => KeyCode::E,
			ToggleAction::RecommendMove => KeyCode::R,
		}
	}
}

fn highlight_hovered_cell(mut state: ResMut<SharedState>, mut hovered: EventReader<CellHovered>) {
	if let Some(CellHovered(point)) = hovered.iter().next() {
		if state.board_options.is_available(point) {
			state.start = Some(*point);
		} else {
			state.start = None;
		}
	}
}

/// When cell clicked
fn handle_cell_clicked(
	mut event: EventReader<CellClicked>,
	state: ResMut<SharedState>,
	mut commands: Commands,
) {
	if let Some(CellClicked(clicked_cell)) = event.iter().next() {
		info!(?state.on_click, "Cell clicked in auto mode {:?}", clicked_cell);

		let state = state.into_inner();
		match state.get(clicked_cell) {
			Some(current_point) => match state.on_click {
				ToggleAction::ToggleCellEnabled => match current_point {
					CellOption::Available { .. } => {
						state.rm(*clicked_cell);
						state.remove_start();
						state.invalidate();
					}
					CellOption::Unavailable => {
						state.make_available(*clicked_cell);
					}
					_ => {
						debug!("Ignoring click");
					}
				},
				ToggleAction::TargetCell => match current_point {
					CellOption::Available { .. } => {
						info!("Targetting point {}", *clicked_cell);
						state.toggle_target(*clicked_cell);
						state.invalidate();
					}
					CellOption::Unavailable | CellOption::Eliminated => {
						debug!("Ignoring click");
					}
				},
				ToggleAction::EliminateCell => match current_point {
					CellOption::Available { .. } | CellOption::Unavailable => {
						info!("Eliminating point {}", *clicked_cell);
						state.eliminate(clicked_cell);
						state.invalidate();
					}
					CellOption::Eliminated => {
						info!("Un-eliminating point {}", *clicked_cell);
						state.make_available(*clicked_cell);
						state.invalidate();
					}
				},
				ToggleAction::RecommendMove => {
					// Confirm previous move
					if let Some(previous_cell) = state.last_clicked_recommended_move {
						if !state.is_available(&previous_cell) {
							info!("Last recommendation not available");
							state.last_clicked_recommended_move = None;
							return;
						}
					}
					match state.last_clicked_recommended_move {
						Some(previous_cell) => {
							// assuming its valid
							info!("Adding a new recommending new move");
							state.add_recommended_move(crate::solver::Move {
								from: previous_cell,
								to: *clicked_cell,
							});
							state.last_clicked_recommended_move = None;
							state.invalidate();
						}
						None => {
							if !state.is_available(clicked_cell) {
								info!("Can't recommend non-available cell");
								return;
							};
							info!("Starting a new recommendation move ...");
							state.last_clicked_recommended_move = Some(*clicked_cell);
						}
					}
				}
			},
			None => {
				let err_msg = format!("Cell {:?} is out of bounds", clicked_cell);
				warn!("{}", err_msg);
				commands.insert_resource(Error::new(err_msg));
				panic!("Cell out of bounds");
			}
		}
	}
}
