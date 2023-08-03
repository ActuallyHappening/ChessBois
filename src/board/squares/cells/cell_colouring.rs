use bevy::{prelude::*, reflect::FromReflect};

use crate::ChessPoint;

use super::cells_state::BorrowedCellsState;


#[derive(Clone, Default, Reflect, FromReflect)]
pub enum CellColouring {
	#[default]
	StandardChessBoard,

	AllOneColour(Color),
}

const CELL_SELECTED_COLOUR: Color = Color::PURPLE;
const CELL_DISABLED_COLOUR: Color = Color::RED;
const CELL_END_COLOUR_FACTOR: Color = Color::BLUE;

/// Takes as much information as it can get and returns the colour the cell should be.
pub fn compute_colour(point: &ChessPoint, state: &BorrowedCellsState, start: Option<&ChessPoint>) -> Color {
	if state.get_unavailable_points().contains(point) {
		CELL_DISABLED_COLOUR
	} else if Some(point) == start {
		CELL_SELECTED_COLOUR
	} else if state.visual_opts.show_end_colour
		&& state.moves.as_ref().is_some_and(|moves| {
			moves
				.moves()
				.into_iter()
				.last()
				.is_some_and(|last| last.to == *point)
		}) {
		CELL_END_COLOUR_FACTOR
	} else {
		point.get_standard_colour()
	}
}