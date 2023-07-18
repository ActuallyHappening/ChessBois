use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::EnumIs;
use std::{
	fmt::{self, Display},
	ops::{Deref, DerefMut},
};

pub mod algs;
pub mod pieces;

// 1 indexed
#[derive(
	Copy,
	Component,
	Reflect,
	Hash,
	Clone,
	Debug,
	Eq,
	PartialEq,
	PartialOrd,
	Ord,
	Serialize,
	Deserialize,
)]
pub struct ChessPoint {
	// Between 1 and COLUMN_SIZE.
	/// Corresponds to x axis
	pub column: u16,

	// Between 1 and ROW_SIZE.
	/// Corresponds to y axis
	pub row: u16,
}

impl ChessPoint {
	fn displace(&self, (dx, dy): &(i16, i16)) -> Option<Self> {
		let mut col = self.column as i16;
		let mut row = self.row as i16;
		if row + dx < 1 || col + dy < 1 {
			return None;
		}
		row += dx;
		col += dy;
		Some(Self {
			row: row as u16,
			column: col as u16,
		})
	}

	pub fn new(row: u16, column: u16) -> Self {
		Self { row, column }
	}

	pub fn new_checked(row: u16, column: u16, board: &BoardOptions) -> Option<Self> {
		if board.validate_point(&Self { row, column }) {
			Some(Self { row, column })
		} else {
			None
		}
	}

	pub fn get_standard_colour(&self) -> Color {
		if (self.row + self.column + 1) % 2 == 0 {
			Color::WHITE
		} else {
			Color::BLACK
		}
	}
}

impl From<(u16, u16)> for ChessPoint {
	fn from((row, column): (u16, u16)) -> Self {
		Self { row, column }
	}
}

impl Display for ChessPoint {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "({}, {})", self.row, self.column)
	}
}

// add two chess points
impl std::ops::Add for ChessPoint {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Self {
			row: self.row + other.row,
			column: self.column + other.column,
		}
	}
}

/// Represents move from one point to another
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

impl From<Vec<Move>> for Moves {
	fn from(moves: Vec<Move>) -> Self {
		Self { moves }
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

#[derive(Debug, Copy, Hash, Clone, PartialEq, Eq, PartialOrd, Ord, EnumIs)]
pub enum CellOption {
	/// Only allows solutions ending on target
	Available {
		can_finish_on: bool,
	},
	Unavailable,
}

mod boardoptions;
pub use boardoptions::*;


