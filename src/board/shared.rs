//! For both manual and automatic states

use bevy::prelude::*;

use super::{*, squares::CellHovered};
use crate::{GroundClicked, ProgramState};

pub struct SharedPlugin;
impl Plugin for SharedPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems((handle_plane_clicked,))
			;
	}
}

impl ProgramState {
	pub fn switch(&self, to: Self, _state: &mut SharedState) {
		match (self, to) {
			(ProgramState::Automatic, ProgramState::Manual) => {
				
			}
			(ProgramState::Manual, ProgramState::Automatic) => {

			}
			(from, to) => panic!("Unacceptable state transition {:?}", (from, to)),
		}
	}
}

fn handle_plane_clicked(mut click: EventReader<GroundClicked>, state: ResMut<SharedState>) {
	if click.iter().next().is_some() {
		debug!("Plane clicked");
		let state = state.into_inner();
		state.remove_start();
	}
}

