use super::{
	automatic::ToggleAction,
	cells::{visualization::VisualOpts, MarkerMarker},
	*,
};
use crate::{
	errors::Error, solver::algs::Computation, MainCamera, ProgramState, UI_ENABLED_COLOUR,
};
use bevy_egui::{
	egui::{Color32, Pos2, RichText, Ui},
	*,
};

pub struct UiPlugin;
impl Plugin for UiPlugin {
	fn build(&self, app: &mut App) {
		app.add_system(left_ui);
	}
}

pub fn left_ui(mut contexts: EguiContexts, state: ResMut<SharedState>) {
	egui::SidePanel::left("Left sidebar").show(contexts.ctx_mut(), |ui| {
		ui.heading("Controls Panel");

		let state = state.into_inner();

		egui::CollapsingHeader::new("Select Algorithm")
			.default_open(true)
			.show(ui, |ui| {
				state.alg.ui(ui);

				ui.label(state.alg.get_description());
			});

		ui.collapsing("Visualisation options", |ui| {
			state.visual_opts.ui(ui);
		});
	});
}

// impl VisualOpts {
// 	pub fn sys_viz_options_ui(mut contexts: EguiContexts, state: ResMut<SharedState>) {
// 		egui::Window::new("Visualization options")
// 			// .default_pos(Pos2::new(4200., 4200.))
// 			.current_pos(Pos2::new(4200., 4200.))
// 			.default_open(false)
// 			.show(contexts.ctx_mut(), |ui| {
// 				ui.heading("Visualization options:");

// 				state.into_inner().visual_opts.ui(ui);
// 			});
// 	}
// }

// impl ManualMoves {
// 	pub fn save_state_ui(
// 		mut contexts: EguiContexts,
// 		state: ResMut<ManualMoves>,
// 		options: Res<CurrentOptions>,
// 		mut commands: Commands,
// 	) {
// 		let current_moves = state.into_inner();

// 		egui::Window::new("Save / Loading")
// 			.default_pos(Pos2::new(4200., 4200.))
// 			.default_open(false)
// 			.show(contexts.ctx_mut(), |ui| {
// 				egui::ScrollArea::vertical().show(ui, |ui| {
// 				ui.heading("Manually saving:");
// 				ui.label("You can save your created chess creation into some JSON, and load it in at any time. \
// 				This works only on desktop versions, web is not supported. I can add this, but it will take more effort");

// 				// copy + paste functionality

// 				// viewer
// 				let state_str = current_moves.to_json();
// 				let mut str = state_str.clone();
// 				ui.collapsing("The actual data", |ui| {
// 					if ui.code_editor(&mut str).changed() {
// 						match ManualMoves::try_from(str.clone()) {
// 							Ok(moves) => {
// 								*current_moves = moves;
// 							}
// 							Err(e) => {
// 								warn!("Could not parse state JSON string: {}", e);
// 								commands.insert_resource(Error::new("Could not parse your data".into()));
// 							}
// 						}
// 					}
// 				});

// 				// copy to clipboard
// 				// non wasm
// 				#[cfg(not(target_arch = "wasm32"))]
// 				if ui.button("Copy current state to clipboard").clicked() {
// 					ui.output_mut(|o| o.copied_text = state_str);

// 					// #[cfg(target_arch = "wasm32")]
// 					// crate::clipboard::set_to_clipboard(&state_str);
// 				}

// 				// copy to clipboard web
// 				// #[cfg(target_arch = "wasm32")]
// 				// if ui.button("Copy current state to clipboard").clicked() {
// 				// 	crate::clipboard::set_to_clipboard(&state_str);
// 				// }

// 				// paste from clipboard
// 				#[cfg(not(target_arch = "wasm32"))]
// 				if ui.button("Paste from your current clipboard").clicked() {
// 					let clip = crate::clipboard::get_from_clipboard();
// 					// if let Some(clip) = clipboard {
// 					match ManualMoves::try_from(clip) {
// 						Ok(moves) => {
// 							*current_moves = moves;
// 						}
// 						Err(e) => {
// 							warn!("Could not parse clipboard JSON string: {}", e);
// 							commands.insert_resource(Error::new("Could not parse your data".into()));
// 						}
// 					}
// 					// }
// 				}

// 				// URL saving
// 				#[cfg(feature = "weburl")]
// 				{
// 					ui.heading("URL saving:");
// 					ui.label("An alternative to saving your state as a dump of JSON is to generate an open link. \
// 					This link will contain all the information needed to recreate your board.");

// 					egui::ScrollArea::vertical().max_height(50.).show(ui, |ui| {
// 						ui.hyperlink(crate::weburl::export_state_to_url(crate::weburl::State {
// 							options: options.current.options.clone(),
// 							manual_moves: current_moves.clone(),
// 						}));
// 					});
// 				}
// 			});
// 		});
// 	}
// }
