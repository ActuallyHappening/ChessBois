use bevy_egui_controls::ControlPanel;

use crate::MainCamera;

use super::*;

pub struct CamZoomPlugin;
impl Plugin for CamZoomPlugin {
	fn build(&self, app: &mut App) {
		app.add_system(zoom_camera);
	}
}

/// start height
pub const CAMERA_HEIGHT: f32 = 75.;

const MAX_HEIGHT: f32 = 150.;
const MIN_HEIGHT: f32 = 50.;

#[derive(PartialEq, Clone, ControlPanel, Reflect, FromReflect)]
pub struct CameraZoom {
	/// Zoom of camera
	#[control(slider(MIN_HEIGHT..=MAX_HEIGHT))]
	height: f32,
}

impl CameraZoom {
	pub const DEFAULT: Self = Self {
		height: CAMERA_HEIGHT,
	};
}

impl Default for CameraZoom {
	fn default() -> Self {
		Self::DEFAULT
	}
}

pub fn zoom_camera(state: Res<SharedState>, mut query: Query<&mut Transform, With<MainCamera>>) {
	query.single_mut().translation.y = state.cam_zoom.height;
}