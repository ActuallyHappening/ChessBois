use crate::ChessPoint;

pub trait ChessPiece {
	fn relative_moves(&self) -> &[(i16, i16)];

	fn is_valid_move(&self, from: ChessPoint, to: ChessPoint) -> bool {
		// checks self.relative_moves
		let dx = to.column as i16 - from.column as i16;
		let dy = to.row as i16 - from.row as i16;
		self.relative_moves().contains(&(dx, dy))
	}
}

#[derive(Copy, Clone)]
pub struct StandardKnight;

impl ChessPiece for StandardKnight {
	fn relative_moves(&self) -> &[(i16, i16)] {
		&[
			(2, 1),
			(1, 2),
			(-1, 2),
			(-2, 1),
			(-2, -1),
			(-1, -2),
			(1, -2),
			(2, -1),
		]
	}
}