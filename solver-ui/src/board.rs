use std::f32::consts::TAU;

use bevy::prelude::*;
use msrc_q11::{Board, CellOptions, ChessPoint};

use crate::{CELL_DEPTH, CELL_HEIGHT, CELL_SELECTED, CELL_SIZE};

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_system(handle_options_changed)
			.add_event::<OptionsChanged>()
			.add_startup_system(debug_trigger_default_board);
	}
}

#[derive(Debug, Clone)]
struct Options {
	options: CellOptions,
	selected_start: ChessPoint,
}

#[derive(Debug, Clone)]
pub struct OptionsChanged {
	new: Options,
	old: Options,
}

#[derive(Resource, Debug, Clone)]
struct CurrentOptions {
	current: Options,
}

// fn get_spacial_coord(board: &Board, chess_position: ChessPoint) -> Vec3 {
// 	let ChessPoint { row: y, column: x } = chess_position;

// 	let total_width = board.width() as f32 * (CELL_SIZE + CELL_MARGIN) - CELL_MARGIN;
// 	let total_height = board.height() as f32 * (CELL_SIZE + CELL_MARGIN) - CELL_MARGIN;

// 	Vec3::new(
// 		{
// 			// get x position, assuming margin between every square

// 			// X position from center -> towards right
// 			let full_x = x as f32 * (CELL_SIZE + CELL_MARGIN);
// 			full_x - total_width / 2.
// 		},
// 		CELL_HEIGHT,
// 		{
// 			// repeat for y
// 			let full_y_up = y as f32 * (CELL_SIZE + CELL_MARGIN);
// 			let full_y_down = total_height - full_y_up;
// 			// full_y_up - total_height / 2.
// 			full_y_down + total_height / 2.
// 		},
// 	)
// }

/// Returns spacial coordinates of center of cell mesh
fn get_spacial_coord_normalized(board: &Board, chess_position: ChessPoint) -> Vec2 {
	let ChessPoint { row: y, column: x } = chess_position;
	let x = x as f32;
	let y = y as f32;
	let width = board.width() as f32;
	let height = board.height() as f32;

	// normalized: (column, row) = (x, y)
	// Adjusted = ((x - 1) -X Delta, (y - 1) - Y Delta)
	// X Delta = (width - 1) / 2

	let x_adjusted = (x - 1.) - (width - 1.) / 2.;
	let y_adjusted = (y - 1.) - (height - 1.) / 2.;

	Vec2::new(x_adjusted, y_adjusted)
}

fn get_spacial_coord(board: &Board, chess_position: ChessPoint) -> Vec3 {
	let normalized = get_spacial_coord_normalized(board, chess_position) * CELL_SIZE;

	Vec3::new(normalized.x, CELL_HEIGHT, normalized.y)
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
	fn test_coords_bl() {
		let coords = get_spacial_coord_normalized(&Board::new(2, 2), ChessPoint::new(1, 1));

		assert_eq!(coords, Vec2::new(-0.5, -0.5));
	}
}

pub fn debug_trigger_default_board(mut new_options: EventWriter<OptionsChanged>) {
	let options = Options {
		options: CellOptions::new(8, 8),
		selected_start: ChessPoint::new(4, 5),
	};
	new_options.send(OptionsChanged {
		new: options.clone(),
		old: options,
	});
}

pub fn handle_options_changed(
	mut commands: Commands,
	options: EventReader<OptionsChanged>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,

	cells: Query<Entity, With<ChessPoint>>,
) {
	if let Some(event) = get_options(options) {
		despawn_cells(&mut commands, cells);

		let new_options = event.new;
		let board = Board::from_options(new_options.options.clone());
		let start = new_options.selected_start;

		for point in board.all_unvisited_available_points() {
			let colour = if point == start {
				CELL_SELECTED
			} else {
				point.get_standard_colour()
			};
			spawn_cell(
				point,
				&board,
				colour,
				&mut commands,
				&mut meshes,
				&mut materials,
			);
		}
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
	));
}

/// Returns options, None if none, or panics if multiple events
fn get_options(mut event_stream: EventReader<OptionsChanged>) -> Option<OptionsChanged> {
	let mut iter = event_stream.into_iter();
	let options = iter.next();
	match options {
		None => None,
		Some(first) => {
			if iter.next().is_some() {
				panic!("Multiple options events");
			} else {
				Some(first.clone())
			}
		}
	}
}
