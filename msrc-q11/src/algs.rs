use crate::{
	pieces::{ChessPiece, StandardKnight},
	*,
};
use cached::proc_macro::cached;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Computation {
	// Computing {
	// 	current: Moves,
	// },
	Successful {
		solution: Moves,
		explored_states: u128,
	},
	Failed {
		total_states: u128,
	},
}

enum PartialComputation {
	Successful { solution: Moves },
	Failed,
}

impl PartialComputation {
	fn add_state_count(self, count: u128) -> Computation {
		match self {
			Self::Successful { solution } => Computation::Successful {
				solution,
				explored_states: count,
			},
			Self::Failed => Computation::Failed { total_states: count },
		}
	}
}

impl From<Option<Moves>> for PartialComputation {
	fn from(moves: Option<Moves>) -> Self {
		match moves {
			Some(moves) => Self::Successful { solution: moves },
			None => Self::Failed,
		}
	}
}

pub enum ImplementedAlgorithms<P: ChessPiece + 'static> {
	Warnsdorf(P),

	BruteRecursive(P),
}

impl<P: ChessPiece> ImplementedAlgorithms<P> {
	// pub async fn tour_no_repeat(
	// 	&self,
	// 	options: BoardOptions,
	// 	start: ChessPoint,
	// ) -> Option<Moves> {
	// 	match self {
	// 		Self::Warnsdorf(piece) => warnsdorf_tour_repeatless(piece, options, start),
	// 		Self::BruteRecursive(piece) => {
	// 			brute_recursive_tour_repeatless_cached(piece, options, start).await
	// 		}
	// 	}
	// }

	pub async fn tour_computation(&self, options: BoardOptions, start: ChessPoint) -> Computation {
		match self {
			Self::Warnsdorf(piece) => warnsdorf_tour_repeatless(piece, options, start),
			_ => unimplemented!(),
		}
	}
}

use std::collections::BTreeMap;

use super::*;
impl ChessPoint {
	fn displace(&self, dx: u8, dy: u8) -> Self {
		Self::new(self.row + dy, self.column + dx)
	}
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
enum CellState {
	NeverOccupied,
	PreviouslyOccupied,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Board {
	cell_states: BTreeMap<ChessPoint, CellState>,
}

impl Board {
	fn get(&self, p: &ChessPoint) -> Option<CellState> {
		self.cell_states.get(p).cloned()
	}
	fn set(&mut self, p: ChessPoint, state: CellState) {
		self.cell_states.insert(p, state);
	}

	fn from_options(options: BoardOptions) -> Self {
		Board {
			cell_states: options
				.get_available_points()
				.into_iter()
				.map(|p| (p, CellState::NeverOccupied))
				.collect(),
		}
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
	fn get_degree(&self, p: &ChessPoint, piece: &impl ChessPiece, state_counter: &mut u128) -> u16 {
		let mut degree = 0;
		for &(dx, dy) in piece.relative_moves() {
			*state_counter += 1;
			
			let p = p.mov(&(dx, dy));
			if self.get(&p) == Some(CellState::NeverOccupied) {
				degree += 1;
			}
		}
		degree
	}
}

pub fn warnsdorf_tour_repeatless<P: ChessPiece>(
	piece: &P,
	options: BoardOptions,
	start: ChessPoint,
) -> Computation {
	let mut board = Board::from_options(options);
	let mut moves = Vec::new();
	let mut current = start;

	let num_available_cells = board.cell_states.len();
	let mut state_counter = 0_u128;

	for _ in 1..num_available_cells {
		if !board.cell_states.contains_key(&current) {
			return Computation::Failed {
				total_states: board.cell_states.len() as u128,
			};
		}

		board.set(current, CellState::PreviouslyOccupied);

		let mut next = None;
		let mut min_degree = u16::MAX;
		for &(dx, dy) in piece.relative_moves() {
			let p = current.mov(&(dx, dy));
			state_counter += 1;

			if board.get(&p) == Some(CellState::NeverOccupied) {
				let degree = board.get_degree(&p, piece, &mut state_counter);
				if degree < min_degree {
					min_degree = degree;
					next = Some(p);
				}
			}
		}

		if let Some(next) = next {
			moves.push(Move::new(current, next));
			current = next;
		} else {
			return Computation::Failed {
				total_states: state_counter,
			};
		}
	}

	Computation::Successful {
		solution: moves.into(),
		explored_states: state_counter,
	}
}

fn try_move_recursive(
	num_moves_required: u8,
	piece: &impl ChessPiece,
	attempting_board: Board,
	current_pos: ChessPoint,
	state_counter: &mut u128,
) -> PartialComputation {
	*state_counter += 1;

	if num_moves_required == 0 {
		// println!("Found solution!");
		// return Some(Vec::new().into());
		return PartialComputation::Successful {
			solution: vec![].into(),
		};
	}

	let mut available_moves = attempting_board.get_available_moves_from(&current_pos, piece);
	if available_moves.is_empty() {
		// println!("No moves available");
		// return None;
		return PartialComputation::Failed;
	}
	// sort by degree
	available_moves.sort_by_cached_key(|p| attempting_board.get_degree(p, piece, state_counter));
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
			state_counter,
		);

		match result {
			PartialComputation::Failed => {},
			PartialComputation::Successful { solution: mut working_moves } => {
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

	moves.into()
}

pub async fn brute_recursive_tour_repeatless_cached<P: ChessPiece + 'static>(
	piece: &P,
	options: BoardOptions,
	start: ChessPoint,
) -> Option<Moves> {
	let all_available_points = options.get_available_points();
	let num_moves_required = all_available_points.len() as u8 - 1;

	let mut state_counter = 0_u128;

	let board = Board::from_options(options);
	try_move_recursive_cached(num_moves_required, piece, board, start, &mut state_counter)
		.await
		.map(|moves| {
			let mut moves = moves.into_iter().rev().collect::<Vec<Move>>();
			moves.push(Move::new(start, start));
			moves.into()
		})
}

async fn try_move_recursive_cached<P: ChessPiece + 'static>(
	num_moves_required: u8,
	piece: &P,
	attempting_board: Board,
	current_pos: ChessPoint,
	state_counter: &mut u128,
) -> Option<Moves> {
	let state = State {
		start: current_pos,
		board: attempting_board.clone(),
	};
	if let Some(solution) = try_get_cached_solution::<P>(&state) {
		info!("Cache hit! Len: {}", solution.len());
		Some(solution)
	} else if let PartialComputation::Successful { solution } = try_move_recursive(
		num_moves_required,
		piece,
		attempting_board,
		current_pos,
		state_counter,
	) {
		add_solution_to_cache::<P>(state, solution.clone());
		Some(solution)
	} else {
		None
	}
}

use cache::{add_solution_to_cache, try_get_cached_solution, State};

mod cache {
	use super::*;
	use lru::LruCache;
	use once_cell::sync::Lazy;
	use std::num::NonZeroUsize;
	use std::{any::TypeId, collections::HashMap, sync::Mutex};
	use tracing::info;

	static CYCLE_CACHE: Lazy<Mutex<HashMap<TypeId, LruCache<State, Moves>>>> =
		Lazy::new(|| Mutex::new(HashMap::new()));

	fn new() -> LruCache<State, Moves> {
		LruCache::new(NonZeroUsize::new(10_000).unwrap())
	}

	#[derive(Hash, PartialEq, Eq, Clone, Debug)]
	pub struct State {
		pub start: ChessPoint,
		pub board: Board,
	}

	pub fn try_get_cached_solution<P: ChessPiece + 'static>(options: &State) -> Option<Moves> {
		let mut caches = CYCLE_CACHE.lock().unwrap();
		let id = TypeId::of::<P>();

		let cache = match caches.get_mut(&id) {
			Some(cache) => cache,
			None => {
				let cache = new();
				caches.insert(id, cache);
				caches.get_mut(&id).unwrap()
			}
		};

		cache.get(options).cloned()
	}

	pub fn add_solution_to_cache<P: ChessPiece + 'static>(options: State, moves: Moves) {
		let mut caches = CYCLE_CACHE.lock().unwrap();
		let id = TypeId::of::<P>();

		let cache = match caches.get_mut(&id) {
			Some(cache) => cache,
			None => {
				let cache = new();
				caches.insert(id, cache);
				caches.get_mut(&id).unwrap()
			}
		};

		info!("Putting a solution in the cache");
		cache.put(options, moves);
	}
}

#[cfg(test)]
mod benchmarks {
	use crate::pieces::StandardKnight;

	use super::*;

	extern crate test;
	use test::Bencher;

	// #[bench]
	// fn normal_square_4x4_removed(b: &mut Bencher) {
	// 	let mut options = BoardOptions::new(8, 8);

	// 	for y in 3..=6 {
	// 		for x in 3..=6 {
	// 			options.rm((x, y));
	// 		}
	// 	}

	// 	let start = ChessPoint::new(1, 1);
	// 	let piece = StandardKnight {};

	// 	b.iter(move || {
	// 		ImplementedAlgorithms::BruteRecursive(piece.clone()).tour_no_repeat(options.clone(), start)
	// 	});
	// }

	// #[bench]
	// fn normal_4x4_removed_minus_2(b: &mut Bencher) {
	// 	let mut options = BoardOptions::new(8, 8);

	// 	for y in 3..=6 {
	// 		for x in 3..=6 {
	// 			options.rm((x, y));
	// 		}
	// 	}
	// 	options.add((4, 5));
	// 	options.add((5, 4));

	// 	let start = ChessPoint::new(1, 1);
	// 	let piece = StandardKnight {};

	// 	b.iter(move || {
	// 		ImplementedAlgorithms::BruteRecursive(piece.clone()).tour_no_repeat(options.clone(), start)
	// 	});
	// }

	// #[bench]
	// fn normal_square_2x2_removed(b: &mut Bencher) {
	// 	let mut options = BoardOptions::new(8, 8);

	// 	for y in 4..=5 {
	// 		for x in 4..=5 {
	// 			options.rm((x, y));
	// 		}
	// 	}

	// 	let start = ChessPoint::new(1, 1);
	// 	let piece = StandardKnight {};

	// 	b.iter(move || {
	// 		ImplementedAlgorithms::BruteRecursive(piece.clone()).tour_no_repeat(options.clone(), start)
	// 	});
	// }
}
