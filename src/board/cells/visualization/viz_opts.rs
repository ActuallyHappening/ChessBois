use super::*;

#[derive(Clone)]
pub struct VisualOpts {
	pub show_numbers: bool,
	pub show_dots: bool,
	pub show_markers: bool,
	
	viz_width: f32,
}

impl VisualOpts {
	pub const DEFAULT: Self = VisualOpts {
		show_numbers: true,
		show_dots: true,
		show_markers: true,
		viz_width: 0.2,
	};

	pub fn dimensions(&self) -> Vec2 {
		Vec2::new(self.viz_width, self.viz_width)
	}

	pub fn set_width(&mut self, viz_width: f32) {
		match viz_width {
			_x if (0.1..=0.5).contains(&_x) => self.viz_width = viz_width,
			_ => {
				warn!("Setting viz_width to unnacceptable value: {viz_width}");
			}
		}
	}
}

impl Default for VisualOpts {
	fn default() -> Self {
		Self::DEFAULT
	}
}
