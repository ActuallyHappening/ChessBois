use crate::ProgramState;

use super::{*, squares::{CellHovered, CellClicked}};

pub use freedom::ManualFreedom;
pub use save::UnstableSavedState;
mod freedom;
mod check_move;
mod save;

pub struct ManualState;
impl Plugin for ManualState {
	fn build(&self, app: &mut App) {
		app.add_systems((highlight_hovered_cell, handle_cell_clicked).in_set(OnUpdate(ProgramState::Manual)));
	}
}

fn highlight_hovered_cell(
	mut state: ResMut<SharedState>,
	mut hovered: EventReader<CellHovered>,
) {
	if let Some(CellHovered(point)) = hovered.iter().next() {
		state.start = Some(*point);
	}
}

fn handle_cell_clicked(
	mut click: EventReader<CellClicked>,
	mut state: ResMut<SharedState>,
) {
	if let Some(CellClicked(point)) = click.iter().next() {
		if state.moves.is_none() {
			state.moves = Some(ColouredMoves::default());
		}
		if let (true, _) = state.manual_freedom.check_move(&state, *point) {
			let col = state.viz_colour;
			state.moves.as_mut().unwrap().manual_add_move(*point, col);
		}
	}
}