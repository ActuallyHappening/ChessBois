use bevy::prelude::*;
use bevy::prelude::{Reflect, FromReflect};
use serde::{Deserialize, Serialize};
use std::{
	fmt::{self, Display},
	ops::{Deref, DerefMut},
};
use strum::EnumIs;

pub mod algs;
pub mod pieces;

pub use moves::{Move, Moves};
mod moves;

// 1 indexed
#[derive(
	Copy,
	Component,
	Hash,
	Clone,
	Debug,
	Eq,
	PartialOrd,
	Ord,
	PartialEq,
	Serialize,
	Deserialize,
	Reflect,
	FromReflect,
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


#[derive(
	Debug, Copy, Hash, Clone, PartialEq, Eq, PartialOrd, Ord, EnumIs, Serialize, Deserialize, Reflect, FromReflect
)]
/// Solver: Available or Unavailable
pub enum CellOption {
	/// Only allows solutions ending on target
	Available {
		can_finish_on: bool,
	},
	Unavailable,
	Eliminated,
}

impl CellOption {
	fn unwrap_available(self) -> bool {
		match self {
			CellOption::Available { can_finish_on } => can_finish_on,
			_ => panic!("Tried to unwrap unavailable cell option"),
		}
	}
}

mod boardoptions;
pub use boardoptions::*;
