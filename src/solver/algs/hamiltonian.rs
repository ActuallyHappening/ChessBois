use crate::{solver::{pieces::ChessPiece, BoardOptions}, ChessPoint};

use super::Computation;



pub fn hamiltonian_tour_repeatless<P: ChessPiece + 'static>(
	piece: &P,
	options: BoardOptions,
	start: ChessPoint,
) -> Computation {

	unimplemented!()
}