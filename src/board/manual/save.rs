use anyhow::Context;
use bevy::reflect::{Reflect, FromReflect};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
	board::{coloured_moves::ColouredMoves, SharedState, VizColour},
	solver::{BoardOptions, Move},
};

pub use ui::SaveState;
mod ui;

#[path = "firebase.rs"]
mod firebase;

/// Serialized
pub type StableSavedState = v0_3_x::StableSavedState;

/// Convertable from/into (Unstable)[SharedState], convertable from/into [StableSavedState].
/// Bridge from stability into unstability
#[derive(Serialize, Deserialize, Debug)]
pub struct UnstableSavedState {
	metadata: MetaData,
	moves: ColouredMoves,
	board_options: BoardOptions,
}

/// This is stable
#[derive(Serialize, Deserialize, Debug, Clone, Reflect, FromReflect)]
pub struct MetaData {
	pub id: Option<firebase::ID>,
	pub title: String,
	pub author: String,
	pub description: String,
	pub dimensions: Dimensions,
}

impl MetaData {
	/// Remember to update `MetaData.dimensions`
	fn dangerous_default() -> Self {
			MetaData {
				id: None,
				title: "".into(),
				author: "".into(),
				description: "".into(),
				dimensions: (8, 8),
			}
	}
}

pub type Dimensions = (u16, u16);

impl TryFrom<SharedState> for UnstableSavedState {
	type Error = String;
	fn try_from(state: SharedState) -> Result<Self, Self::Error> {
		Ok(Self {
			metadata: state.clone().try_into()?,
			moves: state.moves.ok_or("No moves to save")?,
			board_options: state.board_options,
		})
	}
}

impl SharedState {
	pub fn dangerous_into(self) -> UnstableSavedState {
		let dimensions = self.board_options.dimensions();
		UnstableSavedState {
			board_options: self.board_options.clone(),
			moves: self.moves.clone().unwrap(),
			metadata: self.try_into().unwrap_or_else(|err| {
				info!("De-Serializing dangerously gave error: {:?}", err);
				let mut metadata = MetaData::dangerous_default();
				metadata.dimensions = dimensions;
				metadata
			}),
		}
	}
}

impl UnstableSavedState {
	/// Serialize into the Stable state
	pub fn into_json(self) -> String {
		let data = v0_3_x::StableSavedState::from(self);
		serde_json::to_string(&data).expect("Cannot serialise data")
	}

	/// Hnadles depreciated format
	pub fn from_json(json: &str) -> Result<Self, anyhow::Error> {
		match serde_json::from_str::<v0_3_x::StableSavedState>(json) {
			Ok(state) => Ok(state.into()),
			Err(new_err) => {
				let old_err = v0_2_x::try_depreciated_from_json(json);
				if let Ok(state) = old_err {
					return Ok(state);
				}
				Err(new_err).context("JSON decoding failed & depreciated failed")
			}
		}
	}

	pub fn apply_to_state(self, state: &mut SharedState) {
		state.board_options = self.board_options;
		state.moves = Some(self.moves);
		state.web_vis = Some(self.metadata);
		state.is_web_vis_first_render = true;
	}
}

#[test]
fn check_serialize_deserialize_works() {
	use crate::solver::ChessPoint;
	let mut moves = ColouredMoves::default();
	moves.manual_add_move(ChessPoint::new(2, 3), VizColour::Blue);

	let board_options = BoardOptions::new(2, 3);

	let data = UnstableSavedState {
		metadata: MetaData {
			id: None,
			title: "test".into(),
			author: "test".into(),
			description: "test".into(),
			dimensions: (2, 3),
		},
		moves,
		board_options,
	};

	let json = data.into_json();
	let _data = UnstableSavedState::from_json(&json).unwrap();
}

mod v0_3_x {
	use bevy::prelude::Color;
	use derive_more::{Deref, DerefMut, From, Into};
	use serde::{Deserialize, Serialize};
	use serde_json_any_key::any_key_map;
	use std::collections::HashMap;

	#[derive(Serialize, Deserialize, Debug)]
	pub struct StableSavedState {
		moves: self::StableColouredMoves,
		board_options: self::StableBoardOptions,
		metadata: super::MetaData,
	}

	#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug)]
	/// row column
	pub struct Point(u16, u16);

	#[derive(Serialize, Deserialize, PartialEq, Debug, From, Into)]
	pub struct StableColor(f32, f32, f32, f32);

	#[derive(Serialize, Deserialize, Deref, DerefMut, From, Into, Debug)]
	struct StableColouredMoves(Vec<(Point, Point, StableColor)>);

	#[derive(Serialize, Deserialize, Deref, DerefMut, From, Into, Debug)]
	pub struct StableBoardOptions(#[serde(with = "any_key_map")] HashMap<Point, StableCellOptions>);

	// from impls

	impl From<StableSavedState> for super::UnstableSavedState {
		/// Un-stabalise the [StableSavedState]
		fn from(value: StableSavedState) -> Self {
			Self {
				moves: value.moves.into(),
				board_options: value.board_options.into(),
				metadata: value.metadata,
			}
		}
	}

	impl From<super::UnstableSavedState> for StableSavedState {
		/// Stabalise the [UnstableSavedState]
		fn from(value: super::UnstableSavedState) -> Self {
			Self {
				moves: value.moves.into(),
				board_options: value.board_options.into(),
				metadata: value.metadata,
			}
		}
	}

	impl From<super::ColouredMoves> for StableColouredMoves {
		fn from(value: super::ColouredMoves) -> Self {
			Self(
				<Vec<_>>::from(value)
					.into_iter()
					.map(|(super::Move { from, to }, colour)| (from.into(), to.into(), colour.into()))
					.collect::<Vec<(Point, Point, _)>>(),
			)
		}
	}

	impl From<StableColouredMoves> for super::ColouredMoves {
		fn from(value: StableColouredMoves) -> Self {
			Self::from(
				<Vec<_>>::from(value)
					.into_iter()
					.map(|(from, to, colour)| {
						(
							super::Move {
								from: from.into(),
								to: to.into(),
							},
							colour.into(),
						)
					})
					.collect::<Vec<(super::Move, super::VizColour)>>(),
			)
		}
	}

	impl From<super::BoardOptions> for StableBoardOptions {
		fn from(value: super::BoardOptions) -> Self {
			Self(
				value
					.into_iter()
					.map(|(point, cell_option)| (point.into(), cell_option.into()))
					.collect::<HashMap<Point, StableCellOptions>>(),
			)
		}
	}

	impl StableBoardOptions {
		pub fn dimensions(&self) -> (u16, u16) {
			let mut max_width = 0;
			let mut max_height = 0;
			for (Point(row, column), _) in self.iter() {
				if *row > max_height {
					max_height = *row;
				}
				if *column > max_width {
					max_width = *column;
				}
			}
			(max_width, max_height)
		}
	}
	impl From<StableBoardOptions> for super::BoardOptions {
		fn from(mut value: StableBoardOptions) -> super::BoardOptions {
			let dimensions = value.dimensions();
			let mut board_options = super::BoardOptions::new(dimensions.0, dimensions.1);
			for (point, cell_option) in value.drain() {
				board_options.set_point(point, cell_option.into());
			}
			board_options
		}
	}

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

	#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug)]
	#[repr(u8)]
	pub enum StableCellOptions {
		Disabled = 0,
		Finishable = 1,
		NoFinishable = 2,
	}

	impl From<StableCellOptions> for crate::solver::CellOption {
		fn from(value: StableCellOptions) -> Self {
			match value {
				StableCellOptions::Disabled => Self::Unavailable,
				StableCellOptions::Finishable => Self::Available {
					can_finish_on: true,
				},
				StableCellOptions::NoFinishable => Self::Available {
					can_finish_on: false,
				},
			}
		}
	}

	impl From<crate::solver::CellOption> for StableCellOptions {
		fn from(value: crate::solver::CellOption) -> Self {
			match value {
				crate::solver::CellOption::Unavailable | crate::solver::CellOption::Eliminated => Self::Disabled,
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

	impl From<StableColor> for crate::board::VizColour {
		fn from(value: StableColor) -> Self {
			Self::from(Color::rgba(value.0, value.1, value.2, value.3))
		}
	}
	impl From<super::VizColour> for StableColor {
		fn from(value: super::VizColour) -> Self {
			let [r, g, b, a] = Color::from(value).as_rgba_f32();
			Self(r, g, b, a)
		}
	}
}

mod v0_2_x {
	use std::collections::HashSet;

	use anyhow::Context;
	use serde::{Deserialize, Serialize};

	use crate::solver::BoardOptions;

	use super::UnstableSavedState;

	/// Old didn't record targets or board size, so heuristics are added to the board options
	pub fn try_depreciated_from_json(json: &str) -> Result<UnstableSavedState, anyhow::Error> {
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

		Ok(UnstableSavedState {
			moves,
			board_options,
			metadata: super::MetaData::depreciated((max_width, max_height)),
		})
	}

	impl super::MetaData {
		fn depreciated(dimensions: super::Dimensions) -> super::MetaData {
			super::MetaData {
				id: None,
				title: "OLD save, no title".into(),
				author: "OLD save, unknown author".into(),
				description: "OLD save, no description".into(),
				dimensions,
			}
		}
	}

	#[test]
	fn data1() {
		let data = r#"{"start":{"column":1,"row":1},"moves":{"moves":[{"from":{"column":1,"row":1},"to":{"column":2,"row":3}},{"from":{"column":2,"row":3},"to":{"column":1,"row":5}},{"from":{"column":1,"row":5},"to":{"column":3,"row":6}},{"from":{"column":3,"row":6},"to":{"column":5,"row":5}},{"from":{"column":5,"row":5},"to":{"column":4,"row":3}},{"from":{"column":4,"row":3},"to":{"column":5,"row":1}},{"from":{"column":5,"row":1},"to":{"column":3,"row":2}},{"from":{"column":3,"row":2},"to":{"column":4,"row":4}},{"from":{"column":4,"row":4},"to":{"column":5,"row":6}},{"from":{"column":5,"row":6},"to":{"column":3,"row":5}},{"from":{"column":3,"row":5},"to":{"column":1,"row":6}},{"from":{"column":1,"row":6},"to":{"column":2,"row":4}},{"from":{"column":2,"row":4},"to":{"column":1,"row":2}},{"from":{"column":1,"row":2},"to":{"column":3,"row":1}},{"from":{"column":3,"row":1},"to":{"column":5,"row":2}},{"from":{"column":5,"row":2},"to":{"column":3,"row":3}},{"from":{"column":3,"row":3},"to":{"column":5,"row":4}},{"from":{"column":5,"row":4},"to":{"column":4,"row":2}},{"from":{"column":4,"row":2},"to":{"column":2,"row":1}},{"from":{"column":2,"row":1},"to":{"column":1,"row":3}},{"from":{"column":1,"row":3},"to":{"column":2,"row":5}},{"from":{"column":2,"row":5},"to":{"column":4,"row":6}},{"from":{"column":4,"row":6},"to":{"column":3,"row":4}},{"from":{"column":3,"row":4},"to":{"column":5,"row":3}},{"from":{"column":5,"row":3},"to":{"column":4,"row":1}},{"from":{"column":4,"row":1},"to":{"column":2,"row":2}},{"from":{"column":2,"row":2},"to":{"column":1,"row":4}},{"from":{"column":1,"row":4},"to":{"column":2,"row":6}},{"from":{"column":2,"row":6},"to":{"column":4,"row":5}},{"from":{"column":4,"row":5},"to":{"column":4,"row":5}}]},"colours":["Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green","Green"]}"#;
		let _state = try_depreciated_from_json(data).unwrap();
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
