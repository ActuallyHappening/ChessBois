use bevy::reflect::Reflect;
use bevy_egui::egui;
use bevy_egui::egui::TextEdit;
use bevy_egui::egui::Ui;
use bevy_egui::egui::Widget;

use crate::board::SharedState;

use super::firebase;
use super::MetaData;
use super::UnstableSavedState;

#[derive(Default, Clone, Reflect)]
pub struct SaveState {
	pub title: String,
	pub author: String,
	pub description: String,

	pub error_str: Option<String>,
	pub loaded_metadatas: Vec<MetaData>,
	pub is_typing: bool,
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

#[cfg(not(target_arch = "wasm32"))]
impl SharedState {
	fn old_save_ui(&mut self, ui: &mut Ui) {
		if self.moves.is_some() && ui.button("Save to clipboard (>= v0.3)").clicked() {
			let state = self.clone().dangerous_into();
			let json = state.into_json();
			ui.output_mut(|out| {
				out.copied_text = json;
			})
		}
		ui.label("This copies a save which is NOT compatiable with older versions! I recommend saving useing the newer database feature.");

		if ui.button("Load from clipboard (all versions)").clicked() {
			let json = crate::clipboard::get_from_clipboard();
			if let Ok(state) = UnstableSavedState::from_json(json.trim()) {
				self.moves = Some(state.moves);
				self.board_options = state.board_options;
			};
		}
		ui.label("This can load older saves.");
	}

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
		// also handle focussing shananigans
		self.save_state.is_typing = false;
		ui.label("Title:");
		if TextEdit::singleline(&mut self.save_state.title).hint_text("My awesome creation").ui(ui).has_focus() {
			self.save_state.is_typing = true;
		}

		ui.label("Author:");
		if TextEdit::singleline(&mut self.save_state.author).hint_text("John Smith").ui(ui).has_focus() {
			self.save_state.is_typing = true;
		}

		ui.label("Description:");
		if TextEdit::multiline(&mut self.save_state.description).hint_text("A chess board showing a cool property of knights moves.\nIt is also awesome.").ui(ui).has_focus() {
			self.save_state.is_typing = true;
		}

		if let Some(err) = &self.save_state.error_str {
			ui.colored_label(Color32::RED, err);
		}
	}

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
			ui.label(format!(
				"Dimensions: {}x{}",
				metadata.dimensions.0, metadata.dimensions.1
			));
			ui.label(format!("Description: {}", metadata.description));
			ui.hyperlink_to(
				"Click to open in browser",
				crate::weburl::create_url_with_id(metadata.id.clone().unwrap().inner().into()),
			);
		}
	}
}
impl SharedState {
	pub fn save_ui(&mut self, ui: &mut Ui) {
		#[cfg(not(target_arch = "wasm32"))]
		{
			egui::CollapsingHeader::new("Old Save/Load")
				.default_open(false)
				.show(ui, |ui| {
					self.old_save_ui(ui);
				});

			egui::CollapsingHeader::new("[New] Save to DB")
				.default_open(false)
				.show(ui, |ui| {
					self.new_save_ui(ui);
				});

			egui::CollapsingHeader::new("[New] Load from DB")
				.default_open(false)
				.show(ui, |ui| {
					// TODO: impl first page load viewing
					self.new_load_ui(ui);
				});
		}
		#[cfg(target_arch = "wasm32")]
		{
			ui.label("Saving / Loading is yet to be supported on web (WASM), but work is in progress!");
		}
	}
}
