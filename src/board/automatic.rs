use crate::ProgramState;

use super::{compute::compute_from_state, *, hotkeys::Hotkeyable};
use bevy_egui_controls::ControlPanel;
use strum::{EnumIs, EnumIter};

pub struct AutomaticPlugin;
impl Plugin for AutomaticPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			(
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
impl Hotkeyable for ToggleAction {}

impl From<ToggleAction> for KeyCode {
	fn from(value: ToggleAction) -> Self {
		match value {
			ToggleAction::TargetCell => KeyCode::T,
			ToggleAction::ToggleCellEnabled => KeyCode::D,
		}
	}
}


