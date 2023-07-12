use super::*;

pub struct AutomaticState;
impl Plugin for AutomaticState {
	fn build(&self, app: &mut App) {
		app
			.add_systems(
				(
					handle_automatic_computation,
					update_cache_from_computation,
					handle_spawning_visualization,
					handle_new_options,
					right_sidebar_ui,
				)
					.in_set(OnUpdate(ProgramState::Automatic)),
			)
			// state changes
			.add_systems(
				(
					state_manual::despawn_visualization,
					state_manual::despawn_markers,
					state_manual::add_empty_manual_moves,
				)
					.in_schedule(OnExit(ProgramState::Automatic)),
			)
			.add_systems(
				(
					state_manual::despawn_visualization,
					state_manual::despawn_markers,
					state_manual::add_empty_manual_moves,
					state_manual::add_default_manual_viz_colour,
				)
					.in_schedule(OnEnter(ProgramState::Automatic)),
			);
	}
}
