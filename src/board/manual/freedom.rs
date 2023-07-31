use bevy::{prelude::Resource, reflect::{FromReflect, Reflect}};
use bevy_egui::egui::Color32;
use bevy_egui_controls::ControlPanel;
use strum::{EnumIs, EnumIter};

use crate::{
	board::SharedState,
	solver::Moves,
	ChessPoint,
};

#[derive(
	Resource,
	strum::Display,
	EnumIs,
	Default,
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	EnumIter,
	ControlPanel,
	Reflect,
	FromReflect,
)]
pub enum ManualFreedom {
	#[strum(serialize = "Only valid")]
	ValidOnly,

	#[strum(serialize = "Any possible")]
	AnyPossible,

	#[strum(serialize = "Completely free")]
	#[default]
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
