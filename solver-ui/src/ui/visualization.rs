use super::*;
use bevy::{prelude::*, sprite::Anchor};

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

fn get_spacial_coord(chess_position: ChessSquareUi) -> Vec3 {
	let ChessSquareUi { x, y } = chess_position;

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

fn spawn_chess_pieces(mut commands: Commands, selected: Res<ChessSquareSelected>) {
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
					transform: Transform::from_translation(get_spacial_coord(ChessSquareUi { x: x as u8, y: y as u8 })),
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
}
