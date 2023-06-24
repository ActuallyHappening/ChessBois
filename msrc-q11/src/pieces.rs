pub trait ChessPiece {
	fn relative_moves(&self) -> &[(i8, i8)];
}

#[derive(Clone)]
pub struct StandardKnight;

impl ChessPiece for StandardKnight {
	fn relative_moves(&self) -> &[(i8, i8)] {
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