use bevy::prelude::*;
use serde::{Deserialize, Serialize};
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
	pub(crate) fn new(moves: Vec<Move>) -> Self {
		Self { moves }
	}

	pub fn find_move_index(&self, m: &Move) -> Option<usize> {
		self.moves.iter().position(|x| x == m)
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

#[derive(Debug, Copy, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CellOption {
	Available,
	Unavailable,
}

impl CellOption {
	fn is_available(&self) -> bool {
		match self {
			CellOption::Available => true,
			CellOption::Unavailable => false,
		}
	}
}

/// Necessary information to make custom board.
/// Does NOT hold actual state, to solve use [Board]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoardOptions {
	options: Vec<Vec<CellOption>>,
}

impl BoardOptions {
	/// Creates square board with given dimensions and all cells available
	pub fn new(rows: u16, columns: u16) -> Self {
		Self {
			options: vec![vec![CellOption::Available; rows as usize]; columns as usize],
		}
	}

	pub fn get(&self, point: &ChessPoint) -> Option<CellOption> {
		if !self.validate_point(point) {
			return None;
		}

		Some(self.options[point.row as usize - 1][point.column as usize - 1])
	}

	// pub fn set(&mut self, point: &ChessPoint, state: CellOption) {
	// self.options[point.row as usize - 1][point.column as usize - 1] = state;
	// }
	pub fn set(self, point: &ChessPoint, state: CellOption) -> Self {
		let mut options = self.options;
		options[point.row as usize - 1][point.column as usize - 1] = state;
		Self { options }
	}

	// pub fn rm(&mut self, p: (u16, u16)) {
	// 	self.options[p.0 as usize - 1][p.1 as usize - 1] = CellOption::Unavailable;
	// }
	// pub fn add(&mut self, p: (u16, u16)) {
	// 	self.options[p.0 as usize - 1][p.1 as usize - 1] = CellOption::Available;
	// }
	pub fn rm(&mut self, p: impl Into<ChessPoint>) {
		let p = p.into();
		trace!(
			"Removing {} (row len = {}, column len = {})",
			p,
			self.options.len(),
			self.options[0].len()
		);
		self.options[p.row as usize - 1][p.column as usize - 1] = CellOption::Unavailable;
	}
	pub fn add(&mut self, p: impl Into<ChessPoint>) {
		let p = p.into();
		self.options[p.row as usize - 1][p.column as usize - 1] = CellOption::Available;
	}

	/// 1 indexed
	pub fn width(&self) -> u16 {
		self.options[0].len() as u16
	}

	/// 1 indexed
	pub fn height(&self) -> u16 {
		self.options.len() as u16
	}

	pub fn validate_point(&self, p: &ChessPoint) -> bool {
		let bounds_check =
			1 <= p.row && p.row <= self.height() && 1 <= p.column && p.column <= self.width();
		if !bounds_check {
			return false;
		}

		let get_check = self
			.options
			.get(p.row as usize - 1)
			.and_then(|row| row.get(p.column as usize - 1));
		get_check.is_some()
	}

	pub fn validate_point_or_panic(&self, p: &ChessPoint) {
		if !self.validate_point(p) {
			panic!("Invalid point: {:?}", p);
		}
	}

	pub fn update_width(self, new_width: u16) -> Self {
		let mut options = self.options;
		for row in options.iter_mut() {
			if row.len() < new_width as usize {
				row.resize(new_width as usize, CellOption::Available);
			} else {
				row.truncate(new_width as usize);
			}
		}
		Self { options }
	}

	/// Increases/decreases the height of the options,
	/// defaulting to Available for new cells
	pub fn update_height(self, new_height: u16) -> Self {
		let width = self.width() as usize;
		let mut options = self.options;
		if options.len() < new_height as usize {
			options.resize_with(new_height as usize, || vec![CellOption::Available; width]);
		} else {
			options.truncate(new_height as usize);
		}
		Self { options }
	}

	pub fn get_unavailable_points(&self) -> Vec<ChessPoint> {
		let mut points = Vec::new();
		for row in 1..=self.height() {
			for column in 1..=self.width() {
				let p = ChessPoint::new(row, column);
				if self.get(&p) == Some(CellOption::Unavailable) {
					points.push(p);
				}
			}
		}
		points
	}

	pub fn get_all_points(&self) -> Vec<ChessPoint> {
		let mut points = Vec::new();
		for row in 1..=self.height() {
			for column in 1..=self.width() {
				points.push(ChessPoint::new(row, column));
			}
		}
		points
	}

	pub fn get_available_points(&self) -> Vec<ChessPoint> {
		let mut points = Vec::new();
		for row in 1..=self.height() {
			for column in 1..=self.width() {
				let p = ChessPoint::new(row, column);
				if self.get(&p) == Some(CellOption::Available) {
					points.push(p);
				}
			}
		}
		points
	}

	pub fn get_description(&self) -> String {
		format!(
			"{}x{} board with {} cells available (and {} cells disabled)",
			self.height(),
			self.width(),
			self.get_available_points().len(),
			self.get_unavailable_points().len()
		)
	}
}

impl Display for BoardOptions {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for row in self.options.iter().rev() {
			for cell in row.iter() {
				match cell {
					CellOption::Available => write!(f, " ✅ ")?,
					CellOption::Unavailable => write!(f, " ❌ ")?,
				}
			}
			writeln!(f)?;
		}
		Ok(())
	}
}
