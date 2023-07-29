use super::*;

pub use freedom::ManualFreedom;
mod freedom;

pub struct ManualState;
impl Plugin for ManualState {
	fn build(&self, app: &mut App) {
		app;
	}
}
