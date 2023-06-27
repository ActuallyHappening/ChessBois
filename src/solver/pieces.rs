pub trait ChessPiece {
	fn relative_moves(&self) -> &[(i16, i16)];
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