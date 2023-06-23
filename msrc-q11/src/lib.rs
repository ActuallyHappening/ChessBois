#[cfg(target_arch = "wasm32")]
pub fn init_debug_tools() {
	use tracing_subscriber::fmt::format::Pretty;
	use tracing_subscriber::fmt::time::UtcTime;
	use tracing_subscriber::prelude::*;
	use tracing_web::{performance_layer, MakeConsoleWriter};

	console_error_panic_hook::set_once();

	let fmt_layer = tracing_subscriber::fmt::layer()
			.with_ansi(false) // Only partially supported across browsers
			.with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
			.with_writer(MakeConsoleWriter) // write events to the console
			// .with_span_events(FmtSpan::ACTIVE)
		;
	let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

	tracing_subscriber::registry()
		.with(fmt_layer)
		.with(perf_layer)
		.init(); // Install these as subscribers to tracing events
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init_debug_tools() {
	use tracing::Level;
	use tracing_subscriber::FmtSubscriber;
	let subscriber = FmtSubscriber::builder()
		.with_max_level(Level::TRACE)
		.finish();
	tracing::subscriber::set_global_default(subscriber).unwrap();
}

use std::{
	fmt::{self, Display},
	ops::Deref,
};
pub mod old;
use bevy::prelude::*;

// 1 indexed
#[derive(Copy, Component, Reflect, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct ChessPoint {
	// Between 1 and COLUMN_SIZE.
	/// Corresponds to x axis
	pub column: u8,

	// Between 1 and ROW_SIZE.
	/// Corresponds to y axis
	pub row: u8,
}

impl ChessPoint {
	fn mov(&self, &(dx, dy): &(i8, i8)) -> Self {
		Self {
			row: self.row.wrapping_add(dx as u8),
			column: self.column.wrapping_add(dy as u8),
		}
	}

	pub fn new(row: u8, column: u8) -> Self {
		Self { row, column }
	}

	pub fn new_checked(row: u8, column: u8, board: &Board) -> Option<Self> {
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

impl From<(u8, u8)> for ChessPoint {
	fn from((row, column): (u8, u8)) -> Self {
		Self { row, column }
	}
}

impl Display for ChessPoint {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "({}, {})", self.column, self.row)
	}
}

// add two chesspoints
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
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Move {
	pub from: ChessPoint,
	pub to: ChessPoint,
}

impl Move {
	pub fn new(from: ChessPoint, to: ChessPoint) -> Self {
		Self { from, to }
	}

	pub fn new_checked(from: ChessPoint, to: ChessPoint, board: &Board) -> Option<Self> {
		if board.validate_point(&from) && board.validate_point(&to) {
			Some(Self { from, to })
		} else {
			None
		}
	}
}

#[derive(Debug)]
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

/// State of active board
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CellState {
	/// Can be moved to but never has
	NeverOccupied,

	/// Has been occupied but can be moved to again.
	/// number represents the order in which it was occupied
	///
	/// TODO: add number of crosses as well
	HasBeenOccupied(u8),

	/// Can't be moved to
	Unavailable,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CellOption {
	Available,
	Unavailable,
}

impl From<CellOption> for CellState {
	fn from(o: CellOption) -> Self {
		match o {
			CellOption::Available => CellState::NeverOccupied,
			CellOption::Unavailable => CellState::Unavailable,
		}
	}
}

impl CellOption {
	fn is_available(&self) -> bool {
		match self {
			CellOption::Available => true,
			CellOption::Unavailable => false,
		}
	}
}

pub type CellStates = Vec<Vec<CellState>>;

#[derive(Debug, Clone)]
pub struct CellOptions(Vec<Vec<CellOption>>);

impl CellOptions {
	/// Creates square board with given dimensions and all cells available
	pub fn new(rows: u8, columns: u8) -> Self {
		Self(vec![
			vec![CellOption::Available; columns as usize];
			rows as usize
		])
	}
}

impl From<CellOptions> for CellStates {
	fn from(options: CellOptions) -> Self {
		options
			.0
			.into_iter()
			.map(|row| row.into_iter().map(|cell| cell.into()).collect())
			.collect()
	}
}

pub struct Board {
	cell_states: CellStates,
}

impl Default for Board {
	fn default() -> Self {
		Self::new(8, 8)
	}
}

impl Display for Board {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for row in self.cell_states.iter() {
			for cell in row.iter() {
				match cell {
					CellState::NeverOccupied => write!(f, " 0 ")?,
					CellState::HasBeenOccupied(n) => write!(f, "{:2} ", n)?,
					CellState::Unavailable => write!(f, " X ")?,
				}
			}
			writeln!(f)?;
		}
		Ok(())
	}
}

pub trait ChessPiece {
	fn relative_moves(&self) -> &[(i8, i8)];
}
pub struct StandardKnight;
impl ChessPiece for StandardKnight {
	fn relative_moves(&self) -> &[(i8, i8)] {
		&[
			(2, 1),
			(1, 2),
			(-1, 2),
			(-2, 1),
			(-2, -1),
			(-1, -2),
			(1, -2),
			(2, -1),
		]
	}
}

impl Board {
	/// Creates square board with given dimensions and all cells available
	pub fn new(rows: u8, columns: u8) -> Self {
		Self {
			cell_states: CellOptions::new(rows, columns).into(),
		}
	}

	pub fn from_options(cell_options: CellOptions) -> Self {
		let cell_states = cell_options
			.0
			.into_iter()
			.map(|row| row.into_iter().map(|cell| cell.into()).collect())
			.collect();
		Self { cell_states }
	}

	pub fn validate_point(&self, p: &ChessPoint) -> bool {
		// 1 <= p.row
		// 	&& p.row <= self.cell_states.len() as u8
		// 	&& 1 <= p.column
		// 	&& p.column <= self.cell_states[0].len() as u8
		1 <= p.row && p.row <= self.height() && 1 <= p.column && p.column <= self.width()
	}

	pub fn validate_point_or_panic(&self, p: &ChessPoint) {
		if !self.validate_point(p) {
			panic!("Invalid point: {:?}", p);
		}
	}

	fn get(&self, p: &ChessPoint) -> CellState {
		self.validate_point_or_panic(p);

		self.cell_states[p.row as usize - 1][p.column as usize - 1]
	}

	fn set(&mut self, p: ChessPoint, state: CellState) {
		self.cell_states[p.row as usize - 1][p.column as usize - 1] = state;
	}

	/// Returns true if point is NeverOccupied.
	/// Returns None if point is invalid
	fn get_availability_no_repeat(&self, p: &ChessPoint) -> Option<bool> {
		if !self.validate_point(p) {
			return None;
		}

		Some(matches!(self.get(p), CellState::NeverOccupied))
	}

	/// Returns bool if point is NeverOccupied or HasBeenOccupied
	// fn get_availability_allowing_repeat(&self, p: ChessPoint) -> bool {
	// 	self.validate_point_or_panic(p);

	// 	matches!(
	// 		self.get(p),
	// 		CellState::NeverOccupied | CellState::HasBeenOccupied(_)
	// 	)
	// }

	fn get_degree_no_repeat(&self, start: ChessPoint, moves: &impl ChessPiece) -> u16 {
		self.validate_point_or_panic(&start);

		let mut degree = 0;
		for &(dx, dy) in moves.relative_moves() {
			let p = start.mov(&(dx, dy));
			if self.get_availability_no_repeat(&p) == Some(true) {
				degree += 1;
			}
		}
		degree
	}

	// fn get_degree_allowing_repeat(&self, start: ChessPoint, moves: &impl ChessPiece) -> u16 {
	// 	self.validate_point_or_panic(start);

	// 	let mut degree = 0;
	// 	for &(dx, dy) in moves.relative_moves() {
	// 		let p = start.mov(&(dx, dy));
	// 		if self.get_availability_allowing_repeat(p) {
	// 			degree += 1;
	// 		}
	// 	}
	// 	degree
	// }

	/// 1 indexed
	pub fn width(&self) -> u8 {
		self.cell_states[0].len() as u8
	}

	/// 1 indexed
	pub fn height(&self) -> u8 {
		self.cell_states.len() as u8
	}

	pub fn all_unvisited_available_points(&self) -> Vec<ChessPoint> {
		let mut points = Vec::new();
		for row in 1..=self.height() {
			for column in 1..=self.width() {
				let p = ChessPoint::new(row, column);
				if self.get_availability_no_repeat(&p) == Some(true) {
					points.push(p);
				}
			}
		}
		points
	}
}

pub fn piece_tour_no_repeat(
	piece: &impl ChessPiece,
	board: &mut Board,
	start: ChessPoint,
) -> Option<Moves> {
	let board = board;
	let mut moves = Vec::new();
	let mut current = start;

	for _ in 1..board.width() * board.height() {
		if !board.validate_point(&current) {
			return None;
		}

		// board.cell_states[current.row as usize - 1][current.column as usize - 1] =
		// CellState::HasBeenOccupied(moves.len() as u8 + 1);
		board.set(current, CellState::HasBeenOccupied(moves.len() as u8 + 1));

		let mut next = None;
		let mut min_degree = u16::MAX;
		for &(dx, dy) in piece.relative_moves() {
			let p = current.mov(&(dx, dy));
			if board.get_availability_no_repeat(&p) == Some(true) {
				let degree = board.get_degree_no_repeat(p, piece);
				if degree < min_degree {
					min_degree = degree;
					next = Some(p);
				}
			}
		}

		if let Some(next) = next {
			moves.push(Move::new_checked(current, next, board).expect("moves generated to be valid"));
			current = next;
		} else {
			return None;
		}
	}

	Some(moves.into())
}
