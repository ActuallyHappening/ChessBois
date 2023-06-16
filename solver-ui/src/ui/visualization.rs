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
    	.add_system(spawn_chess_pieces.in_schedule(OnEnter(ChessEngineState::ViewValidPaths)))
    .add_system(despawn_chess_pieces.in_schedule(OnExit(ChessEngineState::ViewValidPaths)))
			// -
			;
	}
}

#[derive(Component, Debug, Clone)]
struct ChessPiece {
	x: u8,
	y: u8,
}

fn despawn_chess_pieces(mut commands: Commands, chess_pieces: Query<Entity, With<ChessPiece>>) {
	for entity in chess_pieces.iter() {
		commands.entity(entity).despawn_recursive();
	}
}

fn spawn_chess_pieces(mut commands: Commands) {
	#![allow(non_upper_case_globals)]

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
	const SQUARE_SIZE: f32 = 50.;
	const WIDTH: f32 = 8.;
	const HEIGHT: f32 = 8.;
	const MARGIN: f32 = 5.;

	for x in 1..=WIDTH.floor() as i32 {
		for y in 1..=HEIGHT.floor() as i32 {
			let translation = Vec3::new(
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
			);
			commands.spawn((
				SpriteBundle {
					sprite: Sprite {
						color: {
							if (x + y + 1) % 2 == 0 {
								Color::BLACK
							} else {
								Color::WHITE
							}
						},
						custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
						anchor: Anchor::BottomCenter,
						..default()
					},
					transform: Transform::from_translation(translation),
					..default()
				},
				Name::new(format!("Chess Square ({}, {})", x, y)),
				ChessPiece {
					x: x as u8,
					y: y as u8,
				},
			));
		}
	}
}
