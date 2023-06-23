use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use msrc_q11::{Board, BoardOptions, ChessPoint};
use std::f32::consts::TAU;

use crate::{
	CELL_DEPTH, CELL_HEIGHT, CELL_SELECTED_COLOUR, CELL_SIZE, VISUALIZATION_COLOUR,
	VISUALIZATION_HEIGHT,
};

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_event::<NewCellSelected>()
			.add_event::<NewBoardCellOptions>()
			.add_startup_system(spawn_initial)
			.add_system(spawn_left_sidebar_ui)
			.add_plugins(
				DefaultPickingPlugins
					.build()
					.disable::<DefaultHighlightingPlugin>()
					.disable::<DebugPickingPlugin>(),
			)
			.add_system(handle_new_cell_selected_event)
			.add_system(handle_new_board_event);
	}
}

/// Represents information required to display cells + visual solutions
#[derive(Debug, Clone)]
pub struct Options {
	options: BoardOptions,
	selected_start: Option<ChessPoint>,
}

#[derive(Debug, Clone)]
pub struct NewCellSelected {
	new: ChessPoint,
}

/// Event representing when the board has changed size/shape/<options>, NOT start location!
#[derive(Debug, Clone)]
pub struct NewBoardCellOptions {
	new: BoardOptions,
}

#[derive(Resource, Debug, Clone)]
pub struct CurrentOptions {
	current: Options,
}

use coords::*;
mod coords {
	use super::*;

	/// Returns spacial coordinates of center of cell mesh
	fn get_spacial_coord_normalized(board: &Board, chess_position: ChessPoint) -> Vec2 {
		let ChessPoint { row: y, column: x } = chess_position;
		let width = board.options().width() as f32;
		let height = board.options().height() as f32;
		let x = x as f32;
		let y = y as f32;

		// normalized: (column, row) = (x, y)
		// Adjusted = ((x - 1) -X Delta, (y - 1) - Y Delta)
		// X Delta = (width - 1) / 2

		let x_adjusted = (x - 1.) - (width - 1.) / 2.;
		let y_adjusted = (y - 1.) - (height - 1.) / 2.;

		Vec2::new(x_adjusted, y_adjusted)
	}

	pub fn get_spacial_coord(board: &Board, chess_position: ChessPoint) -> Vec3 {
		let normalized = get_spacial_coord_normalized(board, chess_position) * CELL_SIZE;

		Vec3::new(normalized.x, CELL_HEIGHT, -normalized.y)
	}

	pub fn get_spacial_coord_2d(board: &Board, chess_position: ChessPoint) -> Vec2 {
		let normalized = get_spacial_coord_normalized(board, chess_position) * CELL_SIZE;

		Vec2::new(normalized.x, -normalized.y)
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_coords_center() {
			let coords = get_spacial_coord_normalized(&Board::new(3, 3), ChessPoint::new(2, 2));

			assert_eq!(coords, Vec2::new(0., 0.));
		}

		#[test]
		fn test_coords_bl_2() {
			let coords = get_spacial_coord_normalized(&Board::new(2, 2), ChessPoint::new(1, 1));

			assert_eq!(coords, Vec2::new(-0.5, -0.5));
		}

		#[test]
		fn test_coords_bl_5() {
			let coords = get_spacial_coord_normalized(&Board::new(5, 5), ChessPoint::new(1, 1));

			assert_eq!(coords, Vec2::new(-2., -2.));
		}
	}
}

fn spawn_initial(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let options = Options {
		options: BoardOptions::new(8, 8),
		selected_start: Some(ChessPoint::new(9, 9)),
	};
	let current_options = CurrentOptions {
		current: options.clone(),
	};

	commands.insert_resource(current_options);

	spawn_cells(&mut commands, &options, &mut meshes, &mut materials);
	// spawn_visualization_from_options(&options, &mut commands, &mut meshes, &mut materials);

	// spawn_left_sidebar_ui(&mut commands);
}

use cells::*;
mod cells {
	use msrc_q11::CellOption;

	use super::*;
	use crate::CELL_DISABLED_COLOUR;

	pub fn spawn_cells(
		commands: &mut Commands,
		options: &Options,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
	) {
		let board = Board::from_options(options.options.clone());
		let start = options.selected_start;
		let options = options.options.clone();

		for point in options.get_all_points() {
			let colour = compute_colour(&point, Some(&options), start);
			spawn_cell(point, &board, colour, commands, meshes, materials);
		}
	}

	fn despawn_cells(commands: &mut Commands, cells: Query<Entity, With<ChessPoint>>) {
		for cell in cells.iter() {
			commands.entity(cell).despawn_recursive();
		}
	}

	/// Takes as much information as it can get and returns the colour the cell should be.
	/// 
	/// - Pass None to options to skip checking if cell is disabled
	/// - Pass None to start to skip checking if cell is selected
	fn compute_colour(
		point: &ChessPoint,
		options: Option<&BoardOptions>,
		start: Option<ChessPoint>,
	) -> Color {
		if options.is_some_and(|options| options.get_unavailable_points().contains(point)) {
			// info!("Point {} is unavailable", point);
			CELL_DISABLED_COLOUR
		} else if Some(*point) == start {
			CELL_SELECTED_COLOUR
		} else {
			point.get_standard_colour()
		}
	}

	fn spawn_cell(
		at: ChessPoint,
		board: &Board,
		colour: Color,
		commands: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
	) {
		let transform = Transform::from_translation(get_spacial_coord(board, at))
			.with_rotation(Quat::from_rotation_x(-TAU / 4.));
		let mesh = meshes.add(shape::Box::new(CELL_SIZE, CELL_SIZE, CELL_DEPTH).into());

		commands.spawn((
			PbrBundle {
				mesh,
				transform,
				material: materials.add(StandardMaterial::from(colour)),
				..default()
			},
			Name::new(format!("Chess Square ({}, {})", at.row, at.column)),
			at,
			PickableBundle::default(),    // Makes the entity pickable
			RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
			// OnPointer::<Move>::run_callback(),
			OnPointer::<Over>::run_callback(cell_selected),
			OnPointer::<Out>::run_callback(cell_deselected),
			OnPointer::<Click>::run_callback(toggle_cell_availability),
		));
	}

	/// Changes selected cell
	fn cell_selected(
		// The first parameter is always the `ListenedEvent`, passed in by the event listening system.
		In(event): In<ListenedEvent<Over>>,

		mut materials: ResMut<Assets<StandardMaterial>>,

		cells: Query<(&Handle<StandardMaterial>, &ChessPoint)>,
		current_options: ResMut<CurrentOptions>,

		mut new_cell_selected: EventWriter<NewCellSelected>,
	) -> Bubble {
		let (mat, point) = cells.get(event.target).unwrap();

		let options = &current_options.current.options;
		let is_disabled = options.get_unavailable_points().contains(point);

		if !is_disabled {
			// sets colour to selected
			let material = materials.get_mut(mat).unwrap();
			material.base_color = CELL_SELECTED_COLOUR;

			// send event
			new_cell_selected.send(NewCellSelected { new: *point });
		}

		Bubble::Up
	}

	/// Just undoes colour change to normal
	fn cell_deselected(
		In(event): In<ListenedEvent<Out>>,
		mut materials: ResMut<Assets<StandardMaterial>>,
		square: Query<(&Handle<StandardMaterial>, &ChessPoint)>,
		options: Res<CurrentOptions>,
	) -> Bubble {
		let (mat, point) = square.get(event.target).unwrap();

		// sets colour to selected
		let material = materials.get_mut(mat).unwrap();
		material.base_color = compute_colour(
			point,
			Some(&options.current.options),
			None,
		);

		Bubble::Up
	}

	fn toggle_cell_availability(
		In(event): In<ListenedEvent<Click>>,
		// mut materials: ResMut<Assets<StandardMaterial>>,
		cells: Query<(&Handle<StandardMaterial>, &ChessPoint)>,
		current_options: ResMut<CurrentOptions>,

		mut new_board: EventWriter<NewBoardCellOptions>,
	) -> Bubble {
		let (mat, point) = cells.get(event.target).unwrap();

		let options = &current_options.current.options;
		match options.get(point) {
			Some(CellOption::Available) => {
				// let material = materials.get_mut(mat).unwrap();
				// material.base_color = CELL_DISABLED_COLOUR;

				new_board.send(NewBoardCellOptions {
					new: options.clone().set(point, CellOption::Unavailable),
				})
			}
			Some(CellOption::Unavailable) => {
				// let material = materials.get_mut(mat).unwrap();
				// material.base_color = point.get_standard_colour();

				new_board.send(NewBoardCellOptions {
					new: options.clone().set(point, CellOption::Available),
				})
			}
			None => (),
		}
		Bubble::Up
	}

	/// Handles re-constructing visual solution
	pub fn handle_new_cell_selected_event(
		mut new_starting_point: EventReader<NewCellSelected>,
		current_options: ResMut<CurrentOptions>,

		vis: Query<Entity, With<VisualizationComponent>>,

		mut commands: Commands,
		mut meshes: ResMut<Assets<Mesh>>,
		mut materials: ResMut<Assets<StandardMaterial>>,
	) {
		let current_options = &current_options.current;
		if let Some(new_starting_point) = new_starting_point.into_iter().next() {
			let new_options = Options {
				options: current_options.options.clone(),
				selected_start: Some(new_starting_point.new),
			};
			commands.insert_resource(CurrentOptions {
				current: new_options.clone(),
			});

			// info!("New starting point: {}", new_starting_point.new);
			despawn_visualization(&mut commands, vis);
			spawn_visualization_from_options(&new_options, &mut commands, &mut meshes, &mut materials);
		}
	}

	pub fn handle_new_board_event(
		mut new_board: EventReader<NewBoardCellOptions>,

		vis: Query<Entity, With<VisualizationComponent>>,
		cells: Query<Entity, With<ChessPoint>>,

		mut commands: Commands,
		mut meshes: ResMut<Assets<Mesh>>,
		mut materials: ResMut<Assets<StandardMaterial>>,
	) {
		if let Some(new_options) = new_board.into_iter().next() {
			let new_options = Options {
				options: new_options.new.clone(),
				selected_start: None,
			};
			commands.insert_resource(CurrentOptions {
				current: new_options.clone(),
			});

			despawn_visualization(&mut commands, vis);
			despawn_cells(&mut commands, cells);

			spawn_cells(&mut commands, &new_options, &mut meshes, &mut materials);
		}
	}
}

use visualization::*;
mod visualization {
	use super::*;
	use msrc_q11::{piece_tour_no_repeat, Move, StandardKnight};

	#[allow(dead_code)]
	#[derive(Component, Debug, Clone)]
	pub struct VisualizationComponent {
		from: ChessPoint,
		to: ChessPoint,
	}

	pub fn spawn_visualization_from_options(
		options: &Options,

		commands: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
	) {
		if let Some(start) = options.selected_start {
			if options.options.get_unavailable_points().contains(&start) {
				// debug!("Start point is disabled!");
				return;
			}

			let mut board = Board::from_options(options.options.clone());
			let piece = StandardKnight {};

			match piece_tour_no_repeat(&piece, &mut board, start) {
				Some(moves) => {
					for Move { from, to } in moves.iter() {
						spawn_path_line(commands, meshes, materials, from, to, &board)
					}
				}
				None => {
					info!("No solution found!");
				}
			}
		}

		// spawn_path_line(
		// 	commands,
		// 	meshes,
		// 	materials,
		// 	&start,
		// 	&ChessPoint::new(4, 4),
		// 	&board,
		// )
	}

	pub fn despawn_visualization(
		commands: &mut Commands,
		visualization: Query<Entity, With<VisualizationComponent>>,
	) {
		for entity in visualization.iter() {
			commands.entity(entity).despawn_recursive();
		}
	}

	fn spawn_path_line(
		commands: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
		from: &ChessPoint,
		to: &ChessPoint,
		board: &Board,
	) {
		let start_pos = get_spacial_coord_2d(board, *from);
		let end_pos = get_spacial_coord_2d(board, *to);

		let center = (start_pos + end_pos) / 2.; // ✅
		let length = (start_pos - end_pos).length(); // ✅
		let angle: f32 = -(start_pos.y - end_pos.y).atan2(start_pos.x - end_pos.x);

		// assert_eq!(angle, TAU / 8., "Drawing from {from} [{from:?}] [{from_pos}] to {to} [{to:?}] [{to_pos}], Angle: {angle}, 𝚫y: {}, 𝚫x: {}", (to_pos.y - from_pos.y), (to_pos.x - from_pos.x));
		// info!("Angle: {angle}, {}", angle.to_degrees());

		let transform =
			Transform::from_translation(Vec3::new(center.x, VISUALIZATION_HEIGHT, center.y))
				.with_rotation(Quat::from_rotation_y(angle));

		// info!("Transform: {:?}", transform);
		// info!("Angle: {:?}, Length: {:?}", angle, length);

		let mesh_thin_rectangle = meshes.add(shape::Box::new(length, 1., 1.).into());

		commands.spawn((
			PbrBundle {
				mesh: mesh_thin_rectangle,
				material: materials.add(VISUALIZATION_COLOUR.into()),
				transform,
				..default()
			},
			VisualizationComponent {
				from: *from,
				to: *to,
			},
		));
	}
}

use ui::*;
mod ui {
	use super::*;
	use bevy_egui::*;

	pub fn spawn_left_sidebar_ui(
		mut contexts: EguiContexts,

		current_options: ResMut<CurrentOptions>,

		mut new_board_event: EventWriter<NewBoardCellOptions>,
	) {
		egui::SidePanel::left("general_controls_panel").show(contexts.ctx_mut(), |ui| {
			let old_options = current_options.current.options.clone();

			ui.heading("Controls");

			// ui.add(egui::Slider::new(&mut my_f32, 3.0..=10.).text("My value"));

			// ui.add(egui::Slider::new(&mut ui_state.value, 0.0..=10.0).text("value"));
			if ui.button("Wider +1").clicked() {
				let new_options = old_options.clone().update_width(old_options.width() + 1);
				new_board_event.send(NewBoardCellOptions { new: new_options });
			}

			if ui.button("Thinner -1").clicked() {
				let new_options = old_options.clone().update_width(old_options.width() - 1);
				new_board_event.send(NewBoardCellOptions { new: new_options });
			}

			if ui.button("Taller +1").clicked() {
				let new_options = old_options.clone().update_height(old_options.height() + 1);
				new_board_event.send(NewBoardCellOptions { new: new_options });
			}

			if ui.button("Shorter -1").clicked() {
				let new_options = old_options.clone().update_height(old_options.height() - 1);
				new_board_event.send(NewBoardCellOptions { new: new_options });
			}

			ui.label(format!(
				"Current Options: \n{}",
				current_options.current.options
			));
		});
	}

	// fn ui_example_system(
	// 	mut ui_state: ResMut<UiState>,
	// 	// You are not required to store Egui texture ids in systems. We store this one here just to
	// 	// demonstrate that rendering by using a texture id of a removed image is handled without
	// 	// making bevy_egui panic.
	// 	mut rendered_texture_id: Local<egui::TextureId>,
	// 	mut is_initialized: Local<bool>,
	// 	// If you need to access the ids from multiple systems, you can also initialize the `Images`
	// 	// resource while building the app and use `Res<Images>` instead.
	// 	images: Local<Images>,
	// 	mut contexts: EguiContexts,
	// ) {
	// 	let egui_texture_handle = ui_state
	// 		.egui_texture_handle
	// 		.get_or_insert_with(|| {
	// 			contexts.ctx_mut().load_texture(
	// 				"example-image",
	// 				egui::ColorImage::example(),
	// 				Default::default(),
	// 			)
	// 		})
	// 		.clone();

	// 	let mut load = false;
	// 	let mut remove = false;
	// 	let mut invert = false;

	// 	if !*is_initialized {
	// 		*is_initialized = true;
	// 		*rendered_texture_id = contexts.add_image(images.bevy_icon.clone_weak());
	// 	}

	// 	let ctx = contexts.ctx_mut();

	// 	egui::SidePanel::left("side_panel")
	// 		.default_width(200.0)
	// 		.show(ctx, |ui| {
	// 			ui.heading("Side Panel");

	// 			ui.horizontal(|ui| {
	// 				ui.label("Write something: ");
	// 				ui.text_edit_singleline(&mut ui_state.label);
	// 			});

	// 			ui.add(egui::widgets::Image::new(
	// 				egui_texture_handle.id(),
	// 				egui_texture_handle.size_vec2(),
	// 			));

	// 			ui.add(egui::Slider::new(&mut ui_state.value, 0.0..=10.0).text("value"));
	// 			if ui.button("Increment").clicked() {
	// 				ui_state.value += 1.0;
	// 			}

	// 			ui.allocate_space(egui::Vec2::new(1.0, 100.0));
	// 			ui.horizontal(|ui| {
	// 				load = ui.button("Load").clicked();
	// 				invert = ui.button("Invert").clicked();
	// 				remove = ui.button("Remove").clicked();
	// 			});

	// 			ui.add(egui::widgets::Image::new(
	// 				*rendered_texture_id,
	// 				[256.0, 256.0],
	// 			));

	// 			ui.allocate_space(egui::Vec2::new(1.0, 10.0));
	// 			ui.checkbox(&mut ui_state.is_window_open, "Window Is Open");

	// 			ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
	// 				ui.add(egui::Hyperlink::from_label_and_url(
	// 					"powered by egui",
	// 					"https://github.com/emilk/egui/",
	// 				));
	// 			});
	// 		});

	// 	egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
	// 		// The top panel is often a good place for a menu bar:
	// 		egui::menu::bar(ui, |ui| {
	// 			egui::menu::menu_button(ui, "File", |ui| {
	// 				if ui.button("Quit").clicked() {
	// 					std::process::exit(0);
	// 				}
	// 			});
	// 		});
	// 	});

	// 	egui::CentralPanel::default().show(ctx, |ui| {
	// 		ui.heading("Egui Template");
	// 		ui.hyperlink("https://github.com/emilk/egui_template");
	// 		ui.add(egui::github_link_file_line!(
	// 			"https://github.com/mvlabat/bevy_egui/blob/main/",
	// 			"Direct link to source code."
	// 		));
	// 		egui::warn_if_debug_build(ui);

	// 		ui.separator();

	// 		ui.heading("Central Panel");
	// 		ui.label("The central panel the region left after adding TopPanel's and SidePanel's");
	// 		ui.label("It is often a great place for big things, like drawings:");

	// 		ui.heading("Draw with your mouse to paint:");
	// 		ui_state.painting.ui_control(ui);
	// 		egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
	// 			ui_state.painting.ui_content(ui);
	// 		});
	// 	});

	// 	egui::Window::new("Window")
	// 		.vscroll(true)
	// 		.open(&mut ui_state.is_window_open)
	// 		.show(ctx, |ui| {
	// 			ui.label("Windows can be moved by dragging them.");
	// 			ui.label("They are automatically sized based on contents.");
	// 			ui.label("You can turn on resizing and scrolling if you like.");
	// 			ui.label("You would normally chose either panels OR windows.");
	// 		});

	// 	if invert {
	// 		ui_state.inverted = !ui_state.inverted;
	// 	}
	// 	if load || invert {
	// 		// If an image is already added to the context, it'll return an existing texture id.
	// 		if ui_state.inverted {
	// 			*rendered_texture_id = contexts.add_image(images.bevy_icon_inverted.clone_weak());
	// 		} else {
	// 			*rendered_texture_id = contexts.add_image(images.bevy_icon.clone_weak());
	// 		};
	// 	}
	// 	if remove {
	// 		contexts.remove_image(&images.bevy_icon);
	// 		contexts.remove_image(&images.bevy_icon_inverted);
	// 	}
	// }
}
