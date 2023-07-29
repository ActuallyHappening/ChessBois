use bevy_egui_controls::ControlPanel;

use super::*;

/// Used in cells module
#[derive(Clone, ControlPanel)]
pub struct VisualOpts {
	/// Whether to show the red numbers
	#[control(bool)]
	pub show_numbers: bool,

	/// Whether to show the helper dots
	#[control(bool)]
	pub show_dots: bool,

	/// Whether to show the markers
	#[control(bool)]
	pub show_markers: bool,

	/// Whether to show the visualisation lines at all [h]
	#[control(bool)]
	pub show_visualisation: bool,

	/// Whether to hightlight last move in blue
	#[control(bool)]
	pub show_end_colour: bool,

	/// The width of the visualisation lines
	#[control(slider(0.1..=0.5))]
	viz_width: f32,
}

impl VisualOpts {
	pub const DEFAULT: Self = VisualOpts {
		show_numbers: true,
		show_dots: true,
		show_markers: true,
		show_visualisation: true,
		show_end_colour: true,
		viz_width: 0.2,
	};

	pub fn dimensions(&self) -> Vec2 {
		Vec2::new(self.viz_width, self.viz_width)
	}
}

impl Default for VisualOpts {
	fn default() -> Self {
		Self::DEFAULT
	}
}
