use super::{
	compute::ComputationResult,
	state_manual::{ManualFreedom, ManualMoves},
	viz_colours::VizColour,
	*,
};
use crate::{errors::Error, solver::algs::Computation, ProgramState};
use bevy_egui::{
	egui::{Color32, RichText},
	*,
};
use strum::IntoEnumIterator;

pub struct UiPlugin;
impl Plugin for UiPlugin {
	fn build(&self, app: &mut App) {
			// app.add_systems((left_sidebar_ui, right_sidebar_ui).before(display_error));
	}
}

#[allow(clippy::too_many_arguments)]
pub fn left_sidebar_ui(
	_commands: Commands,
	mut contexts: EguiContexts,

	mut cam: Query<&mut Transform, With<MainCamera>>,
	mut next_state: ResMut<NextState<ProgramState>>,
	state: Res<State<ProgramState>>,
	current_level: ResMut<ManualFreedom>,
	viz_colour: ResMut<viz_colours::VizColour>,

	moves: ResMut<ManualMoves>,

	mut commands: Commands,
	// markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,
	options: ResMut<CurrentOptions>,
	mut new_board_event: EventWriter<NewOptions>,
) {
	egui::SidePanel::left("general_controls_panel").show(contexts.ctx_mut(), |ui| {
		let options = options.clone().into_options();
		let current_alg = &options.selected_algorithm;
		let current_level = current_level.into_inner();
		let current_state = &state.into_inner().0;
		let current_colour = viz_colour.into_inner();
		let current_moves = moves.into_inner();

		ui.heading("Controls Panel");

		match current_state {
			ProgramState::Automatic => {
				if ui.button("Switch to manual mode").clicked() {
					next_state.set(ProgramState::Manual);
				}

				ui.label("Instructions: Hover over cell to begin knight there. Click cell to toggle availability (red = unavailable). You can alter the dimensions of the board (below) and the algorithm used.");

				const SIZE_CAP: f64 = 20.;
				ui.add(egui::Slider::from_get_set((2.)..=SIZE_CAP, |val| {
					let mut options = options.clone();
					if let Some(new_val) = val {
						options.options = options.options.update_width(new_val as u16);
						options.selected_start = None;
						new_board_event.send(NewOptions::from_options(options));
						new_val
					} else {
						options.options.width() as f64
					}
				}).text("Width"));
				ui.add(egui::Slider::from_get_set((2.)..=SIZE_CAP, |val| {
					let mut options = options.clone();
					if let Some(new_val) = val {
						options.options = options.options.update_height(new_val as u16);
						options.selected_start = None;
						new_board_event.send(NewOptions::from_options(options));
						new_val
					} else {
						options.options.height() as f64
					}
				}).text("Height"));

				ui.label("Select algorithm:");
				ui.horizontal_wrapped(|ui| {
					for alg in Algorithm::iter() {
						let str: &'static str = alg.into();
						let mut text = RichText::new(str);
						if &alg == current_alg {
							text = text.color(UI_ENABLED_COLOUR);
						}
						if ui.button(text).clicked() {
							new_board_event.send(NewOptions::from_options(Options {
								selected_algorithm: alg,
								selected_start: None,
								..options.clone()
							}));
						}
					}
				});
				ui.label(current_alg.get_description());

				ui.add(egui::Slider::from_get_set((10.)..=10_000_000., |val| {
					if let Some(new_val) = val {
						*ALG_STATES_CAP.lock().unwrap() = new_val as u128;
						new_val
					} else {
						*ALG_STATES_CAP.lock().unwrap() as f64
					}
				}).text("Safety States Cap"));
				ui.label("If your computer is good, you can safely make this number higher. This cap is put in to stop your computer infinitely computing. I can allow it higher if you want");

				ui.add(egui::Slider::from_get_set(CAMERA_HEIGHT as f64..=(CAMERA_HEIGHT as f64 * 2.), |val| {
					if let Some(new_val) = val {
						cam.single_mut().translation.y = new_val as f32;
						new_val
					} else {
						cam.single().translation.y as f64
					}
				}).text("Camera zoom"));
				ui.label("You can change the camera zoom to see larger boards");

				// if ui.button("Hide visual icons").clicked() {
				// 	despawn_markers(&mut commands, markers);
				// }
			},
			ProgramState::Manual => {
				if ui.button("Switch back to automatic mode").clicked() {
					next_state.set(ProgramState::Automatic);
				}

				ui.label("Manual mode allows you to plot that path that you want. It has varying levels of freedom, from completely free which allows you to jump from any available square to any other available square\
				to only allowing you to move to squares that are one knight move away from the current square. If you want to disable/re-enable a square, switch back to automatic then back to manual.\
				To reset your drawing, change modes then change back.
				");

				ui.label("Select a freedom level:");
				ui.horizontal_wrapped(|ui| {
					for level in ManualFreedom::iter() {
						let name = format!("{}", level);
						let mut text = RichText::new(name);
						if &level == current_level {
							text = text.color(UI_ENABLED_COLOUR);
						}
						if ui.button(text).clicked() {
							*current_level = level;
						}
					}
				});
				ui.label(current_level.get_description());

				ui.label("Pick a manual visualization colour");
				ui.horizontal_wrapped(|ui| {
					for col in VizColour::iter() {
						let str = format!("{}", col);
						let mut text = RichText::new(str);
						if &col == current_colour {
							text = text.color(UI_ENABLED_COLOUR);
						}
						if ui.button(text).clicked() {
							*current_colour = col;
						}
					}
				});

					// copy + paste functionality
				let mut state_str = current_moves.to_json();
				if ui.text_edit_singleline(&mut state_str).changed() {
					match ManualMoves::try_from(state_str) {
						Ok(moves) => {
							*current_moves = moves;
						}
						Err(e) => {
							warn!("Could not parse state JSON string: {}", e);
							commands.insert_resource(Error::new("Could not parse your data".into(), format!("Parsing str failed with err: {} ; {:?}", e, e)));
						}
					}
				}

				// undo button
				if ui.button("Undo").clicked() {
					current_moves.undo_move();
				}
			},
		}
	});
}

pub fn right_sidebar_ui(
	options: Res<CurrentOptions>,
	computation: Option<Res<ComputationResult>>,
	mut contexts: EguiContexts,
) {
	let options = &options.current;
	let solution = computation.map(|comp| comp.into_comp());

	egui::SidePanel::right("right_sidebar").show(contexts.ctx_mut(), |ui| {
		ui.heading("Results Panel");

		if let Some(solution) = solution {
			let alg: Algorithm = options.selected_algorithm;
			match solution {
				Computation::Successful {
					explored_states: states,
					..
				} => {
					let mut msg = format!("Solution found in {} states considered", states);
					if !alg.should_show_states() {
						msg = "Solution found".to_string();
					}
					ui.label(RichText::new(msg).color(Color32::GREEN));
					ui.label("Notes: if states =0 it is because the solution was cached");
				}
				Computation::Failed {
					total_states: states,
				} => {
					let mut msg = format!("No solution found, with {} states considered", states);
					if !alg.should_show_states() {
						msg = "Solution found".to_string();
					}
					ui.label(RichText::new(msg).color(Color32::RED));
				}
				Computation::GivenUp { explored_states: states } => {
					let mut msg = format!("To avoid excessive computation finding a solution was given up, with {} states considered", states);
					if !alg.should_show_states() {
						msg = "Solution found".to_string();
					}
					ui.label(RichText::new(msg).color(Color32::RED));
				}
			}
		}

		if let Some(start) = &options.selected_start {
			let alg_selected: &str = options.selected_algorithm.into();
			ui.label(format!(
				"Current info: Starting at {start} with {} algorithm {}",
				alg_selected,
				options.options.get_description()
			));
		}
	
	});
}
