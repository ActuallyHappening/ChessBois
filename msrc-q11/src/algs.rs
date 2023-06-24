use crate::{pieces::ChessPiece, *};
use cached::proc_macro::cached;

pub enum ImplementedAlgorithms<P: ChessPiece + 'static> {
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
			Self::WarnsdorfCached(piece) => warnsdorf_tour_repeatless_cached(piece, board_options, start),
		}
	}
}


fn warnsdorf_tour_repeatless_cached<P: ChessPiece + 'static>(
	piece: &P,
	options: BoardOptions,
	start: ChessPoint,
) -> Option<Moves> {
	if let Some(cached_cycle) = try_get_cached_cycle::<P>(&options) {
		cached_cycle.generate_moves_starting_at(start)
	} else {
		let moves = warnsdorf_tour_repeatless(piece, options.clone(), start);
		if let Some(moves) = moves {
			add_cycle_to_cache::<P>(options, moves.clone());
			Some(moves)
		} else {
			None
		}
	}
}

use cache::*;
mod cache {
	use crate::{pieces::ChessPiece, BoardOptions, ChessPoint, Move, Moves};
	use lru::LruCache;
	use once_cell::sync::Lazy;
use tracing::info;
	use std::num::NonZeroUsize;
	use std::ops::{Deref, DerefMut};
	use std::{any::TypeId, collections::HashMap, sync::Mutex};

	static CYCLE_CACHE: Lazy<Mutex<HashMap<TypeId, LruCache<BoardOptions, Cycle>>>> =
		Lazy::new(|| Mutex::new(HashMap::new()));

	#[derive(Debug, Clone, PartialEq, Eq, Hash)]
	pub struct Cycle {
		// options:
		original_moves: Moves,
	}

	impl Cycle {
		pub fn new(original_moves: Moves) -> Self {
			Self { original_moves }
		}

		pub fn generate_moves_starting_at(self, start: ChessPoint) -> Option<Moves> {
			let mut moves = self.original_moves;

			// find index of start
			let start_index = moves.iter().position(|m| m.from == start)?;

			// reorder moves so that start is first, and wrap around
			let moves: &mut Vec<Move> = moves.deref_mut();

			moves.rotate_left(start_index);
			// moves.push(moves[0]);
			// moves.rotate_left(1);

			let moves = moves.deref().clone();

			Some(moves.into())
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_gen_moves_from_cycle() {
			let initial = Moves::new(vec![
				Move::new(ChessPoint::new(1, 1), ChessPoint::new(2, 2)),
				Move::new(ChessPoint::new(2, 2), ChessPoint::new(3, 3)),
				Move::new(ChessPoint::new(3, 3), ChessPoint::new(4, 4)),
				Move::new(ChessPoint::new(4, 4), ChessPoint::new(1, 1)),
			]);
			let new_start_pos = ChessPoint::new(3, 3);
			let expected = Moves::new(vec![
				Move::new(ChessPoint::new(3, 3), ChessPoint::new(4, 4)),
				Move::new(ChessPoint::new(4, 4), ChessPoint::new(1, 1)),
				Move::new(ChessPoint::new(1, 1), ChessPoint::new(2, 2)),
				Move::new(ChessPoint::new(2, 2), ChessPoint::new(3, 3)),
			]);

			let cycle = Cycle::new(initial);
			let actual = cycle.generate_moves_starting_at(new_start_pos).unwrap();

			println!("Expected: \n{expected}, got: \n{actual}");

			assert_eq!(actual, expected)
		}
	}

	pub fn try_get_cached_cycle<P: ChessPiece + 'static>(options: &BoardOptions) -> Option<Cycle> {
		let mut caches = CYCLE_CACHE.lock().unwrap();
		let id = TypeId::of::<P>();

		let cache = match caches.get_mut(&id) {
			Some(cache) => cache,
			None => {
				let cache = LruCache::new(NonZeroUsize::new(100).unwrap());
				caches.insert(id, cache);
				caches.get_mut(&id).unwrap()
			}
		};

		cache.get(options).cloned()
	}

	pub fn add_cycle_to_cache<P: ChessPiece + 'static>(options: BoardOptions, moves: Moves) {
		let mut caches = CYCLE_CACHE.lock().unwrap();
		let id = TypeId::of::<P>();

		let cache = match caches.get_mut(&id) {
			Some(cache) => cache,
			None => {
				let cache = LruCache::new(NonZeroUsize::new(100).unwrap());
				caches.insert(id, cache);
				caches.get_mut(&id).unwrap()
			}
		};

		info!("Putting a solution in the cache");
		cache.put(options, Cycle::new(moves));
	}
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
