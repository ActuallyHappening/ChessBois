use bevy::prelude::Resource;
use derive_more::Display;
use strum::{EnumIs, EnumIter};

#[derive(Resource, Display, EnumIs, EnumIter, Default, Debug, Clone, PartialEq, Eq)]
pub enum ManualFreedom {
	#[strum(serialize = "Only valid knights moves")]
	#[default]
	ValidOnly,

	#[strum(serialize = "Any possible knights move")]
	AnyPossible,

	#[strum(serialize = "Completely free")]
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