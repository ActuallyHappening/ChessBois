use crate::{
	errors::Error,
	solver::{algs::Computation, CellOption},
	ChessPoint, GroundClicked, ProgramState,
};

use super::{cells::CellClicked, *};
use bevy_egui::egui::Ui;

use strum::{EnumIs, EnumIter, IntoEnumIterator};

pub struct AutomaticPlugin;
impl Plugin for AutomaticPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems((handle_plane_clicked, handle_cell_clicked).in_set(OnUpdate(ProgramState::Automatic)));
	}
}

/// WHat happens when you click on a cell.
/// Specific to **automatic** mode.
#[derive(Resource, Clone, Copy, Default, PartialEq, Eq, EnumIs, strum::Display, EnumIter)]
pub enum ToggleAction {
	#[strum(serialize = "Enable / Disable [d]")]
	#[default]
	ToggleCellEnabled,

	#[strum(serialize = "Target / Untarget [t]")]
	TargetCell,
}

impl ToggleAction {
	/// Inserts [ToggleAction] resource to toggle cell enabled
	pub fn sys_toggle_enabled(mut commands: Commands) {
		commands.insert_resource(ToggleAction::ToggleCellEnabled);
	}
	pub fn sys_toggle_targets(mut commands: Commands) {
		commands.insert_resource(ToggleAction::TargetCell);
	}

	pub fn render(&mut self, ui: &mut Ui) {
		use bevy_egui::egui::*;
		ui.label("What happens when you click a cell?");
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

	pub fn change_toggle_action_hotkeys(
		keys: Res<Input<KeyCode>>,
		mut selected_action: ResMut<Self>,
	) {
		for key in ToggleAction::iter() {
			if keys.just_pressed(KeyCode::from(key)) {
				*selected_action = key;
			}
		}
	}
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
		state.into_inner().remove_start();
	}
}

/// When cell clicked
fn handle_cell_clicked(
	mut event: EventReader<CellClicked>,
	state: ResMut<SharedState>,
	mut commands: Commands,
) {
	if let Some(CellClicked(point)) = event.iter().next() {
		debug!("Cell clicked in auto mode, toggling: {:?}", point);

		let state = state.into_inner();
		match state.get(point) {
			Some(current_point) => match state.on_click {
				ToggleAction::ToggleCellEnabled => match current_point {
					CellOption::Available { .. } => {
						state.rm(*point);
						state.remove_start();
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

		state.invalidate();
	}
}
