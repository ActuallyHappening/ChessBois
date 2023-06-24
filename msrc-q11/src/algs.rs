use std::collections::HashMap;

use crate::{pieces::ChessPiece, *};
use cached::proc_macro::cached;

pub enum ImplementedAlgorithms<P: ChessPiece + 'static> {
	Warnsdorf(P),

	BruteForce(P),
}

impl<P: ChessPiece> ImplementedAlgorithms<P> {
	pub fn tour_no_repeat(&self, board_options: BoardOptions, start: ChessPoint) -> Option<Moves> {
		match self {
			Self::Warnsdorf(piece) => warnsdorf_tour_repeatless(piece, board_options, start),
			Self::BruteForce(piece) => {
				brute_force_tour_repeatless(piece, board_options, start)
			}
		}
	}
}

use brute_force::brute_force_tour_repeatless;
mod brute_force {
	use super::*;
	impl ChessPoint {
		fn displace(&self, dx: u8, dy: u8) -> Self {
			Self::new(self.row + dy, self.column + dx)
		}
	}

	#[derive(Debug, Clone, PartialEq)]
	enum CellState {
		NeverOccupied,
		PreviouslyOccupied,
	}

	#[derive(Debug, Clone)]
	struct Board {
		cell_states: HashMap<ChessPoint, CellState>,
	}

	impl Board {
		fn get(&self, p: &ChessPoint) -> Option<CellState> {
			self.cell_states.get(p).cloned()
		}
		fn set(&mut self, p: ChessPoint, state: CellState) {
			self.cell_states.insert(p, state);
		}
		fn from_options(options: BoardOptions) -> Self {
			let mut cell_states = HashMap::new();
			for row in 1..=options.height() {
				for column in 1..=options.width() {
					let p = ChessPoint::new(row, column);
					cell_states.insert(p, CellState::NeverOccupied);
				}
			}
			Self { cell_states }
		}

		fn get_available_moves_from(&self, p: &ChessPoint, piece: &impl ChessPiece) -> Vec<ChessPoint> {
			let mut moves = Vec::new();
			for &(dx, dy) in piece.relative_moves() {
				let p = p.mov(&(dx, dy));
				if self.get(&p) == Some(CellState::NeverOccupied) {
					moves.push(p);
				}
			}
			moves
		}
		fn get_degree(&self, p: &ChessPoint, piece: &impl ChessPiece) -> u16 {
			let mut degree = 0;
			for &(dx, dy) in piece.relative_moves() {
				let p = p.mov(&(dx, dy));
				if self.get(&p) == Some(CellState::NeverOccupied) {
					degree += 1;
				}
			}
			degree
		}
	}

	pub fn brute_force_tour_repeatless<P: ChessPiece>(
		piece: &P,
		options: BoardOptions,
		start: ChessPoint,
	) -> Option<Moves> {
		let all_available_points = options.get_available_points();
		let num_moves_required = all_available_points.len() as u8 - 1;

		let board = Board::from_options(options);
		try_move_recursive(num_moves_required, piece, board, start)
			// .map(|moves| moves.into_iter().rev().collect())
    .map(|moves| {
			let mut moves = moves.into_iter().rev().collect::<Vec<Move>>();
			moves.push(Move::new(start, start));
			moves.into()
		})
	}

	fn try_move_recursive(
		num_moves_required: u8,
		piece: &impl ChessPiece,
		attempting_board: Board,
		current_pos: ChessPoint,
	) -> Option<Moves> {
		if num_moves_required == 0 {
			// println!("Found solution!");
			return Some(Vec::new().into());
		}

		let mut available_moves = attempting_board.get_available_moves_from(&current_pos, piece);
		if available_moves.is_empty() {
			// println!("No moves available");
			return None;
		}
		// sort by degree
		available_moves.sort_by_cached_key(|p| attempting_board.get_degree(p, piece));
		// println!("Available moves: {:?}", available_moves);

		let mut moves = None;

		for potential_next_move in available_moves {
			let mut board_with_potential_move = attempting_board.clone();

			board_with_potential_move.set(current_pos, CellState::PreviouslyOccupied);

			let result = try_move_recursive(
				num_moves_required - 1,
				piece,
				board_with_potential_move,
				potential_next_move,
			);

			match result {
				None => {}
				Some(mut working_moves) => {
					// initially, working_moves will be empty
					// first iteration must add move from current_pos to potential_next_move
					// this repeats
					working_moves.push(Move::new(current_pos, potential_next_move));
					// return Some(working_moves);
					moves = Some(working_moves);
					break;
				}
			};
		}

		moves
	}

	#[cfg(test)]
	mod tests {
		use crate::pieces::StandardKnight;

		use super::*;

		#[test]
		fn single_recursion_test() {
			let piece = StandardKnight {};
			let mut states: HashMap<ChessPoint, CellState> = HashMap::new();

			states.insert((1, 1).into(), CellState::NeverOccupied);

			let board = Board {
				cell_states: states,
			};

			let result = try_move_recursive(0, &piece, board, (1, 1).into());

			assert_eq!(result, Some(Moves::new(vec![])));
		}

		#[test]
		fn double_recursion_test() {
			let piece = StandardKnight {};
			let mut states: HashMap<ChessPoint, CellState> = HashMap::new();

			states.insert((1, 1).into(), CellState::NeverOccupied);
			states.insert((2, 3).into(), CellState::NeverOccupied);

			let board = Board {
				cell_states: states,
			};

			let result = try_move_recursive(1, &piece, board, (1, 1).into());

			assert_eq!(
				result,
				Some(Moves::new(vec![Move::new((1, 1).into(), (2, 3).into())]))
			);
		}

		#[test]
		fn triple_recursion_test() {
			let piece = StandardKnight {};
			let mut states: HashMap<ChessPoint, CellState> = HashMap::new();

			states.insert((1, 1).into(), CellState::NeverOccupied);
			states.insert((2, 3).into(), CellState::NeverOccupied);
			states.insert((3, 1).into(), CellState::NeverOccupied);

			let board = Board {
				cell_states: states,
			};
			// actually reversed
			let expected = vec![
				Move::new((1, 1).into(), (2, 3).into()),
				Move::new((2, 3).into(), (3, 1).into()),
			];

			let result = try_move_recursive(2, &piece, board, (1, 1).into());
			assert_eq!(result, Some(Moves::new(expected.into_iter().rev().collect())));
		}

		// #[test]
		// fn test_brute_force() {
		// 	let options = BoardOptions::new(5, 5);
		// 	let start = ChessPoint::new(1, 1);
		// 	let piece = StandardKnight {};

		// 	let expected = ImplementedAlgorithms::Warnsdorf(piece.clone())
		// 		.tour_no_repeat(options.clone(), start)
		// 		.unwrap();

		// 	let result = ImplementedAlgorithms::BruteForce(piece.clone())
		// 		.tour_no_repeat(options.clone(), start)
		// 		.unwrap();

		// 	println!("Expected:\n {expected}, \nResult: {result}");

		// 	assert_eq!(result, expected);

		// 	// assert!(result.is_some());
		// 	// let result = result.unwrap();
		// 	// assert_eq!(result.len(), 63);
		// }
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
				.map(|row| {
					row
						.into_iter()
						.map(|cell| cell.into())
						.collect::<Vec<CellState>>()
				})
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

	/// State of active board
	#[derive(Copy, Clone, Debug, PartialEq)]
	pub enum CellState {
		/// Can be moved to but never has
		NeverOccupied,

		/// Has been occupied already
		/// (number indicates at what step)
		HasBeenOccupied(u8),

		/// Can't be moved to
		Unavailable,
	}

	impl From<CellOption> for CellState {
		fn from(o: CellOption) -> Self {
			match o {
				CellOption::Available => CellState::NeverOccupied,
				CellOption::Unavailable => CellState::Unavailable,
			}
		}
	}

	impl From<BoardOptions> for CellStates {
		fn from(options: BoardOptions) -> Self {
			options
				.options
				.into_iter()
				.map(|row| row.into_iter().map(|cell| cell.into()).collect())
				.collect()
		}
	}

	pub type CellStates = Vec<Vec<CellState>>;

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
