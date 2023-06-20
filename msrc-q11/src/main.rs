use tracing::info;
use msrc_q11::*;

fn main() {
	msrc_q11::init_debug_tools();

	let (x, y) = (8, 2);
	let (width, height) = (8, 8);
	info!("Board size: {}x{}", width, height);
	let piece = StandardKnight {};

	let mut board = Board::new(width, height);
	
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