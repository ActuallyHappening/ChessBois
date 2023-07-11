use bevy_egui::*;
use derive_more::Constructor;

use super::*;

#[derive(Resource, Constructor, Debug, Clone, PartialEq, Eq)]
pub struct Error {
	pub title: String,
	pub debug: String,
}

pub fn display_error(
	mut contexts: EguiContexts,

	errors: Option<Res<Error>>,
) {
	egui::TopBottomPanel::bottom("errors_panel").show(contexts.ctx_mut(), |ui| {
		ui.heading("Errors panel");
	});
}