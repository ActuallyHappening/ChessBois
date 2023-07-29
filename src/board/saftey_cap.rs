use bevy_egui::egui::{self, Ui};
use derive_more::Into;

#[derive(PartialEq, Eq, Clone, Into)]
pub struct SafteyCap {
	/// Maximum amount of states considered before giving up
	cap: u128,
}

const MIN: u128 = 10;
const MAX: u128 = 1000000;
const DEFAULT: u128 = 6969;

impl SafteyCap {
	pub fn ui(&mut self, ui: &mut Ui) {
		ui.add(
			egui::Slider::from_get_set((MIN as f64)..=(MAX as f64), |val| {
				if let Some(val) = val {
					self.cap = val as u128;
				}
				self.cap as f64
			})
			.text("Safety cap")
			.logarithmic(true),
		);
	}
}

impl Default for SafteyCap {
	fn default() -> Self {
		Self { cap: DEFAULT }
	}
}
