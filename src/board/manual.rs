use crate::ProgramState;

use super::{*, squares::CellHovered};

pub use freedom::ManualFreedom;
mod freedom;

pub struct ManualState;
impl Plugin for ManualState {
	fn build(&self, app: &mut App) {
		app.add_system(highlight_hovered_cell.in_set(OnUpdate(ProgramState::Manual)));
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

