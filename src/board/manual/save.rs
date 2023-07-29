use bevy_egui::egui::Ui;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use crate::{
	board::{coloured_moves::ColouredMoves, SharedState},
	solver::{pieces::StandardPieces, BoardOptions},
};

#[derive(Serialize, Deserialize, Constructor)]
pub struct SavedState {
	pub moves: ColouredMoves,
	pub piece: StandardPieces,
	pub board_options: BoardOptions,
}

impl TryFrom<SharedState> for SavedState {
	type Error = ();
	fn try_from(value: SharedState) -> Result<Self, Self::Error> {
		Ok(Self {
			moves: value.moves.ok_or(())?,
			piece: value.piece,
			board_options: value.board_options,
		})
	}
}

impl SavedState {
	pub fn to_json(&self) -> String {
		serde_json::to_string(self).unwrap()
	}

	pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
		serde_json::from_str(json)
	}
}

impl SharedState {
	pub fn saved_state_ui(&mut self, ui: &mut Ui) {
		#[cfg(not(target_arch = "wasm32"))]
		if self.moves.is_some() && ui.button("Save").clicked() {
			let state = SavedState::try_from(self.clone()).unwrap();
			let json = state.to_json();
			ui.output_mut(|out| {
				out.copied_text = json;
			})
		}
		#[cfg(not(target_arch = "wasm32"))]
		if ui.button("Load").clicked() {
			let json = crate::clipboard::get_from_clipboard();
			let state = SavedState::from_json(&json).unwrap();
			self.moves = Some(state.moves);
			self.piece = state.piece;
			self.board_options = state.board_options;
		}
	}
}
