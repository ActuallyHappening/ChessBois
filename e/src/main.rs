use bevy_solver::solver::{algs::{Algorithm, Computation}, pieces::StandardKnight, BoardOptions};

fn main() {
	println!("Hello, world!");

	let piece = StandardKnight {};
	let mut board = BoardOptions::new(5, 5);
	board.rm((1, 1));

	match Algorithm::HamiltonianPath.tour_computation(&piece, board, (2, 1).into()) {
		Computation::Successful { solution, explored_states } => {
			println!("Solution: {:?}", solution);
		}
		Computation::Failed { total_states } => {
			println!("Failed after {} states", total_states);
		}
	}
}
