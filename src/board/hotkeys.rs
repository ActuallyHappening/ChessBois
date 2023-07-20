use super::{
	automatic::ToggleAction, ui::control_ui_hotkeys_automatic, viz_colours::colour_hotkeys,
};
use crate::ProgramState;
use bevy::prelude::*;

pub struct HotkeysPlugin;
impl Plugin for HotkeysPlugin {
	fn build(&self, app: &mut App) {
		app
			// automatic
			.add_systems(
				(
					ToggleAction::change_toggle_action_hotkeys,
					control_ui_hotkeys_automatic,
				)
					.in_set(OnUpdate(ProgramState::Automatic)),
			)
			// manual
			.add_systems((colour_hotkeys,).in_set(OnUpdate(ProgramState::Manual)))
			// both
			.add_systems(());
	}
}
