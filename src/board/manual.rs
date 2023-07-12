use super::*;

pub struct ManualState;
impl Plugin for ManualState {
	fn build(&self, app: &mut App) {
		app
			.add_event::<ManualNextCell>()
			.init_resource::<ManualFreedom>()
			.init_resource::<VizColour>()
			.add_systems(
				(
					state_manual::handle_manual_visualization,
					state_manual::handle_new_manual_selected,
					viz_colours::colour_hotkeys,
				)
					.in_set(OnUpdate(ProgramState::Manual)),
			);
	}
}
