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
		if board.options().validate_point(&Self { row, column }) {
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

	pub fn new_checked(from: ChessPoint, to: ChessPoint, board: &BoardOptions) -> Option<Self> {
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

/// Necessary information to make custom board.
/// Does NOT hold actual state, to solve use [Board]
#[derive(Debug, Clone)]
pub struct BoardOptions {
	options: Vec<Vec<CellOption>>,
}

impl BoardOptions {
	/// Creates square board with given dimensions and all cells available
	pub fn new(rows: u8, columns: u8) -> Self {
		Self {
			options: vec![vec![CellOption::Available; columns as usize]; rows as usize],
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

	/// 1 indexed
	pub fn width(&self) -> u8 {
		self.options[0].len() as u8
	}

	/// 1 indexed
	pub fn height(&self) -> u8 {
		self.options.len() as u8
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

	pub fn update_width(self, new_width: u8) -> Self {
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
	pub fn update_height(self, new_height: u8) -> Self {
		let width = self.width() as usize;
		let mut options = self.options;
		if options.len() < new_height as usize {
			options.resize_with(new_height as usize, || {
				vec![CellOption::Available; width]
			});
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
}

impl From<BoardOptions> for CellStates {
	fn from(options: BoardOptions) -> Self {
		options
			.options
			.into_iter()
			.map(|row| row.into_iter().map(|cell| cell.into()).collect())
			.collect()
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

pub struct Board {
	cell_states: CellStates,
	options: BoardOptions,
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

impl Board {
	/// Creates square board with given dimensions and all cells available
	pub fn new(rows: u8, columns: u8) -> Self {
		let options = BoardOptions::new(rows, columns);
		Self {
			options: options.clone(),
			cell_states: options.into(),
		}
	}

	pub fn from_options(cell_options: BoardOptions) -> Self {
		let cell_states = cell_options
			.clone()
			.options
			.into_iter()
			.map(|row| row.into_iter().map(|cell| cell.into()).collect())
			.collect();
		Self {
			cell_states,
			options: cell_options,
		}
	}

	pub fn options(&self) -> &BoardOptions {
		&self.options
	}

	fn get(&self, p: &ChessPoint) -> Option<CellState> {
		if !self.options.validate_point(p) {
			return None;
		}
		Some(self.cell_states[p.row as usize - 1][p.column as usize - 1])
	}

	fn set(&mut self, p: ChessPoint, state: CellState) {
		self.cell_states[p.row as usize - 1][p.column as usize - 1] = state;
	}

	/// Returns true if point is NeverOccupied.
	/// Returns None if point is invalid
	fn get_availability_no_repeat(&self, p: &ChessPoint) -> Option<bool> {
		match self.get(p) {
			Some(CellState::NeverOccupied) => Some(true),
			Some(CellState::HasBeenOccupied(_)) | Some(CellState::Unavailable) => Some(false),
			None => None,
		}
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
		self.options().validate_point_or_panic(&start);

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

	pub fn all_unvisited_available_points(&self) -> Vec<ChessPoint> {
		let mut points = Vec::new();
		for row in 1..=self.options().height() {
			for column in 1..=self.options().width() {
				let p = ChessPoint::new(row, column);
				if self.get_availability_no_repeat(&p) == Some(true) {
					points.push(p);
				}
			}
		}
		points
	}
}

impl Deref for Board {
	type Target = CellStates;

	fn deref(&self) -> &Self::Target {
		&self.cell_states
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

pub fn piece_tour_no_repeat(
	piece: &impl ChessPiece,
	board: &mut Board,
	start: ChessPoint,
) -> Option<Moves> {
	let board = board;
	let mut moves = Vec::new();
	let mut current = start;

	let num_available_cells = board
		.options()
		.get_all_points()
		.iter()
		.filter(|p| board.get_availability_no_repeat(p) == Some(true))
		.count();

	for _ in 1..num_available_cells {
		if !board.options().validate_point(&current) {
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
			moves.push(
				Move::new_checked(current, next, board.options()).expect("moves generated to be valid"),
			);
			current = next;
		} else {
			return None;
		}
	}

	Some(moves.into())
}
