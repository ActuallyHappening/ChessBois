use bevy_egui::egui::{self, RichText, Ui};
use bevy_egui_controls::ControlPanel;
use serde::{Deserialize, Serialize};
use strum::{EnumIs, EnumIter};

use crate::{board::StateInvalidated, ChessPoint};

/// Holds info on valid moves
#[derive(Hash, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ChessPiece {
	valid_moves: Vec<(i16, i16)>,
}

impl ChessPiece {
	pub fn new(moves: Vec<(i16, i16)>) -> Self {
		Self { valid_moves: moves }
	}

	pub fn relative_moves(&self) -> &Vec<(i16, i16)> {
		&self.valid_moves
	}

	pub fn is_valid_move(&self, from: ChessPoint, to: ChessPoint) -> bool {
		// checks self.relative_moves
		let dx = to.column as i16 - from.column as i16;
		let dy = to.row as i16 - from.row as i16;
		self.relative_moves().contains(&(dx, dy))
	}
}

/// Collection of standard sets of moves
#[derive(Default, Clone, Copy, PartialEq, Serialize, Deserialize, Debug, EnumIs, EnumIter, strum::Display)]
pub enum StandardPieces {
	/// Same as [Pieces::ABKnight(1, 2)]
	#[strum(serialize = "Standard Knight")]
	#[default]
	StandardKnight,

	#[strum(serialize = "AB Knight")]
	ABKnight(i8, i8),
}

impl Default for ChessPiece {
	fn default() -> Self {
		StandardPieces::StandardKnight.into()
	}
}

impl From<StandardPieces> for Vec<(i16, i16)> {
	fn from(value: StandardPieces) -> Self {
		match value {
			StandardPieces::StandardKnight => vec![
				(2, 1),
				(1, 2),
				(-1, 2),
				(-2, 1),
				(-2, -1),
				(-1, -2),
				(1, -2),
				(2, -1),
			],
			StandardPieces::ABKnight(a, b) => {
				let a = a as i16;
				let b = b as i16;
				vec![
					(a, b),
					(-a, b),
					(a, -b),
					(-a, -b),
					(b, a),
					(-b, a),
					(b, -a),
					(-b, -a),
				]
			}
		}
	}
}

impl From<StandardPieces> for ChessPiece {
	fn from(value: StandardPieces) -> Self {
		ChessPiece::new(value.into())
	}
}

impl StandardPieces {
	const MAX_AB: i8 = 5;
	const MIN_AB: i8 = 0;

	pub fn ui(&mut self, ui: &mut Ui) -> StateInvalidated {
		let mut invalidate = StateInvalidated::Valid;

		if ui
			.button(RichText::new("Standard Knight").color({
				if self.is_standard_knight() {
					egui::Color32::GREEN
				} else {
					egui::Color32::GRAY
				}
			}))
			.clicked()
		{
			*self = StandardPieces::StandardKnight;
			invalidate = StateInvalidated::Invalidated;
		}
		if ui
			.button(RichText::new("AB Knight").color({
				if self.is_ab_knight() {
					egui::Color32::GREEN
				} else {
					egui::Color32::GRAY
				}
			}))
			.clicked()
		{
			*self = StandardPieces::ABKnight(2, 1);
		}

		if let StandardPieces::ABKnight(a, b) = self {
			ui.add(egui::Slider::from_get_set(
				(Self::MIN_AB as f64)..=(Self::MAX_AB as f64),
				|val| {
					if let Some(val) = val {
						*a = val as i8;
						invalidate = StateInvalidated::Invalidated;
					}
					*a as f64
				},
			).text("A"));

			ui.add(egui::Slider::from_get_set(
				(Self::MIN_AB as f64)..=(Self::MAX_AB as f64),
				|val| {
					if let Some(val) = val {
						*b = val as i8;
						invalidate = StateInvalidated::Invalidated;
					}
					*b as f64
				},
			).text("B"));

			ui.label("An 'AB Knight' refers to a piece that must move A squares in one direction, and B squares in any perpendicular direction.
This makes the Standard knight equivalent to an AB Knight with A=1 and B=2, or A=2 and B=1.
And, a bishop is equivalen to a [1, 1] AB knight (diagonals only).
Trying playing with a [1, 0] knight!");
		}

		invalidate
	}
}
