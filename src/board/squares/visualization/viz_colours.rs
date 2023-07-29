use bevy_egui_controls::ControlPanel;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use super::*;

#[derive(
	Resource, Copy, Clone, Default, Display, EnumIter, PartialEq, Eq, Debug, Serialize, Deserialize, ControlPanel
)]
pub enum VizColour {
	#[default]
	#[strum(serialize = "Green [g]")]
	Green,

	#[strum(serialize = "Red [r]")]
	Red,
	#[strum(serialize = "Blue [b]]")]
	Blue,
	#[strum(serialize = "Orange [o]")]
	Orange,

	#[strum(serialize = "Invisible [i]")]
	Invisible,
}
impl Hotkeyable for VizColour {}

impl From<VizColour> for Color {
	fn from(colour: VizColour) -> Self {
		match colour {
			VizColour::Green => Color::GREEN,
			VizColour::Red => Color::RED,
			VizColour::Blue => Color::BLUE,
			VizColour::Orange => Color::ORANGE,

			VizColour::Invisible => Color::rgba(0., 0., 0., 0.),
		}
	}
}

impl From<VizColour> for KeyCode {
	fn from(value: VizColour) -> Self {
		match value {
			VizColour::Green => KeyCode::G,
			VizColour::Red => KeyCode::R,
			VizColour::Blue => KeyCode::B,
			VizColour::Orange => KeyCode::O,
			VizColour::Invisible => KeyCode::I,
		}
	}
}
