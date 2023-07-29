use bevy::prelude::*;
use std::f32::consts::TAU;

use super::*;
use crate::errors::{Error, LogLevel};
use crate::solver::CellOption;
use crate::utils::EntityCommandsExt;
use crate::*;
use crate::{ChessPoint, CELL_DISABLED_COLOUR};
use derive_more::{From, Into};

pub use markers::CellMark;

use coords::*;

pub use cells::CellClicked;

mod cells;
mod coords;
mod markers;
pub mod visualization;

pub struct CellsPlugin;
impl Plugin for CellsPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<CellClicked>();
	}
}

/// Immutable version of board_options, used for rendering
pub struct AnnotatedBoardOptions {}

pub struct AnnotatedCellOptions {
	tint: Color,
	board_option: CellOption,
}

/// Marker for Markers lol
#[derive(Component)]
pub struct MarkerMarker;
