use bevy_egui::egui::Ui;

use crate::board::SharedState;

use super::UnstableSavedState;


impl SharedState {
	#[cfg(not(target_arch = "wasm32"))]
	fn old_save_ui(&mut self, ui: &mut Ui) {
		if self.moves.is_some() && ui.button("Save to clipboard (>= v0.3)").clicked() {
			let state = UnstableSavedState::try_from(self.clone()).unwrap();
			let json = state.to_json();
			ui.output_mut(|out| {
				out.copied_text = json;
			})
		}
		ui.label("Not recommended, use newer save to DB feature which hopefully works on web as well.");

		if ui.button("Load from clipboard (all versions)").clicked() {
			let json = crate::clipboard::get_from_clipboard();
			if let Ok(state) = UnstableSavedState::from_json(&json) {
				self.moves = Some(state.moves);
				self.board_options = state.board_options;
			};
		}
		ui.label("This can load older saves.");
	}

	fn new_save_ui(&mut self, ui: &mut Ui) {
		if ui.button("Save to DB (>= v0.3").clicked() {
			let state = UnstableSavedState::try_from(self.clone()).unwrap();
			// TODO
		}
		ui.label("This saves the current state to the database under the specified title");

		
	}

	pub fn save_ui(&mut self, ui: &mut Ui) {
		
		self.old_save_ui(ui);
		self.new_save_ui(ui);
	}
}