use super::*;
use bevy::{
	prelude::*,
	sprite::{Anchor, MaterialMesh2dBundle},
};

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
const SQUARE_SIZE: f32 = 50.;
const MARGIN: f32 = 5.;

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
		0.,
	)
}

fn spawn_path_line(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<ColorMaterial>>,
	from: &ChessSquareVisual,
	to: &ChessSquareVisual,
) {
	let from = get_spacial_coord(from.clone());
	let to = get_spacial_coord(to.clone());

	let transform = Transform::from_translation(from)
	// .looking_at(to, Vec3::Y)
.with_rotation(Quat::from_rotation_z((45_f32).to_radians()))
.with_scale(Vec3::new(1., 1., 50.))
	// -
	;

	info!("Transform: {:?}", transform);

	// let mesh_circle = meshes.add(shape::Circle::new(50.).into()).into();
	let mesh_thin_rectangle = meshes.add(shape::Quad::new(Vec2::new(10., 50.)).into()).into();

	commands.spawn(MaterialMesh2dBundle {
		// mesh: mesh_circle,
		mesh: mesh_thin_rectangle,
		material: materials.add(ColorMaterial::from(Color::PURPLE)),
		transform,
		..default()
	});
}

fn spawn_chess_pieces(
	mut commands: Commands,
	selected: Res<ChessSquareSelected>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	info!("Spawning chess board visualization ...");
	let selected: ChessSquareVisual = selected.selected.expect("No square selected?").into();


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
			commands.spawn((
				SpriteBundle {
					sprite: Sprite {
						color: {
							if selected == (x as u8, y as u8) {
								Color::RED
							} else if (x + y + 1) % 2 == 0 {
								Color::BLACK
							} else {
								Color::WHITE
							}
						},
						custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
						anchor: Anchor::BottomCenter,
						..default()
					},
					transform: Transform::from_translation(get_spacial_coord(ChessSquareVisual {
						x: x as u8,
						y: y as u8,
					})),
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

	spawn_path_line(&mut commands, &mut meshes, &mut materials, &selected, &selected);
}
