use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use msrc_q11::{BoardOptions, ChessPoint, SolverAlgorithm};
use std::f32::consts::TAU;

use crate::*;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_event::<NewCellSelected>()
			.add_event::<NewBoardCellOptions>()
			.add_startup_system(setup)
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

#[derive(Resource, Debug, Default)]
pub enum Algorithm {
	#[default]
	Warnsdorf,
}

use coords::*;
mod coords {
	use super::*;

	/// Returns spacial coordinates of center of cell mesh
	fn get_spacial_coord_normalized(board: &BoardOptions, chess_position: ChessPoint) -> Vec2 {
		let ChessPoint { row: y, column: x } = chess_position;
		let width = board.width() as f32;
		let height = board.height() as f32;
		let x = x as f32;
		let y = y as f32;

		// normalized: (column, row) = (x, y)
		// Adjusted = ((x - 1) -X Delta, (y - 1) - Y Delta)
		// X Delta = (width - 1) / 2

		let x_adjusted = (x - 1.) - (width - 1.) / 2.;
		let y_adjusted = (y - 1.) - (height - 1.) / 2.;

		Vec2::new(x_adjusted, y_adjusted)
	}

	pub fn get_spacial_coord(board: &BoardOptions, chess_position: ChessPoint) -> Vec3 {
		let normalized = get_spacial_coord_normalized(board, chess_position) * CELL_SIZE;

		Vec3::new(normalized.x, CELL_HEIGHT, -normalized.y)
	}

	pub fn get_spacial_coord_2d(board: &BoardOptions, chess_position: ChessPoint) -> Vec2 {
		let normalized = get_spacial_coord_normalized(board, chess_position) * CELL_SIZE;

		Vec2::new(normalized.x, -normalized.y)
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_coords_center() {
			let coords = get_spacial_coord_normalized(&BoardOptions::new(3, 3), ChessPoint::new(2, 2));

			assert_eq!(coords, Vec2::new(0., 0.));
		}

		#[test]
		fn test_coords_bl_2() {
			let coords = get_spacial_coord_normalized(&BoardOptions::new(2, 2), ChessPoint::new(1, 1));

			assert_eq!(coords, Vec2::new(-0.5, -0.5));
		}

		#[test]
		fn test_coords_bl_5() {
			let coords = get_spacial_coord_normalized(&BoardOptions::new(5, 5), ChessPoint::new(1, 1));

			assert_eq!(coords, Vec2::new(-2., -2.));
		}
	}
}

fn setup(
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
	commands.init_resource::<Algorithm>();

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
		let start = options.selected_start;
		let options = options.options.clone();

		for point in options.get_all_points() {
			let colour = compute_colour(&point, Some(&options), start);
			spawn_cell(point, &options, colour, commands, meshes, materials);
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
		options: &BoardOptions,
		colour: Color,
		commands: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
	) {
		let transform = Transform::from_translation(get_spacial_coord(options, at))
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
		algs: Res<Algorithm>,

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
			spawn_visualization_from_options(&new_options, algs, &mut commands, &mut meshes, &mut materials);
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
	use msrc_q11::{Move, StandardKnight, SolverAlgorithm};

	#[allow(dead_code)]
	#[derive(Component, Debug, Clone)]
	pub struct VisualizationComponent {
		from: ChessPoint,
		to: ChessPoint,
	}

	pub fn spawn_visualization_from_options(
		options: &Options,
		alg: Res<Algorithm>,

		commands: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
	) {
		if let Some(start) = options.selected_start {
			if options.options.get_unavailable_points().contains(&start) {
				// debug!("Start point is disabled!");
				return;
			}

			let options = options.options.clone();
			let piece = StandardKnight {};

			match piece.try_piece_tour_warnsdorf(options.clone(), start) {
				Some(moves) => {
					for Move { from, to } in moves.iter() {
						spawn_path_line(commands, meshes, materials, from, to, &options)
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
		options: &BoardOptions,
	) {
		let start_pos = get_spacial_coord_2d(options, *from);
		let end_pos = get_spacial_coord_2d(options, *to);

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
}
