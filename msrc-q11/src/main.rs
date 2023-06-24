use msrc_q11::*;
use tracing::info;

fn main() {
	msrc_q11::init_debug_tools();

	let start: ChessPoint = (1, 1).into();
	let (width, height) = (8, 8);
	info!("Board size: {}x{}", width, height);
	let board = BoardOptions::new(8, 8);
	let knight = StandardKnight{};

	let moves = knight.try_piece_tour_warnsdorf(board.clone(), start).expect("tour to be possible");

	info!("Board: \n{}", board);
	info!("Moves: {}", moves);
}

// fn main() {
// 	msrc_q11::init_debug_tools();

// 	let (x, y) = (8, 2);
// 	info!("Board size: {}", SIZE);
// 	info!("Starting position: ({}, {})", x, y);
// 	match knights_tour(x, y) {
// 		Some(b) => info!("Board: \n{}", b.0),
// 		None => info!("Fail!"),
// 	}
// }
