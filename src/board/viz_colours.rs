use strum::{Display, EnumIter};

use super::*;

#[derive(Resource, Default, Display, EnumIter, PartialEq, Eq)]
pub enum ManualVizColour {
	#[default]
	Green,

	Red,
	Blue,
	Orange,
}

impl From<ManualVizColour> for Color {
	fn from(colour: ManualVizColour) -> Self {
		match colour {
			ManualVizColour::Green => Color::GREEN,
			ManualVizColour::Red => Color::RED,
			ManualVizColour::Blue => Color::BLUE,
			ManualVizColour::Orange => Color::ORANGE,
		}
	}
}