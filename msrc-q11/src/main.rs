use tracing::info;
use msrc_q11::*;

fn main() {
	msrc_q11::init_debug_tools();

	let (x, y) = (3, 1);
	info!("Board size: {}", SIZE);
	info!("Starting position: ({}, {})", x, y);
	match knights_tour(x, y) {
		Some(b) => info!("Board: \n{}\nMoves: {:?}", b.0, b.1),
		None => info!("Fail!"),
	}
}