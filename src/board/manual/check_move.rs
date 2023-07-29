use bevy_egui::egui::Color32;

use crate::{board::SharedState, ChessPoint, solver::Moves};

use super::ManualFreedom;

/// `<= MoveWarning::NoMoves` is always a valid move
#[derive(PartialOrd, PartialEq, strum::Display)]
pub enum MoveWarning {
	// smallest
	OK,

	/// No moves to judge agains
	#[strum(serialize = "No moves to judge against")]
	NoMoves,

	/// Same as *last* move
	#[strum(serialize = "Cell is the same as the last cell.")]
	Repeated,

	/// Same as *any* previous move
	#[strum(serialize = "Cell already passed through")]
	AlreadyDone,

	// (potential) ERRs
	/// Not on board
	#[strum(serialize = "Point does not exist on the board")]
	NonExistant,

	// largest
	/// Not a valid knight/piece move
	#[strum(serialize = "Not a valid knight/piece move")]
	NotValid,
}

impl ManualFreedom {
	pub fn check_move(&self, state: &SharedState, next: ChessPoint) -> (bool, MoveWarning) {
		let warning = state.check_move(next);
		let ok = match self {
			ManualFreedom::Free => !matches!(warning, MoveWarning::NonExistant),
			ManualFreedom::AnyPossible => warning < MoveWarning::NotValid,
			ManualFreedom::ValidOnly => warning <= MoveWarning::NoMoves,
		};
		(ok, warning)
	}
}

impl MoveWarning {
	pub fn ui(&self, ui: &mut bevy_egui::egui::Ui) {
		ui.colored_label(
			{
				if self <= &MoveWarning::NoMoves {
					Color32::GREEN
				} else if self <= &MoveWarning::NotValid {
					Color32::YELLOW
				} else {
					Color32::RED
				}
			},
			format!("{}", self),
		);
	}
}

impl SharedState {
	fn check_move(&self, next: ChessPoint) -> MoveWarning {
		// checks if non-existant
		if !self.board_options.validate_point(&next) {
			return MoveWarning::NonExistant;
		}
		match &self.moves {
			None => MoveWarning::NoMoves,
			Some(moves) => {
				let moves: Moves = moves.moves();
				if moves.is_empty() {
					MoveWarning::NoMoves
				} else if !self.piece.is_valid_move(moves.last().unwrap().to, next) {
					MoveWarning::NotValid
				} else if moves.get_all_passed_through_points().contains(&next) {
					MoveWarning::AlreadyDone
				} else if moves.len() > 1 && moves[moves.len() - 2].to == next {
					MoveWarning::Repeated
				} else {
					MoveWarning::OK
				}
			}
		}
	}
}
