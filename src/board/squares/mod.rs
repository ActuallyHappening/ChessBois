use bevy::prelude::*;
use std::f32::consts::TAU;

use super::*;
use crate::solver::CellOption;
use crate::utils::EntityCommandsExt;
use crate::*;
use crate::ChessPoint;
use derive_more::{From, Into};

pub use markers::CellMark;

use coords::*;

pub use cells::{CellClicked, CellHovered, CellUnhovered, CellColouring};

mod cells;
mod coords;
mod markers;
pub mod visualization;

pub struct SquaresPlugin;
impl Plugin for SquaresPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_event::<CellClicked>()
			.add_event::<CellHovered>()
			.add_event::<CellUnhovered>();
	}
}

/// Immutable version of board_options, used for rendering
pub struct AnnotatedBoardOptions {}

pub struct AnnotatedCellOptions {
	tint: Color,
	board_option: CellOption,
}
