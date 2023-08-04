use crate::{ProgramState, solver::CellOption, errors::Error};

use super::{compute::compute_from_state, *, hotkeys::Hotkeyable, squares::{CellClicked, CellHovered}};
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
	Resource, Clone, Copy, Default, PartialEq, Eq, EnumIs, strum::Display, EnumIter, Debug, ControlPanel, Reflect, FromReflect
)]
pub enum ToggleAction {
	#[strum(serialize = "Enable / Disable [d]")]
	#[default]
	ToggleCellEnabled,

	#[strum(serialize = "Target / Untarget [t]")]
	TargetCell,
}
impl Hotkeyable for ToggleAction {}

impl From<ToggleAction> for KeyCode {
	fn from(value: ToggleAction) -> Self {
		match value {
			ToggleAction::TargetCell => KeyCode::T,
			ToggleAction::ToggleCellEnabled => KeyCode::D,
		}
	}
}

fn highlight_hovered_cell(
	mut state: ResMut<SharedState>,
	mut hovered: EventReader<CellHovered>,
) {
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
	if let Some(CellClicked(point)) = event.iter().next() {
		info!("Cell clicked in auto mode, toggling: {:?}", point);

		let state = state.into_inner();
		match state.get(point) {
			Some(current_point) => match state.on_click {
				ToggleAction::ToggleCellEnabled => match current_point {
					CellOption::Available { .. } => {
						state.rm(*point);
						state.remove_start();
						state.invalidate();
					}
					CellOption::Unavailable => {
						state.add(*point);
					}
				},
				ToggleAction::TargetCell => {
					match current_point {
						CellOption::Available { .. } => {
							info!("Targetting point {}", *point);
							state.toggle_target(*point);
							state.invalidate();
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
				panic!("Cell out of bounds");
			}
		}
	}
}

