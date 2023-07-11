use strum::{Display, EnumIter, IntoEnumIterator};

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

impl From<VizColour> for KeyCode {
	fn from(value: VizColour) -> Self {
		match value {
			VizColour::Green => KeyCode::G,
			VizColour::Blue => KeyCode::B,
			VizColour::Invisible => KeyCode::I,
			VizColour::Orange => KeyCode::O,
			VizColour::Red => KeyCode::R,
		}
	}
}

pub fn colour_hotkeys(keys: Res<Input<KeyCode>>, mut viz_col: ResMut<VizColour>) {
	for key in VizColour::iter() {
		if keys.just_pressed(KeyCode::from(key)) {
			*viz_col = key;
		}
	}
}