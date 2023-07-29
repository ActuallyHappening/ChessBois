use bevy::prelude::Resource;
use bevy_egui_controls::ControlPanel;
use strum::{EnumIs, EnumIter};

use crate::{
	board::SharedState,
	solver::{pieces::ChessPiece, Moves},
	ChessPoint,
};

#[derive(
	Resource,
	strum::Display,
	EnumIs,
	Default,
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	EnumIter,
	ControlPanel,
)]
pub enum ManualFreedom {
	#[strum(serialize = "Only valid")]
	ValidOnly,

	#[strum(serialize = "Any possible")]
	AnyPossible,

	#[strum(serialize = "Completely free")]
	#[default]
	Free,
}

impl ManualFreedom {
	pub fn get_description(&self) -> &'static str {
		match self {
			ManualFreedom::Free => "Chose any move that is on the board and not disabled. The most free option available.",
			ManualFreedom::AnyPossible => "Chose only moves that are valid knight moves. Can still jump onto squares multiple times",
			ManualFreedom::ValidOnly => "Chose only moves that are valid knight moves and have not been visited yet. The most restrictive option available."
		}
	}

	pub fn check_move(&self, state: &SharedState, next: ChessPoint) -> (bool, MoveWarning) {
		let warning = state.check_move(next);
		let ok = match self {
			ManualFreedom::Free => !matches!(warning, MoveWarning::NonExistant),
			ManualFreedom::AnyPossible => !matches!(warning, MoveWarning::NonExistant | MoveWarning::NotValid),
			ManualFreedom::ValidOnly => matches!(warning, MoveWarning::OK | MoveWarning::NoMoves),
		};
		(ok, warning)
	}
}

#[derive(PartialOrd, PartialEq)]
pub enum MoveWarning {
	// (potential) ERRs
	/// Not on board
	NonExistant,

	/// Not a valid knight/piece move
	NotValid,

	/// Same as *any* previous move
	AlreadyDone,

	/// Same as *last* move
	Repeated,

	// OK
	/// No moves to judge agains
	NoMoves,

	OK,
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
