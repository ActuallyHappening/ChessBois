use rand::Rng;
use bevy::prelude::*;

#[derive(Debug, Reflect, Clone)]
pub struct ChessSquare {
	pub x: u8,
	pub y: u8,
}

#[derive(Debug, Reflect, Clone)]
struct ChessBoardProperties {
	pub width: u8,
	pub height: u8,
}

impl Default for ChessBoardProperties {
	fn default() -> Self {
		Self {
			width: 8,
			height: 8,
		}
	}
}

#[derive(Debug, Clone)]
pub enum Moves {
	End(ChessSquare),
	Continued(Box<Moves>),
}

pub fn debug_random(start: ChessSquare) -> Moves {
	let mut rng = rand::thread_rng();
	let mut moves = Moves::Continued(Box::new(Moves::End(start)));
	for _ in 0..10 {
		let x = rng.gen_range(1..=8);
		let y = rng.gen_range(1..=8);
		moves = Moves::Continued(Box::new(Moves::End(ChessSquare { x, y })));
	}
	info!("Generated your movements: {:?}", moves);
	moves
}