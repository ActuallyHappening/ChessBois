use bevy_egui::egui;
use bevy_egui::egui::Ui;

use crate::board::SharedState;

use super::MetaData;
use super::firebase;
use super::UnstableSavedState;

#[derive(Default, Clone)]
pub struct SaveState {
	pub title: String,
	pub author: String,
	pub description: String,

	pub error_str: Option<String>,
	pub loaded_metadatas: Vec<MetaData>,
}

impl TryFrom<SharedState> for super::MetaData {
	type Error = String;
	fn try_from(state: SharedState) -> Result<Self, Self::Error> {
		if state.save_state.title.is_empty() {
			return Err("No title specified".to_string());
		}
		if state.save_state.author.is_empty() {
			return Err("No author specified".to_string());
		}
		if state.save_state.description.is_empty() {
			return Err("No description provided".to_string());
		}
		Ok(super::MetaData {
			id: None,
			title: state.save_state.title,
			author: state.save_state.author,
			description: state.save_state.description,
			dimensions: state.board_options.dimensions(),
		})
	}
}

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
		use bevy_egui::egui::Color32;

		if ui.button("Save to DB (>= v0.3").clicked() {
			match UnstableSavedState::try_from(self.clone()) {
				Ok(state) => {
					firebase::save_to_db(state);
				}
				Err(err) => {
					self.save_state.error_str = Some(err);
				}
			}
		}
		ui.label("This saves the current state to the database under the specified title");

		// Add metadata
		ui.label("Title:");
		ui.text_edit_singleline(&mut self.save_state.title);

		ui.label("Author:");
		ui.text_edit_singleline(&mut self.save_state.author);

		if let Some(err) = &self.save_state.error_str {
			ui.colored_label(Color32::RED, err);
		}
	}

	#[cfg(not(target_arch = "wasm32"))]
	fn new_load_ui(&mut self, ui: &mut Ui) {
		if ui.button("Load list of saves").clicked() {
			match firebase::get_metadata_list() {
				Some(list) => {
					self.save_state.loaded_metadatas = list;
				}
				None => {
					self.save_state.error_str = Some("Failed to get list of saves".to_string());
				}
			}
		}

		for metadata in &self.save_state.loaded_metadatas {
			if ui.button(metadata.title.clone()).clicked() {
				match firebase::get_from_db(metadata.id.clone().unwrap()) {
					Some(state) => {
						self.moves = Some(state.moves);
						self.board_options = state.board_options;
					}
					None => {
						self.save_state.error_str = Some("Failed to load save".to_string());
					}
				}
			}
			ui.label(format!("By: {}", metadata.author));
			ui.label(format!("Dimensions widthxheight: {}x{}", metadata.dimensions.0, metadata.dimensions.1));
			ui.label(format!("Description: {}", metadata.description));
		}
	}


	pub fn save_ui(&mut self, ui: &mut Ui) {
		#[cfg(not(target_arch = "wasm32"))]
		{
			egui::CollapsingHeader::new("Old Save/Load")
				.default_open(false)
				.show(ui, |ui| {
					self.old_save_ui(ui);
				});

			egui::CollapsingHeader::new("[New] Save to DB")
				.default_open(true)
				.show(ui, |ui| {
					self.new_save_ui(ui);
				});

			egui::CollapsingHeader::new("[New] Load from DB").default_open(false).show(ui, |ui| {
				// TODO: impl first page load viewing
				self.new_load_ui(ui);
			});
		}
	}
}
