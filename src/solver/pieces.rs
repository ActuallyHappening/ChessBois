use serde::{Serialize, Deserialize};

use crate::ChessPoint;

/// Holds info on valid moves
#[derive(Hash, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ChessPiece {
	valid_moves: Vec<(i16, i16)>,
}

impl ChessPiece {
	pub fn new(moves: Vec<(i16, i16)>) -> Self {
		Self { valid_moves: moves }
	}

	pub fn relative_moves(&self) -> &Vec<(i16, i16)> {
		&self.valid_moves
	}

	pub fn is_valid_move(&self, from: ChessPoint, to: ChessPoint) -> bool {
		// checks self.relative_moves
		let dx = to.column as i16 - from.column as i16;
		let dy = to.row as i16 - from.row as i16;
		self.relative_moves().contains(&(dx, dy))
	}
}

/// Collection of standard sets of moves
pub enum Pieces {
	/// Same as [Pieces::ABKnight(1, 2)]
	StandardKnight,
	ABKnight(i8, i8),
}

impl Default for ChessPiece {
	fn default() -> Self {
		Pieces::StandardKnight.into()
	}
}

impl From<Pieces> for Vec<(i16, i16)> {
	fn from(value: Pieces) -> Self {
		match value {
			Pieces::StandardKnight => vec![
				(2, 1),
				(1, 2),
				(-1, 2),
				(-2, 1),
				(-2, -1),
				(-1, -2),
				(1, -2),
				(2, -1),
			],
			Pieces::ABKnight(a, b) => {
				let a = a as i16;
				let b = b as i16;
				vec![
					(a, b),
					(-a, b),
					(a, -b),
					(-a, -b),
					(b, a),
					(-b, a),
					(b, -a),
					(-b, -a),
				]
			}
		}
	}
}

impl From<Pieces> for ChessPiece {
	fn from(value: Pieces) -> Self {
		ChessPiece::new(value.into())
	}
}
