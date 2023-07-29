use crate::{errors::Error, solver::CellOption, GroundClicked, ProgramState};

use super::{cells::CellClicked, compute::compute_from_state, *};
use bevy_egui_controls::ControlPanel;
use strum::{EnumIs, EnumIter};

pub struct AutomaticPlugin;
impl Plugin for AutomaticPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			(
				handle_plane_clicked,
				handle_cell_clicked,
				compute_from_state,
			)
				.in_set(OnUpdate(ProgramState::Automatic)),
		);
	}
}

/// WHat happens when you click on a cell.
/// Specific to **automatic** mode.
#[derive(
	Resource, Clone, Copy, Default, PartialEq, Eq, EnumIs, strum::Display, EnumIter, ControlPanel,
)]
pub enum ToggleAction {
	#[strum(serialize = "Enable / Disable [d]")]
	#[default]
	ToggleCellEnabled,

	#[strum(serialize = "Target / Untarget [t]")]
	TargetCell,
}

impl From<ToggleAction> for KeyCode {
	fn from(value: ToggleAction) -> Self {
		match value {
			ToggleAction::TargetCell => KeyCode::T,
			ToggleAction::ToggleCellEnabled => KeyCode::D,
		}
	}
}

fn handle_plane_clicked(mut click: EventReader<GroundClicked>, state: ResMut<SharedState>) {
	if click.iter().next().is_some() {
		debug!("Plane clicked");
		let state = state.into_inner();
		state.remove_start();
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
