use serde::{Serialize, Deserialize};
use super::*;


/// Represents move from one point to another
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize, FromReflect, Reflect)]
pub struct Move {
	pub from: ChessPoint,
	pub to: ChessPoint,
}

impl Move {
	pub fn new(from: ChessPoint, to: ChessPoint) -> Self {
		Self { from, to }
	}

	pub fn from_tuple((from, to): (ChessPoint, ChessPoint)) -> Self {
		Move::new(from, to)
	}

	pub fn new_checked(from: ChessPoint, to: ChessPoint, board: &BoardOptions) -> Option<Self> {
		if board.validate_point(&from) && board.validate_point(&to) {
			Some(Self { from, to })
		} else {
			None
		}
	}
}

/// Wrapper around `Vec<Move>` with some extra functionality
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Moves {
	moves: Vec<Move>,
}

impl Display for Moves {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for m in self.moves.iter() {
			writeln!(f, "{} -> {}", m.from, m.to)?;
		}
		Ok(())
	}
}

impl Display for Move {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} -> {}", self.from, self.to)
	}
}

impl From<Vec<Move>> for Moves {
	fn from(moves: Vec<Move>) -> Self {
		Self { moves }
	}
}

impl From<Moves> for Vec<Move> {
	fn from(value: Moves) -> Self {
		value.moves
	}
}

impl Deref for Moves {
	type Target = Vec<Move>;

	fn deref(&self) -> &Self::Target {
		&self.moves
	}
}

impl DerefMut for Moves {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.moves
	}
}

impl Moves {
	#[allow(dead_code)]
	pub(crate) fn new(moves: Vec<Move>) -> Self {
		Self { moves }
	}

	pub fn find_move_index(&self, m: &Move) -> Option<usize> {
		self.moves.iter().position(|x| x == m)
	}

	/// Assumes all moves are connected
	pub fn get_all_passed_through_points(&self) -> Vec<ChessPoint> {
		let mut ret: Vec<_> = self.moves.iter().map(|m| m.from).collect();
		if let Some(last) = self.moves.last() {
			ret.push(last.to);
		}
		ret
	}
}

impl IntoIterator for Moves {
	type Item = Move;
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.moves.into_iter()
	}
}

impl std::iter::FromIterator<Move> for Moves {
	fn from_iter<T: IntoIterator<Item = Move>>(iter: T) -> Self {
		Self {
			moves: iter.into_iter().collect(),
		}
	}
}