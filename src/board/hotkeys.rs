use super::*;

use strum::IntoEnumIterator;

pub struct HotkeysPlugin;
impl Plugin for HotkeysPlugin {
	fn build(&self, app: &mut App) {
		app.add_system(hotkeys);
	}
}

pub trait Hotkeyable: Into<KeyCode> + Clone + IntoEnumIterator {
	fn activate_hotkeys(&mut self, keys: &Res<Input<KeyCode>>) {
		for key in Self::iter() {
			if keys.just_pressed(key.clone().into()) {
				*self = key
			}
		}
	}
}

pub fn hotkeys(state: ResMut<SharedState>, keys: Res<Input<KeyCode>>) {
	let state = state.into_inner();
	state.on_click.activate_hotkeys(&keys);
	state.alg.activate_hotkeys(&keys);

	if keys.just_pressed(KeyCode::Escape) {
		state.start = None;
	}
	if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Delete) {
		state.start = None;
		// state.moves = None;
	}

	if keys.just_pressed(KeyCode::H) {
		state.visual_opts.show_visualisation = !state.visual_opts.show_visualisation;
	}
}
