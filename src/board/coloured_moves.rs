use derive_more::{Deref, DerefMut, From, Into};
use serde::{Serialize, Deserialize};

use crate::{solver::{Move, Moves}, ChessPoint};

use super::squares::visualization::VizColour;

/// Wrapper around [Vec<(Move, VizColour)>] with some extra functionality
#[derive(Clone, Default, From, Into, Deref, DerefMut, Serialize, Deserialize, Debug, PartialEq)]
pub struct ColouredMoves(Vec<(Move, VizColour)>);

impl From<ColouredMoves> for Moves {
	fn from(value: ColouredMoves) -> Self {
		value.0.into_iter().map(|(m, _)| m).collect()
	}
}

impl From<&ColouredMoves> for Moves {
	fn from(value: &ColouredMoves) -> Self {
		value.0.iter().map(|(m, _)| *m).collect()
	}
}

impl ColouredMoves {
	pub fn into_moves(self) -> Moves {
		self.into()
	}

	pub fn moves(&self) -> Moves {
		self.into()
	}
}

impl Moves {
	pub fn using_colour(self, colour: VizColour) -> ColouredMoves {
		self.into_iter().map(|m| (m, colour)).collect()
	}
}

impl FromIterator<(Move, VizColour)> for ColouredMoves {
	fn from_iter<T: IntoIterator<Item = (Move, VizColour)>>(iter: T) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl ColouredMoves {
	pub fn undo(&mut self) -> &mut Self {
		self.0.pop();
		self
	}

	pub fn manual_add_move(&mut self, point: ChessPoint, col: VizColour) -> &mut Self {
		if self.iter().len() == 0 {
			self.push((Move::new(point, point), col));
		} else {
			let last = *self.last().unwrap();
			self.push((Move::new(last.0.to, point), col));
		}
		self
	}
}