use super::cells::MarkerMarker;
use super::visualization::spawn_visualization;
use super::visualization::VisualizationComponent;
use super::viz_colours::VizColour;
use super::*;
use crate::errors::Error;
use crate::solver::pieces::ChessPiece;
use crate::solver::pieces::StandardKnight;
use crate::solver::Move;
use crate::solver::Moves;
use crate::ChessPoint;
use crate::ProgramState;
use derive_more::From;
use derive_more::Into;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumIs;
use strum::EnumIter;

pub struct ManualState;
impl Plugin for ManualState {
	fn build(&self, app: &mut App) {
		app
			.add_event::<ManualNextCell>()
			.init_resource::<ManualFreedom>()
			.init_resource::<VizColour>()
			.add_systems(
				(
					handle_manual_visualization,
					handle_new_manual_selected,
					viz_colours::colour_hotkeys,
				)
					.in_set(OnUpdate(ProgramState::Manual)),
			);
	}
}

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
#[derive(Resource, Default, Into, From, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
		serde_json::to_string(self).expect("To be able to convert moves to JSON")
	}

	pub fn add_move(&mut self, from: ChessPoint, to: ChessPoint, colour: VizColour) {
		self.moves.push(Move::new(from, to));
		self.colours.push(colour);
	}

	pub fn undo_move(&mut self) {
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
	mut commands: Commands,
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
					manual_moves.add_move(from, *cell, *viz_col);
				}
				ManualFreedom::AnyPossible => {
					let piece = StandardKnight;
					if piece.is_valid_move(from, *cell) {
						manual_moves.add_move(from, *cell, *viz_col);
					} else {
						let err_msg = format!(
							"Invalid move: {} -> {}; A knight can never make that move",
							from, *cell
						);
						warn!("{err_msg}");
						commands.insert_resource(Error::new(err_msg.clone()));
					}
				}
				ManualFreedom::ValidOnly => {
					let piece = StandardKnight;
					if piece.is_valid_move(from, *cell) {
						if !manual_moves.moves.iter().any(|m| m.to == *cell)
							&& manual_moves.start != Some(*cell)
						{
							manual_moves.add_move(from, *cell, *viz_col);
						} else {
							let err_msg = format!(
								"Invalid move: {:?} -> {:?}; The square you are moving to has already been occupied",
								from, *cell
							);
							warn!("{err_msg}");
							commands.insert_resource(Error::new(err_msg.clone()));
						}
					} else {
						let err_msg = format!(
							"Invalid move: {:?} -> {:?}; A knight can never make that move",
							from, *cell
						);
						warn!("{err_msg}");
						commands.insert_resource(Error::new(err_msg.clone()));
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

pub fn add_default_manual_viz_colour(mut commands: Commands) {
	commands.insert_resource(VizColour::default());
}
