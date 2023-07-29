use anyhow::Context;
use bevy_egui::egui::Ui;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use crate::{
	board::{coloured_moves::ColouredMoves, SharedState},
	solver::BoardOptions,
};

#[derive(Serialize, Deserialize, Constructor)]
pub struct SavedState {
	pub moves: ColouredMoves,
	pub board_options: BoardOptions,
}

impl TryFrom<SharedState> for SavedState {
	type Error = ();
	fn try_from(value: SharedState) -> Result<Self, Self::Error> {
		Ok(Self {
			moves: value.moves.ok_or(())?,
			board_options: value.board_options,
		})
	}
}

impl SavedState {
	pub fn to_json(&self) -> String {
		serde_json::to_string(&self).unwrap()
	}

	pub fn from_json(json: &str) -> Result<Self, anyhow::Error> {
		match serde_json::from_str::<Self>(json) {
			Ok(state) => Ok(state),
			Err(new_err) => {
				let old_err = v0_2_x::try_depreciated_from_json(json);
				if let Ok(state) = old_err {
					return Ok(state);
				}
				Err(new_err).context("JSON decoding failed & depreciated failed")
			}
		}
	}
}

mod v0_3_x {
	use std::collections::HashMap;

	use serde::{Deserialize, Serialize};

	#[derive(Serialize, Deserialize)]
	struct State {
		moves: Vec<(Point, Point)>,
		options: HashMap<Point, Cell>,
	}

	#[derive(Serialize, Deserialize, Hash, PartialEq, Eq)]
	/// row column
	struct Point(u16, u16);

	impl From<Point> for crate::solver::ChessPoint {
		fn from(value: Point) -> Self {
			Self {
				column: value.1,
				row: value.0,
			}
		}
	}

	impl From<crate::solver::ChessPoint> for Point {
		fn from(value: crate::solver::ChessPoint) -> Self {
			Self(value.row, value.column)
		}
	}

	#[derive(Serialize_repr, Deserialize_repr)]
	enum Cell {
		Disabled = 0,
		Finishable = 1,
		NoFinishable = 2,
	}

	impl From<Cell> for crate::solver::CellOption {
		fn from(value: Cell) -> Self {
			match value {
				Cell::Disabled => Self::Unavailable,
				Cell::Finishable => Self::Available {
					can_finish_on: true,
				},
				Cell::NoFinishable => Self::Available {
					can_finish_on: false,
				},
			}
		}
	}

	impl From<crate::solver::CellOption> for Cell {
		fn from(value: crate::solver::CellOption) -> Self {
			match value {
				crate::solver::CellOption::Unavailable => Self::Disabled,
				crate::solver::CellOption::Available { can_finish_on } => {
					if can_finish_on {
						Self::Finishable
					} else {
						Self::NoFinishable
					}
				}
			}
		}
	}
}

mod v0_2_x {
	use std::collections::HashSet;

	use anyhow::Context;
	use serde::{Deserialize, Serialize};

	use crate::solver::BoardOptions;

	use super::SavedState;

	/// Old didn't record targets or board size, so heuristics are added to the board options
	pub fn try_depreciated_from_json(json: &str) -> Result<SavedState, anyhow::Error> {
		let mut data: State = serde_json::from_str(json).context("JSON decoding failed")?;
		let mut reached_points = HashSet::new();
		for Move { from, to } in data.moves.moves.iter() {
			reached_points.insert(from);
			reached_points.insert(to);
		}
		let max_width = reached_points
			.iter()
			.map(|p| p.column)
			.max()
			.context("No moves")?;
		let max_height = reached_points.iter().map(|p| p.row).max().unwrap();

		let board_options = BoardOptions::new(max_width, max_height);
		let moves = data
			.moves
			.moves
			.into_iter()
			.map(|m| (m.into(), data.colours.pop().unwrap().into()))
			.collect();

		Ok(SavedState {
			moves,
			board_options,
		})
	}

	#[test]
	fn data1() {
		let data = r#"{"start":{"column":1,"row":1},"moves":{"moves":[{"from":{"column":1,"row":1},"to":{"column":2,"row":3}},{"from":{"column":2,"row":3},"to":{"column":1,"row":5}},{"from":{"column":1,"row":5},"to":{"column":3,"row":6}},{"from":{"column":3,"row":6},"to":{"column":5,"row":5}},{"from":{"column":5,"row":5},"to":{"column":4,"row":3}},{"from":{"column":4,"row":3},"to":{"column":5,"row":1}},{"from":{"column":5,"row":1},"to":{"column":3,"row":2}},{"from":{"column":3,"row":2},"to":{"column":4,"row":4}},{"from":{"column":4,"row":4},"to":{"column":5,"row":6}},{"from":{"column":5,"row":6},"to":{"column":3,"row":5}},{"from":{"column":3,"row":5},"to":{"column":1,"row":6}},{"from":{"column":1,"row":6},"to":{"column":2,"row":4}},{"from":{"column":2,"row":4},"to":{"column":1,"row":2}},{"from":{"column":1,"row":2},"to":{"column":3,"row":1}},{"from":{"column":3,"row":1},"to":{"column":5,"row":2}},{"from":{"column":5,"row":2},"to":{"column":3,"row":3}},{"from":{"column":3,"row":3},"to":{"column":5,"row":4}},{"from":{"column":5,"row":4},"to":{"column":4,"row":2}},{"from":{"column":4,"row":2},"to":{"column":2,"row":1}},{"from":{"column":2,"row":1},"to":{"column":1,"row":3}},{"from":{"column":1,"row":3},"to":{"column":2,"row":5}},{"from":{"column":2,"row":5},"to":{"column":4,"row":6}},{"from":{"column":4,"row":6},"to":{"column":3,"row":4}},{"from":{"column":3,"row":4},"to":{"column":5,"row":3}},{"from":{"column":5,"row":3},"to":{"column":4,"row":1}},{"from":{"column":4,"row":1},"to":{"column":2,"row":2}},{"from":{"column":2,"row":2},"to":{"column":1,"row":4}},{"from":{"column":1,"row":4},"to":{"column":2,"row":6}},{"from":{"column":2,"row":6},"to":{"column":4,"row":5}},{"from":{"column":4,"row":5},"to":{"column":4,"row":5}}]},"colours":["Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green"]}"#;
		let state = try_depreciated_from_json(data).unwrap();
	}

	#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
	struct State {
		moves: Moves,
		colours: Vec<Colour>,
	}

	#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
	struct Moves {
		moves: Vec<Move>,
	}

	#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
	struct ChessPoint {
		column: u16,
		row: u16,
	}

	impl From<ChessPoint> for crate::solver::ChessPoint {
		fn from(value: ChessPoint) -> Self {
			Self {
				column: value.column,
				row: value.row,
			}
		}
	}

	#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
	struct Move {
		from: ChessPoint,
		to: ChessPoint,
	}

	impl From<Move> for crate::solver::Move {
		fn from(value: Move) -> Self {
			Self {
				from: value.from.into(),
				to: value.to.into(),
			}
		}
	}

	#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
	enum Colour {
		Green,
		Blue,
		Red,
		Orange,
		Invisible,
	}

	impl From<Colour> for crate::board::VizColour {
		fn from(value: Colour) -> Self {
			match value {
				Colour::Green => Self::Green,
				Colour::Blue => Self::Blue,
				Colour::Red => Self::Red,
				Colour::Orange => Self::Orange,
				Colour::Invisible => Self::Invisible,
			}
		}
	}
}

impl SharedState {
	pub fn saved_state_ui(&mut self, ui: &mut Ui) {
		#[cfg(not(target_arch = "wasm32"))]
		if self.moves.is_some() && ui.button("Save").clicked() {
			let state = SavedState::try_from(self.clone()).unwrap();
			let json = state.to_json();
			ui.output_mut(|out| {
				out.copied_text = json;
			})
		}
		#[cfg(not(target_arch = "wasm32"))]
		if ui.button("Load").clicked() {
			let json = crate::clipboard::get_from_clipboard();
			if let Ok(state) = SavedState::from_json(&json) {
				self.moves = Some(state.moves);
				self.board_options = state.board_options;
			};
		}
	}
}
