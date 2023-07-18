pub use bevy::ecs::schedule::LogLevel;
use bevy_egui::{egui::Ui, *};
use derive_more::Constructor;

use super::*;

#[derive(Resource, Constructor, Debug, Clone, PartialEq, Eq)]
pub struct Error {
	pub title: String,
}

impl Error {
	pub fn render_to_ui(&self, ui: &mut Ui) {
		// red title
		ui.colored_label(egui::Color32::RED, self.title.clone());
	}

	pub fn handle_error(intensity: LogLevel, err: &str, commands: &mut Commands) {
		commands.insert_resource(Error::new(err.to_string()));
		match intensity {
			LogLevel::Error => {
				error!("{}", err);
			}
			LogLevel::Warn => {
				warn!("{}", err);
			}
			LogLevel::Ignore => {
				trace!("(Error) {}", err);
			}
		}
	}
}

pub fn display_error(
	mut contexts: EguiContexts,
	errors: Option<ResMut<Error>>,
	mut commands: Commands,
) {
	egui::Window::new("errors_panel").show(contexts.ctx_mut(), |ui| {
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
