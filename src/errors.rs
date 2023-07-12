use bevy_egui::{egui::Ui, *};
use derive_more::Constructor;

use super::*;

#[derive(Resource, Constructor, Debug, Clone, PartialEq, Eq)]
pub struct Error {
	pub title: String,
	pub debug: String,
}

impl Error {
	fn render_to_ui(&self, ui: &mut Ui) {
		// red title
		ui.colored_label(egui::Color32::RED, self.title.clone());
		ui.label(&self.debug);
	}
}

pub fn sys_clear_error(mut commands: Commands, errors: ResMut<Error>) {
	commands.remove_resource::<Error>();
}

pub fn display_error(
	mut contexts: EguiContexts,
	errors: Option<ResMut<Error>>,
	mut commands: Commands,
) {
	egui::TopBottomPanel::bottom("errors_panel").show(contexts.ctx_mut(), |ui| {
		ui.heading("Errors panel");
		if ui.button("Clear").clicked() {
			commands.remove_resource::<Error>();
		}

		match errors {
			None => {
				ui.label("No errors or warnings");
			}
			Some(err) => {
				let err = err.into_inner();
				err.render_to_ui(ui);
			}
		}
	});
}
