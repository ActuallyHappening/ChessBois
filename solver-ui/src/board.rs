use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use msrc_q11::{Board, CellOptions, ChessPoint};
use std::f32::consts::TAU;

use crate::{
	CELL_DEPTH, CELL_HEIGHT, CELL_SELECTED, CELL_SIZE, VISUALIZATION_COLOUR, VISUALIZATION_HEIGHT,
};

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_event::<NewCellSelected>()
			.add_startup_system(spawn_initial_board)
			.add_plugins(
				DefaultPickingPlugins
					.build()
					.disable::<DefaultHighlightingPlugin>()
					.disable::<DebugPickingPlugin>(),
			)
			.add_system(handle_new_cell_selected_event);
	}
}

/// Represents information required to display cells + visual solutions
#[derive(Debug, Clone)]
pub struct Options {
	options: CellOptions,
	selected_start: ChessPoint,
}

#[derive(Debug, Clone)]
pub struct NewCellSelected {
	new: ChessPoint,
}

#[derive(Resource, Debug, Clone)]
pub struct CurrentOptions {
	current: Options,
}

use coords::get_spacial_coord;
mod coords {
	use super::*;

	/// Returns spacial coordinates of center of cell mesh
	fn get_spacial_coord_normalized(board: &Board, chess_position: ChessPoint) -> Vec2 {
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

	pub fn get_spacial_coord(board: &Board, chess_position: ChessPoint) -> Vec3 {
		let normalized = get_spacial_coord_normalized(board, chess_position) * CELL_SIZE;

		Vec3::new(normalized.x, CELL_HEIGHT, -normalized.y)
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

fn spawn_initial_board(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let options = Options {
		options: CellOptions::new(8, 8),
		selected_start: ChessPoint::new(1, 1),
	};
	let current_options = CurrentOptions {
		current: options.clone(),
	};

	commands.insert_resource(current_options);

	spawn_cells(&mut commands, &options, &mut meshes, &mut materials);
}

use cells::*;
mod cells {
	use super::*;

	pub fn spawn_cells(
		commands: &mut Commands,
		options: &Options,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
	) {
		let board = Board::from_options(options.options.clone());
		let start = options.selected_start;

		for point in board.all_unvisited_available_points() {
			let colour = if point == start {
				CELL_SELECTED
			} else {
				point.get_standard_colour()
			};
			spawn_cell(point, &board, colour, commands, meshes, materials);
		}
	}

	fn despawn_cells(commands: &mut Commands, cells: Query<Entity, With<ChessPoint>>) {
		for cell in cells.iter() {
			commands.entity(cell).despawn_recursive();
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
		));
	}

	fn cell_selected(
		// The first parameter is always the `ListenedEvent`, passed in by the event listening system.
		In(event): In<ListenedEvent<Over>>,

		mut materials: ResMut<Assets<StandardMaterial>>,

		square: Query<(&Handle<StandardMaterial>, &ChessPoint)>,
		current_options: ResMut<CurrentOptions>,

		mut new_cell_selected: EventWriter<NewCellSelected>,
	) -> Bubble {
		let (mat, point) = square.get(event.target).unwrap();

		let options = &current_options.current;
		if options.selected_start == *point {
			Bubble::Up
		} else {
			// sets colour to selected
			let material = materials.get_mut(mat).unwrap();
			material.base_color = CELL_SELECTED;

			// send event
			new_cell_selected.send(NewCellSelected { new: *point });

			Bubble::Up
		}
	}

	fn cell_deselected(
		In(event): In<ListenedEvent<Out>>,
		mut materials: ResMut<Assets<StandardMaterial>>,
		square: Query<(&Handle<StandardMaterial>, &ChessPoint)>,
	) -> Bubble {
		let (mat, point) = square.get(event.target).unwrap();

		// sets colour to selected
		let material = materials.get_mut(mat).unwrap();
		material.base_color = point.get_standard_colour();

		Bubble::Up
	}

	/// Handles re-constructing visual solution
	pub fn handle_new_cell_selected_event(
		mut new_starting_point: EventReader<NewCellSelected>,
		current_options: ResMut<CurrentOptions>,

		mut commands: Commands,
		mut meshes: ResMut<Assets<Mesh>>,
		mut materials: ResMut<Assets<StandardMaterial>>,
	) {
		let current_options = &current_options.current;
		if let Some(new_starting_point) = new_starting_point.into_iter().next() {
			let new_options = Options {
				options: current_options.options.clone(),
				selected_start: new_starting_point.new,
			};
			commands.insert_resource(CurrentOptions {
				current: new_options,
			});

			// TODO: Show visualization here!
			// info!("New starting point: {}", new_starting_point.new);
		}
	}
}

use visualization::*;
mod visualization {
	use super::*;
	use msrc_q11::{piece_tour_no_repeat, Move, StandardKnight};

	fn spawn_visualization_from_options(
		options: &Options,

		commands: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
	) {
		let mut board = Board::from_options(options.options.clone());
		let start = options.selected_start;
		let piece = StandardKnight {};

		let moves = piece_tour_no_repeat(&piece, &mut board, start).expect("tour to be possible");
		for Move { from, to } in moves.iter() {
			spawn_path_line(commands, meshes, materials, from, to, &board)
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
		let from = get_spacial_coord(board, *from);
		let to = get_spacial_coord(board, *to);

		let center = (from + to) / 2.;
		let length = (from - to).length();
		let angle: f32 = (to.y - from.y).atan2(to.x - from.x);

		let transform = Transform::from_translation(center + Vec3::new(0., VISUALIZATION_HEIGHT, 0.))
	// .looking_at(to, Vec3::Y)
	.with_rotation(Quat::from_rotation_y(angle))
	// -
	;

		// info!("Transform: {:?}", transform);
		// info!("Angle: {:?}, Length: {:?}", angle, length);

		let mesh_thin_rectangle = meshes.add(shape::Box::new(length, 1., 1.).into());

		commands.spawn(PbrBundle {
			mesh: mesh_thin_rectangle,
			material: materials.add(VISUALIZATION_COLOUR.into()),
			transform,
			..default()
		});
	}
}
