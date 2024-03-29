use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use tracing::{debug, trace};

use crate::{
	solver::{pieces::ChessPiece, BoardOptions, Move, Moves},
	ChessPoint,
};

use super::Computation;

type Key = u32;
type Graph = HashMap<Key, HashSet<Key>>;
type Path = Vec<Key>;

#[allow(non_snake_case)]
fn find_hamiltonian_path(
	end: u32,
	P: &Path,
	g: &Graph,
	state_counter: &mut u128,
	safety_cap: u128,
) -> Result<Option<Path>, ()> {
	*state_counter += 1;
	if *state_counter >= safety_cap {
		return Err(());
	}

	let v = P.last().unwrap();
	if P.len() == g.len() && g.get(v).unwrap().contains(&end) {
		let mut C = P.clone();
		C.push(end);
		Ok(Some(C))
	} else {
		for w in g.get(v).unwrap() {
			if P.contains(w) {
				continue;
			}
			let mut Q = P.clone();
			Q.push(*w);
			let H = find_hamiltonian_path(end, &Q, g, state_counter, safety_cap)?;
			if H.is_some() {
				return Ok(H);
			}
		}
		Ok(None)
	}
}

impl ChessPoint {
	pub fn hash(&self) -> Key {
		(self.column as Key) << 16 | (self.row as Key)
	}
	pub fn un_hash(key: Key) -> Self {
		Self {
			column: (key >> 16) as u16,
			row: (key & 0xFFFF) as u16,
		}
	}
}

/// cycle paramter: whether to find Hamiltonian Cycle or Path
pub fn hamiltonian_tour_repeatless(
	piece: &ChessPiece,
	options: BoardOptions,
	start: ChessPoint,
	safety_cap: u128,
	cycle: bool,
) -> Computation {
	assert!(options.get_available_points().contains(&start));

	let available_points = options.get_available_points();

	let mut available_mapped_points: HashMap<&ChessPoint, Key> = HashMap::new();
	for point in available_points.iter() {
		available_mapped_points.insert(point, point.hash());
	}

	let mut graph: Graph = HashMap::new();
	let valid_moves = piece.relative_moves();
	for point in available_points.iter() {
		let mut edges: HashSet<Key> = HashSet::new();
		for d in valid_moves.iter() {
			if let Some(point) = point.displace(d) {
				if available_mapped_points.contains_key(&point) {
					// if point is adjacent, exists, and is available, it is valid edge
					edges.insert(*available_mapped_points.get(&point).unwrap());
				}
			}
		}
		graph.insert(*available_mapped_points.get(point).unwrap(), edges);
	}

	trace!(
		"Graph for {len} points: \n{:?}\nPoint mappings: {:?}",
		graph,
		available_mapped_points,
		len = available_points.len()
	);

	let start = *available_mapped_points.get(&start).unwrap();
	let start_vec = vec![start];
	if !cycle {
		// show any path that works
		let mut state_counter: u128 = 0;
		for valid_end_point in available_mapped_points.values() {
			match find_hamiltonian_path(*valid_end_point, &start_vec, &graph, &mut state_counter, safety_cap) {
				Err(_) => return Computation::Failed { total_states: 0 },
				Ok(None) => continue,
				Ok(Some(mut path)) => {
					path.pop();

					let moves: Moves = path
						.into_iter()
						.map(ChessPoint::un_hash)
						.tuple_windows()
						.map(Move::from_tuple)
						.collect();

					return Computation::Successful {
						solution: moves,
						explored_states: state_counter,
					};
				}
			}
		}
		Computation::Failed { total_states: 0 }
	} else {
		let mut state_counter: u128 = 0;
		match find_hamiltonian_path(start, &start_vec, &graph, &mut state_counter, safety_cap) {
			Err(_) => Computation::GivenUp {
				explored_states: state_counter,
			},
			Ok(None) => Computation::Failed {
				total_states: state_counter,
			},
			Ok(Some(path)) => {
				// show only cycle
				assert_eq!(available_points.len(), path.len() - 1);

				debug!("Path found: {:?}", path);
				// path always ends back up at start

				let moves: Moves = path
					.into_iter()
					.map(ChessPoint::un_hash)
					.tuple_windows()
					.map(Move::from_tuple)
					.collect();

				Computation::Successful {
					solution: moves,
					explored_states: 0,
				}
			}
		}
	}
}
