//! For both manual and automatic states

use bevy::prelude::*;

use super::*;
use crate::{GroundClicked, ProgramState};

pub struct SharedPlugin;
impl Plugin for SharedPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems((handle_plane_clicked,)).add_system(
			ProgramState::Automatic
				.sys_switch_to()
				.in_schedule(OnEnter(ProgramState::Automatic)),
		).add_system(
			ProgramState::Manual
				.sys_switch_to()
				.in_schedule(OnEnter(ProgramState::Manual)),
		);
	}
}

impl SharedState {
	pub fn switch(&mut self, to: ProgramState) {
		match to {
			ProgramState::Manual => {
				self.visual_opts.show_numbers = false;
			}
			ProgramState::Automatic => {
				self.visual_opts.show_numbers = true;
			}
		}
	}
}

impl ProgramState {
	pub fn sys_switch_to(self) -> impl Fn(ResMut<SharedState>) {
		move |state: ResMut<SharedState>| {
			state.into_inner().switch(self);
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
