use crate::{
	board::Hotkeyable,
	solver::{pieces::ChessPiece, *},
};
use bevy_egui_controls::ControlPanel;
use strum::{Display, EnumIter, IntoStaticStr};

mod hamiltonian;
use hamiltonian::hamiltonian_tour_repeatless;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Computation {
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

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct OwnedComputeInput {
	pub alg: Algorithm,
	pub safety_cap: u128,
	pub start: ChessPoint,
	pub board_options: BoardOptions,
	pub piece: ChessPiece,
}

#[derive(Clone)]
pub enum PartialComputation {
	Successful { solution: Moves },
	Failed,

	GivenUp,
}

mod parital_computation {
	use super::*;

	impl PartialComputation {
		pub fn add_state_count(self, count: u128) -> Computation {
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

		pub fn map(self, f: impl FnOnce(Moves) -> Moves) -> Self {
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
}

#[derive(
	Copy,
	Debug,
	Clone,
	Default,
	PartialEq,
	Eq,
	EnumIter,
	IntoStaticStr,
	Hash,
	PartialOrd,
	Ord,
	Display,
	ControlPanel,
	Reflect,
	FromReflect,
)]
pub enum Algorithm {
	#[strum(serialize = "Brute Force [f]")]
	#[default]
	BruteForceWarnsford,

	#[strum(serialize = "Warnsdorf (incomplete) [w]")]
	WarnsdorfBacktrack,

	#[strum(serialize = "Hamiltonian Cycle [c]")]
	HamiltonianCycle,

	#[strum(serialize = "Hamiltonian Cycle Brute Force [h]")]
	HamiltonianBruteForce,
}

impl From<Algorithm> for KeyCode {
	fn from(value: Algorithm) -> Self {
		match value {
			Algorithm::BruteForceWarnsford => KeyCode::F,
			Algorithm::WarnsdorfBacktrack => KeyCode::W,
			Algorithm::HamiltonianCycle => KeyCode::C,
			Algorithm::HamiltonianBruteForce => KeyCode::H,
		}
	}
}
impl Hotkeyable for Algorithm {}

impl Algorithm {
	pub fn get_description(&self) -> &'static str {
		match self {
			Algorithm::WarnsdorfBacktrack => "INCOMPLETE open knights tour. Takes into account targets and recommended moves.
This algorithm fully implemented Warnsdorf's rule, in that it tries all equal possibilities by backtracking. \
			As such it is not complete, it won't find a solution to every board (but never finds a false solution). \
			This algorithm works best with no targets.
			",
			Algorithm::BruteForceWarnsford => "COMPLETE open knights tour. Takes into account targets and recommended moves.
This algorithm is a Warnsdorf-biased brute force, which checks every possible path a knight can take (without repeating squares). \
			It contains no heuristics for targets, and will finish after the first valid path is found. \
			This algorithm works best with small boards and with a high saftey-states cap (preferrably not on web).
			",
			Algorithm::HamiltonianCycle => "UNTESTED closed knights tour, IGNORES targets and recommended moves.
This algorithm tries to find a hamiltonian cycle using a copy-pasted algorithm from the internet. \
			It appears to work but I haven't audited the code. It will stop if it has surpassed the saftey states cap. \
			This algorithm is not recommended for use, but is added because I am lazy.
			",
			Algorithm::HamiltonianBruteForce => "COMPLETE CLOSED knights tour. Takes into account recommended moves, targets don't make sense so are ignored. \
This algorithm uses brute force and will check every possible knight path, with biasing towards Warnsdorf's rule and complete backtracking. \
See the Brute Force algorithm's description for more.
			"
		}
	}
}

impl Algorithm {
	pub fn tour_computation(&self, input: OwnedComputeInput) -> Computation {
		match self {
			// Algorithm::WarnsdorfUnreliable => warnsdorf_tour_repeatless(piece, options, start),
			Algorithm::WarnsdorfBacktrack => brute_recursive_repeatless(
				&input.piece,
				input.board_options,
				input.start,
				TourType::WeakOpen,
				input.safety_cap,
			),
			Algorithm::BruteForceWarnsford => brute_recursive_repeatless(
				&input.piece,
				input.board_options,
				input.start,
				TourType::BruteForceOpen,
				input.safety_cap,
			),
			Algorithm::HamiltonianBruteForce => brute_recursive_repeatless(
				&input.piece,
				input.board_options,
				input.start,
				TourType::BruteForceClosed,
				input.safety_cap,
			),
			// Algorithm::HamiltonianPath => hamiltonian_tour_repeatless(piece, options, start, false),
			Algorithm::HamiltonianCycle => hamiltonian_tour_repeatless(
				&input.piece,
				input.board_options,
				input.start,
				input.safety_cap,
				true,
			),
		}
	}

	/// Actually compute, with caching
	pub fn tour_computation_cached(input: OwnedComputeInput) -> Option<Computation> {
		if !input
			.board_options
			.get_available_points()
			.contains(&input.start)
		{
			error!("Trying to solve a board starting on a square that is not available!");
			return None;
		}
		if let Some(cached_comp) = try_get_cached_solution(&input) {
			debug!("Solution cache hit!");

			if let Computation::GivenUp { explored_states } = cached_comp {
				if explored_states != input.safety_cap {
					// must recompute
					trace!(
						"Cache hit but recomputing because {} != {} (old != new)",
						explored_states,
						input.safety_cap
					);
					let comp = input.alg.tour_computation(input.clone());
					add_solution_to_cache(input, comp.clone());
				} else {
					trace!(
						"Cache hit on GivenUp and same states limit ({})",
						explored_states
					);
				}
			}

			Some(cached_comp)
		} else {
			debug!("Cache miss");
			let comp = input.alg.tour_computation(input.clone());
			add_solution_to_cache(input, comp.clone());
			Some(comp)
		}
	}
}

use std::collections::BTreeMap;

/// Actual state of cell
#[derive(Debug, Clone, PartialEq, Hash, Eq, EnumIs)]
enum CellState {
	NeverOccupied { target_allows_finish_here: bool },
	PreviouslyOccupied,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Board<'a> {
	cell_states: BTreeMap<ChessPoint, CellState>,
	recommended_moves: &'a Moves,
}

impl<'a> Board<'a> {
	fn get(&self, p: &ChessPoint) -> Option<CellState> {
		self.cell_states.get(p).cloned()
	}
	fn set(&mut self, p: ChessPoint, state: CellState) {
		self.cell_states.insert(p, state);
	}

	fn from_options(options: &'a BoardOptions) -> Self {
		Board {
			recommended_moves: options.recommended_moves(),
			cell_states: options
				.get_available_points()
				.into_iter()
				.map(|p| {
					(
						p,
						CellState::NeverOccupied {
							target_allows_finish_here: options.get(&p).unwrap().unwrap_available(),
						},
					)
				})
				.collect(),
		}
	}

	fn get_unrecommended_moves_from(
		&self,
		p: &ChessPoint,
		piece: &ChessPiece,
		allow_start_location: &ChessPoint,
	) -> std::collections::HashSet<ChessPoint> {
		let mut moves = std::collections::HashSet::new();
		for &(dx, dy) in piece.relative_moves() {
			if let Some(p) = p.displace(&(dx, dy)) {
				if self.get(&p).map(|s| s.is_never_occupied()) == Some(true) {
					moves.insert(p);
				} else if p == *allow_start_location {
					moves.insert(p);
				}
			}
		}
		moves
	}

	/// Also takes into account recommended moves
	fn get_available_moves_from(
		&self,
		p: &ChessPoint,
		piece: &ChessPiece,
		allow_start_location: &ChessPoint,
	) -> Vec<ChessPoint> {
		let mut moves = self.get_unrecommended_moves_from(p, piece, allow_start_location);
		// take into account recommended moves
		if !self.recommended_moves.is_empty() {
			// relevant moves from start
			let relevant_recommendations: std::collections::HashSet<ChessPoint> = self
				.recommended_moves
				.iter()
				.filter_map(|recommended_move| {
					if recommended_move.from == *p {
						Some(recommended_move.to)
					} else if recommended_move.to == *p {
						Some(recommended_move.from)
					} else {
						None
					}
				})
				.collect();
			// if any valid moves are contained in the recommendations, we must prioritise them
			if relevant_recommendations.len() != 0
				&& moves.intersection(&relevant_recommendations).count() > 0
			{
				debug!(message = "Moves before", ?moves);
				moves = moves
					.into_iter()
					.filter(|p| relevant_recommendations.contains(p))
					.collect();
				debug!(message = "Moves after", ?moves);
			}
		}

		Vec::from_iter(moves)
	}
	fn get_degree(&self, p: &ChessPoint, piece: &ChessPiece) -> u16 {
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

/// Recursively solves a knights tour
fn try_move_recursive(
	tour_type: TourType,
	num_moves_required: u16,
	piece: &ChessPiece,
	attempting_board: Board,
	current_pos: ChessPoint,
	starting_position: ChessPoint,
	state_counter: &mut u128,
	state_cap: u128,
) -> PartialComputation {
	*state_counter += 1;
	if *state_counter >= state_cap {
		// base case to avoid excessive computation
		return PartialComputation::GivenUp;
	}

	if num_moves_required == 0 {
		// base case
		if let Some(state) = attempting_board.get(&current_pos) {
			match (tour_type, state) {
				(TourType::WeakOpen | TourType::BruteForceOpen, CellState::PreviouslyOccupied) => {
					panic!("What, trying to end on point already moved to?")
				}
				// If you can finish on this square.
				// If a target is present, this may be false.
				// this check rejects solutions that don't end on a target.
				(
					TourType::WeakOpen | TourType::BruteForceOpen,
					CellState::NeverOccupied {
						target_allows_finish_here: true,
					},
				) => {
					return PartialComputation::Successful {
						solution: vec![].into(),
					};
				}
				(
					TourType::WeakOpen | TourType::BruteForceOpen,
					CellState::NeverOccupied {
						target_allows_finish_here: false,
					},
				) => {
					return PartialComputation::Failed;
				}
				(TourType::BruteForceClosed, CellState::NeverOccupied { .. }) => {
					return PartialComputation::Failed; // For brute force closed we must end at the end
				}
				(TourType::BruteForceClosed, CellState::PreviouslyOccupied) => {
					if current_pos == starting_position {
						return PartialComputation::Successful {
							solution: vec![].into(),
						};
					} else {
						panic!("The only time we should end on a previously occupied square is when we are back at the start.");
					}
				}
			}
		}
	}

	let mut available_moves =
		attempting_board.get_available_moves_from(&current_pos, piece, &starting_position);
	if available_moves.is_empty() {
		// stuck, no where to move
		return PartialComputation::Failed;
	}

	// sort by degree
	// this implicitely applies Warnsdorf algorithm
	available_moves.sort_by_cached_key(|p| attempting_board.get_degree(p, piece));

	match tour_type {
		TourType::WeakOpen => {
			// IMPORTANT: Only considers moves with the lowest degree. To make brute force, remove this
			let lowest_degree = attempting_board.get_degree(&available_moves[0], piece);
			available_moves.retain(|p| attempting_board.get_degree(p, piece) == lowest_degree);
		}
		TourType::BruteForceOpen | TourType::BruteForceClosed => {}
	}

	let mut moves = None;

	for potential_next_move in available_moves {
		let mut board_with_potential_move = attempting_board.clone();

		// imagine making the move
		board_with_potential_move.set(current_pos, CellState::PreviouslyOccupied);

		// now imagine the future of making the move (recursion)
		let result = try_move_recursive(
			tour_type.clone(),
			num_moves_required - 1,
			piece,
			board_with_potential_move,
			potential_next_move,
			starting_position,
			state_counter,
			state_cap,
		);

		match result {
			PartialComputation::Failed => { /* Continue looping, try to find a non-failed solution */ }
			PartialComputation::Successful {
				solution: mut working_moves,
			} => {
				// initially, working_moves will be empty
				// first iteration must add move from current_pos to potential_next_move
				// this repeats
				working_moves.push(Move::new(current_pos, potential_next_move));

				// found a solution, set to moves, stop looping and return success!
				moves = Some(working_moves);
				break;
			}
			PartialComputation::GivenUp => {
				// If a child recursive call has reached the call stack limit, give up as well
				return PartialComputation::GivenUp;
			}
		};
	}

	PartialComputation::from(moves)
}

#[derive(Clone, Copy)]
enum TourType {
	/// Does not always find solution but is significantly faster
	/// Used by [Algorithm::WarnsdorfBacktrack]
	WeakOpen,
	/// Always finds solution but is significantly slower
	BruteForceOpen,
	/// [Algorithm::HamiltonianBruteForce]
	BruteForceClosed,
}

fn brute_recursive_repeatless(
	piece: &ChessPiece,
	options: BoardOptions,
	start: ChessPoint,
	tour_type: TourType,
	safety_cap: u128,
) -> Computation {
	let all_available_points = options.get_available_points();
	let num_moves_required = all_available_points.len() as u16 - 1;

	let mut state_counter = 0_u128;

	let board = Board::from_options(&options);
	try_move_recursive(
		tour_type,
		num_moves_required,
		piece,
		board,
		start,
		start,
		&mut state_counter,
		safety_cap,
	)
	.map(|moves| {
		let mut moves = moves.into_iter().rev().collect::<Vec<Move>>();
		let end = &moves.last().unwrap().to;
		moves.push(Move::new(*end, *end));
		moves.into()
	})
	.add_state_count(state_counter)
}

use cache::add_solution_to_cache;
pub use cache::try_get_cached_solution;

mod cache {
	use super::*;
	use lru::LruCache;
	use once_cell::sync::Lazy;
	use std::num::NonZeroUsize;
	use std::sync::Mutex;

	static COMPUTE_CACHE: Lazy<Mutex<LruCache<Key, Solution>>> = Lazy::new(|| Mutex::new(new()));

	fn new() -> LruCache<Key, Solution> {
		LruCache::new(NonZeroUsize::new(10_000).unwrap())
	}

	type Key = OwnedComputeInput;
	type Solution = Computation;

	pub fn try_get_cached_solution(options: &Key) -> Option<Solution> {
		let mut cache = COMPUTE_CACHE.lock().unwrap();

		cache.get(options).cloned()
	}

	pub fn add_solution_to_cache(options: Key, moves: Solution) {
		let mut cache = COMPUTE_CACHE.lock().unwrap();

		debug!("Putting something in the algs cache");
		cache.put(options, moves);
	}
}
