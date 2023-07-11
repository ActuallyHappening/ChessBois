use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumIs;
use strum::EnumIter;

use super::viz_colours::VizColour;
use super::*;
use crate::solver::Move;
use crate::solver::Moves;

#[derive(
	derive_more::Into,
	derive_more::From,
	derive_more::Deref,
	derive_more::DerefMut,
	Debug,
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
)]
pub struct ManualNextCell {
	pub cell: ChessPoint,
}

/// Resource for storing manual moves to present visualization
#[derive(Resource, Default, derive_more::Into, derive_more::From, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualMoves {
	pub start: Option<ChessPoint>,
	moves: Moves,
	colours: Vec<VizColour>,
}

#[derive(Resource, Display, EnumIs, EnumIter, Default, Debug, Clone, PartialEq, Eq)]
pub enum ManualFreedom {
	#[strum(serialize = "Only valid knights moves")]
	#[default]
	ValidOnly,

	#[strum(serialize = "Any possible knights move")]
	AnyPossible,

	#[strum(serialize = "Completely free")]
	Free,
}

impl ManualFreedom {
	pub fn get_description(&self) -> &'static str {
		match self {
			ManualFreedom::Free => "Chose any move that is on the board and not disabled. The most free option available.",
			ManualFreedom::AnyPossible => "Chose only moves that are valid knight moves. Can still jump onto squares multiple times",
			ManualFreedom::ValidOnly => "Chose only moves that are valid knight moves and have not been visited yet. The most restrictive option available."
		}
	}
}

impl ManualMoves {
	pub fn to_json(&self) -> String {
			// using serde_json
			serde_json::to_string(self).expect("To be able to convert moves to string")
	}

	pub fn add_move(&mut self, from: ChessPoint, to: ChessPoint, colour: VizColour) {
		self.moves.push(Move::new(from, to));
		self.colours.push(colour);
	}

	pub fn undo_move(&mut self,) {
		self.moves.pop();
		self.colours.pop();
	}
}

impl TryFrom<String> for ManualMoves {
	type Error = serde_json::Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		serde_json::from_str(&value)
	}
}

pub fn handle_manual_visualization(
	mut commands: Commands,
	options: Res<CurrentOptions>,

	manual_moves: ResMut<ManualMoves>,

	visualization: Query<Entity, With<VisualizationComponent>>,
	mut mma: ResSpawning,
) {
	let moves = manual_moves.moves.clone();
	super::visualization::despawn_visualization(&mut commands, visualization);
	spawn_visualization(
		moves,
		options.into_inner().current.options.clone(),
		&mut commands,
		&mut mma,
		manual_moves.colours.clone(),
	);
}

pub fn handle_new_manual_selected(
	mut events: EventReader<ManualNextCell>,
	mut manual_moves: ResMut<ManualMoves>,
	current_level: Res<ManualFreedom>,
	viz_col: Res<VizColour>,
) {
	let current_level = current_level.into_inner();
	let viz_col = viz_col.into_inner();
	for ManualNextCell { cell } in events.iter() {
		if manual_moves.start.is_none() {
			manual_moves.start = Some(*cell);
			info!("Manually adding start cell: {:?}", manual_moves.start);
		} else {
			let from = if manual_moves.moves.last().is_none() {
				manual_moves.start.unwrap()
			} else {
				manual_moves.moves.last().unwrap().to
			};

			match current_level {
				ManualFreedom::Free => {
					manual_moves.moves.push(Move::new(from, *cell));
					manual_moves.colours.push(*viz_col);
				}
				ManualFreedom::AnyPossible => {
					let piece = StandardKnight;
					if piece.is_valid_move(from, *cell) {
						manual_moves.moves.push(Move::new(from, *cell));
						manual_moves.colours.push(*viz_col);
					} else {
						warn!(
							"Invalid move: {:?} -> {:?}; A knight can never make that move",
							from, *cell
						);
					}
				}
				ManualFreedom::ValidOnly => {
					let piece = StandardKnight;
					if piece.is_valid_move(from, *cell) && !manual_moves.moves.iter().any(|m| m.to == *cell) {
						manual_moves.moves.push(Move::new(from, *cell));
						manual_moves.colours.push(*viz_col);
					} else {
						warn!("Invalid move: {:?} -> {:?}; A knight can never make that move, OR the square you are moving to has already been occupied", from, *cell);
					}
				}
			}
		}
	}
}

/// overwrites / resets manual moves
pub fn add_empty_manual_moves(mut commands: Commands) {
	commands.insert_resource(ManualMoves::default());
}

pub fn despawn_visualization(
	mut commands: Commands,
	visualization: Query<Entity, With<VisualizationComponent>>,
) {
	super::visualization::despawn_visualization(&mut commands, visualization);
}

pub fn despawn_markers(
	mut commands: Commands,
	markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,
) {
	super::cells::despawn_markers(&mut commands, markers);
}

pub fn add_default_manual_viz_colour(mut commands: Commands) {
	commands.insert_resource(VizColour::default());
}
