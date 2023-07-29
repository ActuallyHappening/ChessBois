use derive_more::{Deref, DerefMut, From, Into};

use crate::solver::{Move, Moves};

use super::cells::visualization::VizColour;

/// Wrapper around [Vec<(Move, VizColour)>] with some extra functionality
#[derive(Clone, From, Into, Deref, DerefMut)]
pub struct ColouredMoves(Vec<(Move, VizColour)>);

impl From<ColouredMoves> for Moves {
	fn from(value: ColouredMoves) -> Self {
		value.0.into_iter().map(|(m, _)| m).collect()
	}
}

impl ColouredMoves {
	pub fn into_moves(self) -> Moves {
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
