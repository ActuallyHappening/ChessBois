//! For both manual and automatic states

use bevy::prelude::*;

use super::{squares::CellClicked, *};
use crate::{errors::Error, solver::CellOption, GroundClicked, ProgramState};

pub struct SharedPlugin;
impl Plugin for SharedPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems((handle_plane_clicked,))
			.add_systems((handle_cell_clicked,).in_set(OnUpdate(ProgramState::Automatic)));
	}
}

fn handle_plane_clicked(mut click: EventReader<GroundClicked>, state: ResMut<SharedState>) {
	if click.iter().next().is_some() {
		debug!("Plane clicked");
		let state = state.into_inner();
		state.remove_start();
	}
}

/// When cell clicked
fn handle_cell_clicked(
	mut event: EventReader<CellClicked>,
	state: ResMut<SharedState>,
	mut commands: Commands,
) {
	if let Some(CellClicked(point)) = event.iter().next() {
		info!("Cell clicked in auto mode, toggling: {:?}", point);

		let state = state.into_inner();
		match state.get(point) {
			Some(current_point) => match state.on_click {
				ToggleAction::ToggleCellEnabled => match current_point {
					CellOption::Available { .. } => {
						state.rm(*point);
						state.remove_start();
						state.invalidate();
					}
					CellOption::Unavailable => {
						state.add(*point);
					}
				},
				ToggleAction::TargetCell => {
					match current_point {
						CellOption::Available { .. } => {
							info!("Targetting point {}", *point);
							state.toggle_target(*point);
							state.invalidate();
						}
						CellOption::Unavailable => {
							//
						}
					}
				}
			},
			None => {
				let err_msg = format!("Cell {:?} is out of bounds", point);
				warn!("{}", err_msg);
				commands.insert_resource(Error::new(err_msg));
				panic!("Cell out of bounds");
			}
		}
	}
}
