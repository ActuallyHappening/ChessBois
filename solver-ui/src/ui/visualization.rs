use std::f32::consts::PI;

use super::*;
use bevy::{
	prelude::*,
	sprite::{MaterialMesh2dBundle},
};
use msrc_q11::knights_tour;

pub struct VisualizationStatePlugin;
impl Plugin for VisualizationStatePlugin {
	fn build(&self, app: &mut App) {
		app
			// -
	  	.add_system(keyboard_back)
    	.add_system(spawn_chess_pieces.in_schedule(OnEnter(ChessEngineState::ViewValidPaths)))
    	.add_system(despawn_chess_pieces.in_schedule(OnExit(ChessEngineState::ViewValidPaths)))
			// -
			;
	}
}

#[derive(Component, Debug, Clone)]
struct ChessSquareVisual {
	x: u8,
	y: u8,
}

impl From<ChessSquareUi> for ChessSquareVisual {
	fn from(ui: ChessSquareUi) -> Self {
		Self { x: ui.x, y: ui.y }
	}
}

/// <ChessSquareUi> == (x, y)
impl<T: PartialEq<u8>> PartialEq<(T, T)> for ChessSquareVisual {
	fn eq(&self, (x, y): &(T, T)) -> bool {
		x == &self.x && y == &self.y
	}
}

fn keyboard_back(
	keyboard_input: Res<Input<KeyCode>>,
	mut state: ResMut<NextState<ChessEngineState>>,
) {
	if keyboard_input.just_pressed(KeyCode::Escape) || keyboard_input.just_pressed(KeyCode::B) {
		state.set(ChessEngineState::PickStartingPosition);
	}
}

fn despawn_chess_pieces(
	mut commands: Commands,
	chess_pieces: Query<Entity, With<ChessSquareVisual>>,
) {
	for entity in chess_pieces.iter() {
		commands.entity(entity).despawn_recursive();
	}
}

const WIDTH: f32 = 8.;
const HEIGHT: f32 = 8.;
const SQUARE_SIZE: f32 = 5.;
const MARGIN: f32 = 1.;

fn get_spacial_coord(chess_position: ChessSquareVisual) -> Vec3 {
	let ChessSquareVisual { x, y } = chess_position;

	Vec3::new(
		{
			// get x position, assuming margin between every square
			let total_width = WIDTH * (SQUARE_SIZE + MARGIN) - MARGIN;
			let full_x = x as f32 * (SQUARE_SIZE + MARGIN);
			full_x - total_width / 2.
		},
		{
			// repeat for y
			let total_height = HEIGHT * (SQUARE_SIZE + MARGIN) - MARGIN;
			let full_y = y as f32 * (SQUARE_SIZE + MARGIN);
			full_y - total_height / 2.
		},
		-5.,
	)
}

fn spawn_path_line(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
	from: &ChessSquareVisual,
	to: &ChessSquareVisual,
) {
	let from = get_spacial_coord(from.clone());
	let to = get_spacial_coord(to.clone());

	let center = (from + to) / 2.;
	let length = (from - to).length();
	let angle: f32 = (to.y - from.y).atan2(to.x - from.x);

	let transform = Transform::from_translation(center + Vec3::new(0., 0., 4.))
	// .looking_at(to, Vec3::Y)
	.with_rotation(Quat::from_rotation_z(angle))
	// -
	;

	// info!("Transform: {:?}", transform);
	info!("Angle: {:?}, Length: {:?}", angle, length);

	let mesh_thin_rectangle = meshes
		.add(shape::Box::new(length, 1., 1.).into());

	commands.spawn(PbrBundle {
		mesh: mesh_thin_rectangle,
		material: materials.add(Color::GREEN.into()),
		transform,
		..default()
	});
}

fn spawn_chess_pieces(
	mut commands: Commands,
	selected: Res<ChessSquareSelected>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	info!("Spawning chess board visualization ...");
	let selected: ChessSquareVisual = selected.selected.expect("No square selected?").into();

	// ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(500.0).into()),
        material: materials.add(Color::SILVER.into()),
				// transform to be behind, xy plane
				transform: Transform::from_xyz(0., 0., -10.).with_rotation(Quat::from_rotation_x(PI / 2.)),
        ..default()
    });

	// commands.spawn(
	// 	(SpriteBundle {
	// 		sprite: Sprite {
	// 			color: Color::BLACK,
	// 			custom_size: Some(Vec2::new(100.0, 100.0)),
	// 			..default()
	// 		},
	// 		..default()
	// 	}),
	// );

	for x in 1..=WIDTH.floor() as i32 {
		for y in 1..=HEIGHT.floor() as i32 {
			let colour = if selected == (x as u8, y as u8) {
				Color::RED
			} else if (x + y + 1) % 2 == 0 {
				Color::BLACK
			} else {
				Color::WHITE
			};
			let mesh = meshes.add(shape::Box::new(SQUARE_SIZE, SQUARE_SIZE, 1.).into());
			commands.spawn((
				PbrBundle {
					mesh,
					material: materials.add(StandardMaterial::from(colour)),
					transform: Transform::from_translation(get_spacial_coord(ChessSquareVisual {
						x: x as u8,
						y: y as u8,
					})).with_scale(Vec3::new(1., 1., 2.)),
					..default()
				},
				Name::new(format!("Chess Square ({}, {})", x, y)),
				ChessSquareVisual {
					x: x as u8,
					y: y as u8,
				},
			));
		}
	}

	match knights_tour(selected.x as i32, selected.y as i32) {
		Some((_, moves)) => {
			for (from, to) in moves {
				spawn_path_line(
					&mut commands,
					&mut meshes,
					&mut materials,
					&ChessSquareVisual { x: from.x as u8, y: from.y as u8 },
					&ChessSquareVisual { x: to.x as u8, y: to.y as u8 },
				);
			}
			// spawn_path_line(
			// 	&mut commands,
			// 	&mut meshes,
			// 	&mut materials,
			// 	&selected,
			// 	&ChessSquareVisual { x: 2, y: 7 },
			// );
		},
		None => info!("Fail!"),
	}
	
	
}
