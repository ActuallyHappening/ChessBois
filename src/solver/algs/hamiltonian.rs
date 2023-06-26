use std::{collections::{HashMap, HashSet}, ops::Add};
use tracing::info;

use crate::{
	solver::{pieces::ChessPiece, BoardOptions},
	ChessPoint,
};

use super::Computation;

type Key = u32;
type Graph = HashMap<Key, HashSet<Key>>;
type Path = Vec<Key>;

fn find_hamiltonian_path(end: u32, P: &Path, g: &Graph) -> Option<Path> {
	let v = P.last().unwrap();
	if P.len() == g.len() && g.get(v).unwrap().contains(&end) {
		let mut C = P.clone();
		C.push(end);
		Some(C)
	} else {
		for w in g.get(v).unwrap() {
			if P.contains(w) {
				continue;
			}
			let mut Q = P.clone();
			Q.push(*w);
			let H = find_hamiltonian_path(end, &Q, g);
			if H.is_some() {
				return H;
			}
		}
		None
	}
}

pub fn hamiltonian_tour_repeatless<P: ChessPiece + 'static>(
	piece: &P,
	options: BoardOptions,
	start: ChessPoint,
) -> Computation {

	impl ChessPoint {
		pub fn hash(&self) -> Key {
			(self.column as Key) << 16 | (self.row as Key)
		}
	}

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

	info!("Graph for {len} points: \n{:?}\nPoint mappings: {:?}", graph, available_mapped_points, len = available_points.len());

	let start = *available_mapped_points.get(&start).unwrap();
	if let Some(path) = find_hamiltonian_path(start, &vec![start], &graph) {
		assert_eq!(available_points.len(), path.len() - 1);

		unimplemented!()
	} else {
		Computation::Failed { total_states: 0 }
	}
}
