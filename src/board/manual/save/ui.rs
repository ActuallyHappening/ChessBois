use bevy_egui::egui;
use bevy_egui::egui::Ui;

use crate::board::SharedState;

use super::UnstableSavedState;
use super::firebase;

impl SharedState {
	#[cfg(not(target_arch = "wasm32"))]
	fn old_save_ui(&mut self, ui: &mut Ui) {
		if self.moves.is_some() && ui.button("Save to clipboard (>= v0.3)").clicked() {
			let state = UnstableSavedState::try_from(self.clone()).unwrap();
			let json = state.into_json();
			ui.output_mut(|out| {
				out.copied_text = json;
			})
		}
		// ui.label("Not recommended, use newer save to DB feature which hopefully works on web as well.");
		ui.label("This copies a save which is NOT compatiable with older versions! I will hopefully implement saving to a database soon.");

		if ui.button("Load from clipboard (all versions)").clicked() {
			let json = crate::clipboard::get_from_clipboard();
			if let Ok(state) = UnstableSavedState::from_json(&json) {
				self.moves = Some(state.moves);
				self.board_options = state.board_options;
			};
		}
		ui.label("This can load older saves.");
	}

	#[cfg(not(target_arch = "wasm32"))]
	fn new_save_ui(&mut self, ui: &mut Ui) {
    use bevy::tasks::AsyncComputeTaskPool;

		if ui.button("Save to DB (>= v0.3").clicked() {
			let state = UnstableSavedState::try_from(self.clone()).unwrap();
			// TODO
			// let pool = AsyncComputeTaskPool::get();
			// pool.scope(|s| {
				// s.spawn(async {
					firebase::save_to_db(state);
				// })
			// });
		}
		ui.label("This saves the current state to the database under the specified title");
	}

	pub fn save_ui(&mut self, ui: &mut Ui) {
		#[cfg(not(target_arch = "wasm32"))]
		{
			egui::CollapsingHeader::new("Old Save/Load").default_open(true).show(ui, |ui| {
				self.old_save_ui(ui);
			});
			egui::CollapsingHeader::new("New Save to DB").default_open(true).show(ui, |ui| {
				self.new_save_ui(ui);
			});
		}
	}
}
