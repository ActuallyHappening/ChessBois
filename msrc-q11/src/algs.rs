use crate::{pieces::ChessPiece, *};
use cached::proc_macro::cached;

pub enum ImplementedAlgorithms<P: ChessPiece> {
	Warnsdorf(P),

	/// This variant stores known board_options already solves, and re-arranges the moves
	/// to start at the expected start location instead of re-applying the Warnsdorf ImplementedAlgorithms
	/// again.
	WarnsdorfCached(P),
}

impl<P: ChessPiece> ImplementedAlgorithms<P> {
	pub fn tour_no_repeat(&self, board_options: BoardOptions, start: ChessPoint) -> Option<Moves> {
		match self {
			Self::Warnsdorf(piece) => warnsdorf_tour_repeatless(piece, board_options, start),
			Self::WarnsdorfCached(piece) => brute_force_tour_repeatless(piece, board_options, start),
		}
	}
}

fn warnsdorf_tour_repeatless(
	piece: &impl ChessPiece,
	options: BoardOptions,
	start: ChessPoint,
) -> Option<Moves> {
	struct Board {
		cell_states: CellStates,
		options: BoardOptions,
	}

	impl Default for Board {
		fn default() -> Self {
			Self::new(8, 8)
		}
	}

	impl Display for Board {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			for row in self.cell_states.iter() {
				for cell in row.iter() {
					match cell {
						CellState::NeverOccupied => write!(f, " 0 ")?,
						CellState::HasBeenOccupied(n) => write!(f, "{:2} ", n)?,
						CellState::Unavailable => write!(f, " X ")?,
					}
				}
				writeln!(f)?;
			}
			Ok(())
		}
	}

	impl Board {
		/// Creates square board with given dimensions and all cells available
		pub fn new(rows: u8, columns: u8) -> Self {
			let options = BoardOptions::new(rows, columns);
			Self {
				options: options.clone(),
				cell_states: options.into(),
			}
		}

		pub fn from_options(cell_options: BoardOptions) -> Self {
			let cell_states = cell_options
				.clone()
				.options
				.into_iter()
				.map(|row| row.into_iter().map(|cell| cell.into()).collect())
				.collect();
			Self {
				cell_states,
				options: cell_options,
			}
		}

		pub fn options(&self) -> &BoardOptions {
			&self.options
		}

		fn get(&self, p: &ChessPoint) -> Option<CellState> {
			if !self.options.validate_point(p) {
				return None;
			}
			Some(self.cell_states[p.row as usize - 1][p.column as usize - 1])
		}

		fn set(&mut self, p: ChessPoint, state: CellState) {
			self.cell_states[p.row as usize - 1][p.column as usize - 1] = state;
		}

		/// Returns true if point is NeverOccupied.
		/// Returns None if point is invalid
		fn get_availability_no_repeat(&self, p: &ChessPoint) -> Option<bool> {
			match self.get(p) {
				Some(CellState::NeverOccupied) => Some(true),
				Some(CellState::HasBeenOccupied(_)) | Some(CellState::Unavailable) => Some(false),
				None => None,
			}
		}

		/// Returns bool if point is NeverOccupied or HasBeenOccupied
		// fn get_availability_allowing_repeat(&self, p: ChessPoint) -> bool {
		// 	self.validate_point_or_panic(p);

		// 	matches!(
		// 		self.get(p),
		// 		CellState::NeverOccupied | CellState::HasBeenOccupied(_)
		// 	)
		// }

		fn get_degree_no_repeat(&self, start: ChessPoint, moves: &impl ChessPiece) -> u16 {
			self.options().validate_point_or_panic(&start);

			let mut degree = 0;
			for &(dx, dy) in moves.relative_moves() {
				let p = start.mov(&(dx, dy));
				if self.get_availability_no_repeat(&p) == Some(true) {
					degree += 1;
				}
			}
			degree
		}

		// fn get_degree_allowing_repeat(&self, start: ChessPoint, moves: &impl ChessPiece) -> u16 {
		// 	self.validate_point_or_panic(start);

		// 	let mut degree = 0;
		// 	for &(dx, dy) in moves.relative_moves() {
		// 		let p = start.mov(&(dx, dy));
		// 		if self.get_availability_allowing_repeat(p) {
		// 			degree += 1;
		// 		}
		// 	}
		// 	degree
		// }

		pub fn all_unvisited_available_points(&self) -> Vec<ChessPoint> {
			let mut points = Vec::new();
			for row in 1..=self.options().height() {
				for column in 1..=self.options().width() {
					let p = ChessPoint::new(row, column);
					if self.get_availability_no_repeat(&p) == Some(true) {
						points.push(p);
					}
				}
			}
			points
		}
	}

	impl Deref for Board {
		type Target = CellStates;

		fn deref(&self) -> &Self::Target {
			&self.cell_states
		}
	}

	let mut board = Board::from_options(options);
	let mut moves = Vec::new();
	let mut current = start;

	let num_available_cells = board
		.options()
		.get_all_points()
		.iter()
		.filter(|p| board.get_availability_no_repeat(p) == Some(true))
		.count();

	for _ in 1..num_available_cells {
		if !board.options().validate_point(&current) {
			return None;
		}

		// board.cell_states[current.row as usize - 1][current.column as usize - 1] =
		// CellState::HasBeenOccupied(moves.len() as u8 + 1);
		board.set(current, CellState::HasBeenOccupied(moves.len() as u8 + 1));

		let mut next = None;
		let mut min_degree = u16::MAX;
		for &(dx, dy) in piece.relative_moves() {
			let p = current.mov(&(dx, dy));
			if board.get_availability_no_repeat(&p) == Some(true) {
				let degree = board.get_degree_no_repeat(p, piece);
				if degree < min_degree {
					min_degree = degree;
					next = Some(p);
				}
			}
		}

		if let Some(next) = next {
			moves.push(
				Move::new_checked(current, next, board.options()).expect("moves generated to be valid"),
			);
			current = next;
		} else {
			return None;
		}
	}

	Some(moves.into())
}

fn warnsdorf_tour_repeatless_cached(
	piece: &impl ChessPiece,
	options: BoardOptions,
	start: ChessPoint,
) -> Option<Moves> {

	unimplemented!()
}



#[cfg(test)]
mod tests {
	#[test]
	fn test_caching() {}
}

fn brute_force_tour_repeatless(
	piece: &impl ChessPiece,
	options: BoardOptions,
	start: ChessPoint,
) -> Option<Moves> {
	unimplemented!()
}
