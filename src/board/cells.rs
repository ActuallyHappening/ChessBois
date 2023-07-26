use bevy::prelude::*;
use std::f32::consts::TAU;


use super::automatic::cache::CellMark;
use super::*;
use crate::board::automatic::cache;
use crate::errors::{Error, LogLevel};
use crate::solver::CellOption;
use crate::utils::EntityCommandsExt;
use crate::*;
use crate::{ChessPoint, CELL_DISABLED_COLOUR};
use derive_more::{From, Into};


pub use markers::*;
use cells::*;
use coords::*;

pub use cells::CellClicked;

mod cells;
mod coords;
mod markers;

pub struct CellsPlugin;
impl Plugin for CellsPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<CellClicked>();
	}
}

/// Marker for Markers lol
#[derive(Component)]
pub struct MarkerMarker;


