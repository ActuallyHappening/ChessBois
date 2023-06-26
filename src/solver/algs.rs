use crate::solver::{pieces::ChessPiece, *};
use strum::{EnumIter, IntoStaticStr};

mod hamiltonian;
use hamiltonian::hamiltonian_tour_repeatless;

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

#[derive(Clone)]
pub enum PartialComputation {
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
			Self::Failed => Computation::Failed {
				total_states: count,
			},
		}
	}

	fn map(self, f: impl FnOnce(Moves) -> Moves) -> Self {
		match self {
			Self::Successful { solution } => Self::Successful {
				solution: f(solution),
			},
			Self::Failed => Self::Failed,
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
	BruteForce(P),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, EnumIter, IntoStaticStr, Hash, PartialOrd, Ord)]
pub enum Algorithm {
	#[default]
	#[strum(serialize = "Warnsdorf")]
	Warnsdorf,

	#[strum(serialize = "Brute Force")]
	BruteForce,
}

impl Algorithm {
	pub fn to_impl<P: ChessPiece>(&self, piece: P) -> ImplementedAlgorithms<P> {
		match self {
			Algorithm::Warnsdorf => ImplementedAlgorithms::Warnsdorf(piece),
			Algorithm::BruteForce => ImplementedAlgorithms::BruteForce(piece),
		}
	}

	pub fn get_description(&self) -> &'static str {
		match self {
			Algorithm::Warnsdorf => "A standard knights tour.\
			This algorithm applies Warnsdorf's Rule, which tells you to always move to the square with the fewest available moves. \
			This algorithm is always guaranteed to terminate in finite time, however it sometimes misses solutions e.g. 8x8 board @ (5, 3).\
			Warnsdorf's Rule is very easy to implement and is very popular because of its simplicity. The implementation used is sub-optimal, but should suffice.
			", 
			Algorithm::BruteForce => "A standard knights tour.\
			This algorithm is a recursive brute-force approach, which favours Warnsdorf's Rule first before backtracking.\
			This algorithm is always guaranteed to terminate in finite time, but that time complexity is exponential compared with number of cells, so \
			large boards with no solutions will take a long time to solve. In worst case scenario, since it is brute force, it will check every possible \
			knights tour before exiting with no solution! However, if Warnsdorf's algorithm finds a solution, this algorithm will find that solution first.
			",
		}
	}
}

/// Represents information required to display cells + visual solutions
#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Options {
	pub options: BoardOptions,
	pub selected_start: Option<ChessPoint>,
	pub selected_algorithm: Algorithm,

	pub force_update: bool,
}

impl Options {
	pub fn with_start(&self, start: ChessPoint) -> Self {
		Self {
			selected_start: Some(start),
			..self.clone()
		}
	}
}

impl<P: ChessPiece> ImplementedAlgorithms<P> {
	/// Returns None if options.selected_start is None
	pub fn tour_computation_cached(&self, options: Options) -> Option<Computation> {
		if let Some(start) = options.selected_start {
			type ComputerFn<P> = dyn Fn(&P, BoardOptions, ChessPoint) -> Computation;
			let (piece, computer_fn): (&P, Box<ComputerFn<P>>) = match self {
				Self::Warnsdorf(piece) => (piece, Box::new(warnsdorf_tour_repeatless)),
				// Self::BruteForce(piece) => (piece, Box::new(brute_recursive_tour_repeatless)),
				Self::BruteForce(piece) => (piece, Box::new(hamiltonian_tour_repeatless)),
			};

			Some(if let Some(comp) = try_get_cached_solution::<P>(&options) {
				debug!("Solution cache hit!");
				comp
			} else {
				debug!("Cache miss");
				let comp = computer_fn(piece, options.options.clone(), start);
				add_solution_to_cache::<P>(options, comp.clone());
				comp
			})
		} else {
			None
		}
	}
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
enum CellState {
	NeverOccupied,
	PreviouslyOccupied,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Board {
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
			if let Some(p) = p.displace(&(dx, dy)) {
				if self.get(&p) == Some(CellState::NeverOccupied) {
					moves.push(p);
				}
			}
		}
		moves
	}
	fn get_degree(&self, p: &ChessPoint, piece: &impl ChessPiece) -> u16 {
		let mut degree = 0;
		for &(dx, dy) in piece.relative_moves() {
			if let Some(p) = p.displace(&(dx, dy)) {
				if self.get(&p) == Some(CellState::NeverOccupied) {
					degree += 1;
				}
			}
		}
		degree
	}
}

fn warnsdorf_tour_repeatless<P: ChessPiece>(
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
			if let Some(p) = current.displace(&(dx, dy)) {
				if board.get(&p) == Some(CellState::NeverOccupied) {
					state_counter += 1;

					let degree = board.get_degree(&p, piece);
					if degree < min_degree {
						min_degree = degree;
						next = Some(p);
					}
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
			state_counter,
		);

		match result {
			PartialComputation::Failed => {}
			PartialComputation::Successful {
				solution: mut working_moves,
			} => {
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

	PartialComputation::from(moves)
}

fn brute_recursive_tour_repeatless<P: ChessPiece + 'static>(
	piece: &P,
	options: BoardOptions,
	start: ChessPoint,
) -> Computation {
	let all_available_points = options.get_available_points();
	let num_moves_required = all_available_points.len() as u8 - 1;

	let mut state_counter = 0_u128;

	let board = Board::from_options(options);
	try_move_recursive(num_moves_required, piece, board, start, &mut state_counter)
		.map(|moves| {
			let mut moves = moves.into_iter().rev().collect::<Vec<Move>>();
			moves.push(Move::new(start, start));
			moves.into()
		})
		.add_state_count(state_counter)
}

use cache::{add_solution_to_cache, try_get_cached_solution};

mod cache {
	use super::*;
	use lru::LruCache;
	use once_cell::sync::Lazy;
	use std::num::NonZeroUsize;
	use std::{any::TypeId, collections::HashMap, sync::Mutex};

	static CYCLE_CACHE: Lazy<Mutex<HashMap<TypeId, LruCache<Key, Solution>>>> =
		Lazy::new(|| Mutex::new(HashMap::new()));

	fn new() -> LruCache<Key, Solution> {
		LruCache::new(NonZeroUsize::new(10_000).unwrap())
	}

	type Key = Options;
	type Solution = Computation;

	pub fn try_get_cached_solution<P: ChessPiece + 'static>(options: &Key) -> Option<Solution> {
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

	pub fn add_solution_to_cache<P: ChessPiece + 'static>(options: Key, moves: Solution) {
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

		debug!("Putting something in the cache");
		cache.put(options, moves);
	}
}
