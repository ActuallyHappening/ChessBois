use strum::{Display, EnumIter};

use super::*;

#[derive(Resource, Copy, Clone, Default, Display, EnumIter, PartialEq, Eq, Debug)]
pub enum VizColour {
	#[default]
	Green,

	Red,
	Blue,
	Orange,

	Invisible,
}

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