use crate::{
	solver::{pieces::ChessPiece, *},
	ALG_STATES_CAP,
};
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

	GivenUp {
		explored_states: u128,
	},
}

#[derive(Clone)]
pub enum PartialComputation {
	Successful { solution: Moves },
	Failed,

	GivenUp,
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
			Self::GivenUp => Computation::GivenUp {
				explored_states: count,
			},
		}
	}

	fn map(self, f: impl FnOnce(Moves) -> Moves) -> Self {
		match self {
			Self::Successful { solution } => Self::Successful {
				solution: f(solution),
			},
			Self::Failed => Self::Failed,
			Self::GivenUp => Self::GivenUp,
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

#[derive(
	Copy, Debug, Clone, Default, PartialEq, Eq, EnumIter, IntoStaticStr, Hash, PartialOrd, Ord,
)]
pub enum Algorithm {
	#[default]
	#[strum(serialize = "Warnsdorf")]
	WarnsdorfBacktrack,

	// #[strum(serialize = "Warnsdorf (Unreliable)")]
	// WarnsdorfUnreliable,

	// #[strum(serialize = "Brute Force (SLOW)")]
	// HamiltonianPath,
	#[strum(serialize = "Brute Force (slow)")]
	BruteForceWarnsford,

	#[strum(serialize = "Hamiltonian Cycle")]
	HamiltonianCycle,
}

impl Algorithm {
	pub fn get_description(&self) -> &'static str {
		match self {
			// Algorithm::WarnsdorfUnreliable => "A standard knights tour.\
			// This algorithm is always guaranteed to terminate in finite time, however it sometimes misses solutions e.g. 8x8 board @ (5, 3).\
			// Warnsdorf's Rule is very easy to implement and is very popular because of its simplicity. The implementation used is sub-optimal, but should suffice.\
			// Also, in the case that Warnsdorf's Rule doesn't fully specify what to do, as in where you have to guess, this algorithm acts *deterministically* but\
			// *not correctly*. A fuller implementation would implement backtracking, which the Reliable Warnsdorf algorithm does.\
			// ", 
			Algorithm::WarnsdorfBacktrack => "A standard knights tour.\
			This algorithm applies Warnsdorf's Rule, which tells you to always move to the square with the fewest available moves. \
			This algorithm is a fuller implementation of Warnsdorf's Rule, including backtracking when Warnsdorf's Rule doesn't fully specify what to do.\
			HOWEVER, it does not check every possible state, so its completeness / correctness still depends on the completeness of the Warnsdorf algorithm!
			i.e., I am not certain this algorithm is ALWAYS correct!
			",
			// Algorithm::HamiltonianPath => "A standard knights tour.\
			// This algorithm attempts to find a hamiltonian path from the start to any end point with brute force.\
			// Therefore, if there is a knights tour it is guarenteed to find it, just not as fast as the Warnsdorf algorithm!\
			// ",
			Algorithm::HamiltonianCycle => "NOT a knights tour!\
			This algorithm tries to find a hamiltonian cycle, as in a path starting and ending at the same point.\
			This is significantly slower than other algorithms, but when found it provides solutions to knights tour for every start point.\
			I believe this is guarenteed to give the correct answer, although I have not tested it thoroughly.\
			",
			Algorithm::BruteForceWarnsford => "A standard knights tour.\
			This algorithm applies Warnsdorf's Rule, which tells you to always move to the square with the fewest available moves. \
			This algorithm is a fuller implementation of Warnsdorf's Rule, including backtracking when Warnsdorf's Rule doesn't fully specify what to do.\
			AND, it checks every possible state, so it is complete and GUARENTEED to find a solution if one exists.\
			",
		}
	}

	pub fn should_show_states(&self) -> bool {
		// matches!(self, Algorithm::WarnsdorfUnreliable | Algorithm::WarnsdorfBacktrack | Algorithm::HamiltonianCycle)
		true
	}
}

/// Represents information required to display cells + visual solutions
#[derive(derivative::Derivative, Debug, Clone, Eq, PartialOrd, Ord)]
#[derivative(PartialEq)]
pub struct Options {
	pub options: BoardOptions,

	pub selected_start: Option<ChessPoint>,
	pub selected_algorithm: Algorithm,

	#[derivative(PartialEq = "ignore")]
	// must be ignored by Hash
	pub requires_updating: bool,
}

impl std::hash::Hash for Options {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.options.hash(state);
		self.selected_start.hash(state);
		self.selected_algorithm.hash(state);
	}
}

impl Deref for Options {
	type Target = BoardOptions;

	fn deref(&self) -> &Self::Target {
		&self.options
	}
}
impl DerefMut for Options {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.options
	}
}

impl Options {
	pub fn with_start(&self, start: ChessPoint) -> Self {
		Self {
			selected_start: Some(start),
			..self.clone()
		}
	}

	/// Requires a re-render
	pub fn requires_updating(&mut self) -> &mut Self {
		self.requires_updating = true;
		self
	}
}

impl Algorithm {
	pub fn tour_computation<P: ChessPiece + 'static>(
		&self,
		piece: &P,
		options: BoardOptions,
		start: ChessPoint,
	) -> Computation {
		match self {
			// Algorithm::WarnsdorfUnreliable => warnsdorf_tour_repeatless(piece, options, start),
			Algorithm::WarnsdorfBacktrack => {
				brute_recursive_tour_repeatless(piece, options, start, TourType::Weak)
			}
			Algorithm::BruteForceWarnsford => {
				brute_recursive_tour_repeatless(piece, options, start, TourType::BruteForce)
			}
			// Algorithm::HamiltonianPath => hamiltonian_tour_repeatless(piece, options, start, false),
			Algorithm::HamiltonianCycle => hamiltonian_tour_repeatless(piece, options, start, true),
		}
	}

	/// Returns None if options.selected_start is None
	pub fn tour_computation_cached<P: ChessPiece + 'static>(
		&self,
		piece: &P,
		options: Options,
	) -> Option<Computation> {
		if let Some(start) = options.selected_start {
			if !options.options.get_available_points().contains(&start) {
				return None;
			}
			Some(if let Some(comp) = try_get_cached_solution::<P>(&options) {
				debug!("Solution cache hit!");

				if let Computation::GivenUp { explored_states } = comp {
					if explored_states != *ALG_STATES_CAP.lock().unwrap() {
						// must recompute
						info!(
							"Cache hit and recomputing because {} != {}",
							explored_states,
							*ALG_STATES_CAP.lock().unwrap()
						);
						let comp = self.tour_computation(piece, options.options.clone(), start);
						add_solution_to_cache::<P>(options, comp.clone());
						return Some(comp);
					} else {
						info!(
							"Cache hit on GivenUp and same states limit ({})",
							explored_states
						);
					}
				}

				comp
			} else {
				debug!("Cache miss");
				let comp = self.tour_computation(piece, options.options.clone(), start);
				add_solution_to_cache::<P>(options, comp.clone());
				comp
			})
		} else {
			None
		}
	}
}

use std::collections::BTreeMap;

/// Actual state of cell
#[derive(Debug, Clone, PartialEq, Hash, Eq, EnumIs)]
enum CellState {
	NeverOccupied { can_finish_on: bool },
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
				.map(|p| {
					(
						p,
						CellState::NeverOccupied {
							can_finish_on: options.get(&p).unwrap().unwrap_available(),
						},
					)
				})
				.collect(),
		}
	}

	fn get_available_moves_from(&self, p: &ChessPoint, piece: &impl ChessPiece) -> Vec<ChessPoint> {
		let mut moves = Vec::new();
		for &(dx, dy) in piece.relative_moves() {
			if let Some(p) = p.displace(&(dx, dy)) {
				if self.get(&p).map(|s| s.is_never_occupied()) == Some(true) {
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
				if self.get(&p).map(|s| s.is_never_occupied()) == Some(true) {
					degree += 1;
				}
			}
		}
		degree
	}
}

/*  #region old alg */
// fn warnsdorf_tour_repeatless<P: ChessPiece>(
// 	piece: &P,
// 	options: BoardOptions,
// 	start: ChessPoint,
// ) -> Computation {
// 	let mut board = Board::from_options(options);
// 	let mut moves = Vec::new();
// 	let mut current = start;

// 	let num_available_cells = board.cell_states.len();
// 	let mut state_counter = 0_u128;

// 	for _ in 1..num_available_cells {
// 		if !board.cell_states.contains_key(&current) {
// 			return Computation::Failed {
// 				total_states: board.cell_states.len() as u128,
// 			};
// 		}

// 		board.set(current, CellState::PreviouslyOccupied);

// 		let mut next = None;
// 		let mut min_degree = u16::MAX;
// 		for &(dx, dy) in piece.relative_moves() {
// 			if let Some(p) = current.displace(&(dx, dy)) {
// 				if board.get(&p) == Some(CellState::NeverOccupied) {
// 					state_counter += 1;

// 					let degree = board.get_degree(&p, piece);
// 					if degree < min_degree {
// 						min_degree = degree;
// 						next = Some(p);
// 					}
// 				}
// 			}
// 		}

// 		if let Some(next) = next {
// 			moves.push(Move::new(current, next));
// 			current = next;
// 		} else {
// 			return Computation::Failed {
// 				total_states: state_counter,
// 			};
// 		}
// 	}

// 	Computation::Successful {
// 		solution: moves.into(),
// 		explored_states: state_counter,
// 	}
// }
/* #endregion */

fn try_move_recursive(
	tour_type: TourType,
	num_moves_required: u16,
	piece: &impl ChessPiece,
	attempting_board: Board,
	current_pos: ChessPoint,
	state_counter: &mut u128,
) -> PartialComputation {
	*state_counter += 1;
	if *state_counter >= *ALG_STATES_CAP.lock().unwrap() {
		// base case to avoid excessive computation
		return PartialComputation::GivenUp;
	}

	if num_moves_required == 0 {
		// base case
		if let Some(state) = attempting_board.get(&current_pos) {
			match state {
				CellState::NeverOccupied {
					can_finish_on: true,
				} => {
					return PartialComputation::Successful {
						solution: vec![].into(),
					};
				}
				CellState::NeverOccupied {
					can_finish_on: false,
				} => {
					return PartialComputation::Failed;
				}
				CellState::PreviouslyOccupied => panic!("What, trying to end on point already moved to?"),
			}
		}
	}

	let mut available_moves = attempting_board.get_available_moves_from(&current_pos, piece);
	if available_moves.is_empty() {
		// println!("No moves available");
		// return None;
		return PartialComputation::Failed;
	}
	// sort by degree
	available_moves.sort_by_cached_key(|p| attempting_board.get_degree(p, piece));

	match tour_type {
		TourType::Weak => {
			// IMPORTANT: Only considers moves with the lowest degree. To make brute force, remove this
			let lowest_degree = attempting_board.get_degree(&available_moves[0], piece);
			available_moves.retain(|p| attempting_board.get_degree(p, piece) == lowest_degree);
		}
		TourType::BruteForce => {}
	}

	let mut moves = None;

	for potential_next_move in available_moves {
		let mut board_with_potential_move = attempting_board.clone();

		board_with_potential_move.set(current_pos, CellState::PreviouslyOccupied);

		let result = try_move_recursive(
			tour_type.clone(),
			num_moves_required - 1,
			piece,
			board_with_potential_move,
			potential_next_move,
			state_counter,
		);

		match result {
			PartialComputation::Failed => { /* Continue looping */ }
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
			PartialComputation::GivenUp => {
				return PartialComputation::GivenUp;
			}
		};
	}

	PartialComputation::from(moves)
}

#[derive(Clone)]
enum TourType {
	/// Does not always find solution but is significantly faster
	Weak,
	/// Always finds solution but is significantly slower
	BruteForce,
}

fn brute_recursive_tour_repeatless<P: ChessPiece + 'static>(
	piece: &P,
	options: BoardOptions,
	start: ChessPoint,
	tour_type: TourType,
) -> Computation {
	let all_available_points = options.get_available_points();
	let num_moves_required = all_available_points.len() as u16 - 1;

	let mut state_counter = 0_u128;

	let board = Board::from_options(options);
	try_move_recursive(
		tour_type,
		num_moves_required,
		piece,
		board,
		start,
		&mut state_counter,
	)
	.map(|moves| {
		let mut moves = moves.into_iter().rev().collect::<Vec<Move>>();
		let end = &moves.last().unwrap().to;
		moves.push(Move::new(*end, *end));
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
