use std::num::NonZeroUsize;

use crate::board::StateInvalidated;

use super::{pieces::ChessPiece, *};
use bevy::prelude::{FromReflect, Reflect};
use bevy_egui::*;

/// Necessary information to make custom board.
/// Derefs to `Vec<Vec<CellOption>>`, is mutable.
#[derive(
	Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Reflect, FromReflect,
)]
pub struct BoardOptions {
	options: Vec<Vec<CellOption>>,
}

impl Default for BoardOptions {
	fn default() -> Self {
		Self::new(8, 8)
	}
}

impl BoardOptions {
	const MAX_SIZE: u8 = 20;
	const MIN_SIZE: u8 = 2;

	pub fn ui(&mut self, ui: &mut egui::Ui) -> StateInvalidated {
		let mut state = StateInvalidated::Valid;

		ui.add(
			egui::Slider::from_get_set((Self::MIN_SIZE as f64)..=(Self::MAX_SIZE as f64), |val| {
				if let Some(new_val) = val {
					self.set_width(new_val as u16);
					state = StateInvalidated::InvalidatedAndClearStart;
				}

				self.width() as f64
			})
			.step_by(1.0)
			.text("Width"),
		);

		ui.add(
			egui::Slider::from_get_set((Self::MIN_SIZE as f64)..=(Self::MAX_SIZE as f64), |val| {
				if let Some(new_val) = val {
					self.set_height(new_val as u16);
					state = StateInvalidated::InvalidatedAndClearStart;
				}

				self.height() as f64
			})
			.step_by(1.0)
			.text("Height"),
		);

		state
	}
}

impl IntoIterator for BoardOptions {
	type Item = (ChessPoint, CellOption);
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		let mut points = Vec::new();
		for row in 1..=self.height() {
			for column in 1..=self.width() {
				let p = ChessPoint::new(row, column);
				points.push((p, self.get(&p).unwrap()));
			}
		}
		points.into_iter()
	}
}

#[derive(EnumIs)]
pub enum TargetRestriction {
	/// All cells are endable.
	/// New cells should be endable
	AllFinishable,
	/// Only some cells are endable, i.e. there exists at least one cell that is not endable.
	/// New cells should not be enable
	CertainFinishable { num_endable: NonZeroUsize },
	/// No cells are endable.
	/// New cells should not be endable, and really the entire board should be re-enabled
	NoneFinishable,
}

impl TargetRestriction {
	/// Given a board's target state, what should a new cell option state be?
	pub fn into_available_cell_option(self) -> CellOption {
		match self {
			TargetRestriction::AllFinishable => CellOption::Available {
				can_finish_on: true,
			},
			TargetRestriction::CertainFinishable { .. } | TargetRestriction::NoneFinishable => {
				CellOption::Available {
					can_finish_on: false,
				}
			}
		}
	}

	pub fn should_show_targets_visual(self) -> bool {
		!matches!(self, TargetRestriction::AllFinishable)
	}

	pub fn get_description(&self) -> String {
		match self {
			TargetRestriction::AllFinishable => "No specific cells are targetted.".to_string(),
			TargetRestriction::CertainFinishable { num_endable } => {
				format!("Only {} cells are targetted", num_endable)
			}
			TargetRestriction::NoneFinishable => {
				"No cells are targetted, which is impossible?".to_string()
			}
		}
	}
}

impl BoardOptions {
	/// Creates square board with given dimensions and all cells available
	pub fn new(rows: u16, columns: u16) -> Self {
		Self {
			options: vec![
				vec![
					CellOption::Available {
						can_finish_on: true
					};
					rows as usize
				];
				columns as usize
			],
		}
	}

	pub fn get(&self, point: &ChessPoint) -> Option<CellOption> {
		if !self.validate_point(point) {
			return None;
		}

		Some(self.options[point.row as usize - 1][point.column as usize - 1])
	}

	pub fn is_available(&self, point: &ChessPoint) -> bool {
		self.get(point).map_or(false, |state| state.is_available())
	}

	// pub fn set(&mut self, point: &ChessPoint, state: CellOption) {
	// self.options[point.row as usize - 1][point.column as usize - 1] = state;
	// }
	pub fn set(self, point: &ChessPoint, state: CellOption) -> Self {
		let mut options = self.options;
		options[point.row as usize - 1][point.column as usize - 1] = state;
		Self { options }
	}
	fn set_p(&mut self, point: &ChessPoint, state: CellOption) {
		self.options[point.row as usize - 1][point.column as usize - 1] = state;
	}
	pub fn set_point(&mut self, point: impl Into<ChessPoint>, state: CellOption) {
		let point = point.into();
		self.validate_point_or_panic(&point);
		self.set_p(&point, state);
	}

	pub fn targets_state(&self) -> TargetRestriction {
		let available_points = self.get_available_points();
		let total_num = available_points.len();
		let mut points_endable = 0;
		for p in self.get_available_points() {
			if let CellOption::Available { can_finish_on } = self.get(&p).unwrap() {
				if can_finish_on {
					points_endable += 1;
				}
			}
		}

		match points_endable {
			0 => TargetRestriction::NoneFinishable,
			_x if _x == total_num => TargetRestriction::AllFinishable,
			x => TargetRestriction::CertainFinishable {
				num_endable: x.try_into().unwrap(),
			},
		}
	}

	/// If no squares are endable, resets board to all endable
	pub fn check_for_targets_reset(&mut self) {
		if self.targets_state().is_none_finishable() {
			self.reset_targets();
		}
	}

	/// Sets point to [CellOption::Unavailable]
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
	/// Sets point to [CellOption::Available].
	/// Sets can finish to true if no cells are can_finish = false
	pub fn add(&mut self, p: impl Into<ChessPoint>) {
		let p = p.into();
		self.options[p.row as usize - 1][p.column as usize - 1] =
			self.targets_state().into_available_cell_option();
	}

	pub fn dimensions(&self) -> (u16, u16) {
		(self.width(), self.height())
	}

	/// Target/Untargets a specific point.
	/// If this is the first target, sets can_finish to false for all other cells
	pub fn toggle_target(&mut self, p: impl Into<ChessPoint>) {
		let p = p.into();
		info!("Targeting/Untargetting cell {}", p);
		match self.targets_state() {
			// requires reset, only this cell should be endable
			TargetRestriction::AllFinishable => {
				info!("Setting all other cells to can_finish false");
				for p in self.get_available_points() {
					// sets all other points to can_finish = false
					self.set_p(
						&p,
						CellOption::Available {
							can_finish_on: false,
						},
					);
				}
				self.set_p(
					&p,
					CellOption::Available {
						can_finish_on: true,
					},
				);
			}
			// no reset required
			TargetRestriction::CertainFinishable { .. } | TargetRestriction::NoneFinishable => {
				let state = self.get(&p).unwrap();
				match state {
					CellOption::Available { can_finish_on } => {
						self.set_p(
							&p,
							CellOption::Available {
								can_finish_on: !can_finish_on,
							},
						);

						self.check_for_targets_reset();
					}
					CellOption::Unavailable => {
						debug!("Cannot target a cell that is disabled");
					}
				}
			}
		}
	}

	/// Resets all targets to can_finish = true
	pub fn reset_targets(&mut self) {
		for p in self.get_available_points() {
			self.set_p(
				&p,
				CellOption::Available {
					can_finish_on: true,
				},
			);
		}
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

	pub fn set_width(&mut self, new_width: u16) -> &mut Self {
		let new_cell = self.targets_state().into_available_cell_option();
		for row in self.options.iter_mut() {
			if row.len() < new_width as usize {
				row.resize(new_width as usize, new_cell);
			} else {
				row.truncate(new_width as usize);
			}
		}
		self.check_for_targets_reset();
		self
	}

	pub fn set_height(&mut self, new_height: u16) -> &mut Self {
		let width = self.width() as usize;
		let new_cell = self.targets_state().into_available_cell_option();
		if self.options.len() < new_height as usize {
			self
				.options
				.resize_with(new_height as usize, || vec![new_cell; width]);
		} else {
			self.options.truncate(new_height as usize);
		}
		self.check_for_targets_reset();
		self
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
				if let Some(state) = self.get(&p) {
					if state.is_available() {
						points.push(p);
					}
				}
			}
		}
		points
	}

	pub fn get_valid_adjacent_points(&self, start: ChessPoint, piece: &ChessPiece) -> Vec<ChessPoint> {
		piece.get_unchecked_relative_points(start).into_iter().filter(|p| self.validate_point(p) && self.is_available(p)).collect()
	}

	pub fn get_description(&self) -> String {
		format!(
			"Current options: {}x{} board with {} cells available (and {} cells disabled). {}",
			self.height(),
			self.width(),
			self.get_available_points().len(),
			self.get_unavailable_points().len(),
			self.targets_state().get_description(),
		)
	}
}

impl Display for BoardOptions {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for row in self.options.iter().rev() {
			for cell in row.iter() {
				match cell {
					CellOption::Available {
						can_finish_on: false,
					} => write!(f, " ✅ ")?,
					CellOption::Unavailable => write!(f, " ❌ ")?,
					CellOption::Available {
						can_finish_on: true,
					} => write!(f, " 🎯 ")?,
				}
			}
			writeln!(f)?;
		}
		Ok(())
	}
}
