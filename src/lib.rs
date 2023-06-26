use bevy::prelude::*;
use bevy_egui::egui::Color32;
use bevy_mod_picking::{prelude::{RaycastPickCamera, RaycastPickTarget, OnPointer, Click}, PickableBundle};
mod board;
use board::*;

#[derive(Default)]
pub struct ChessSolverPlugin;
impl Plugin for ChessSolverPlugin {
	fn build(&self, app: &mut App) {
		app.add_startup_system(setup).add_plugin(BoardPlugin);
	}
}

const CAMERA_HEIGHT: f32 = 75.;
const LIGHT_HEIGHT: f32 = CAMERA_HEIGHT;

/// Square width and height
const CELL_SIZE: f32 = 5.;
/// Distance from ground plane, y = 0
const CELL_HEIGHT: f32 = 1.;
/// Depth of cell
const CELL_DEPTH: f32 = 2.;
const CELL_SELECTED_COLOUR: Color = Color::PURPLE;
const CELL_DISABLED_COLOUR: Color = Color::RED;

const VISUALIZATION_HEIGHT: f32 = 3.;
const VISUALIZATION_DIMENSIONS: Vec2 = Vec2::new(0.2, 0.2);
const VISUALIZATION_SELECTED_COLOUR: Color = Color::GREEN;
const VISUALIZATION_ALL_BASE_COLOUR: Color = Color::Rgba {
	red: 0.,
	green: 1.,
	blue: 0.1,
	alpha: 0.5,
};

const UI_ALG_ENABLED_COLOUR: Color32 = Color32::GREEN;

pub fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	// cam
	commands.spawn((
		Camera3dBundle {
			transform: Transform::from_xyz(0., CAMERA_HEIGHT, 0.)
				.with_rotation(Quat::from_rotation_x(-90_f32.to_radians())),
			..default()
		},
		RaycastPickCamera::default(),
	));

	// light
	commands.spawn(PointLightBundle {
		point_light: PointLight {
			intensity: 50000.0,
			range: 250.,
			// shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(0., LIGHT_HEIGHT, 0.),
		..default()
	});

	// ground plane
	commands.spawn((
		PbrBundle {
			mesh: meshes.add(shape::Plane::from_size(500.0).into()),
			material: materials.add(Color::SILVER.into()),
			// transform to be behind, xy plane
			transform: Transform::from_xyz(0., 0., 0.),
			..default()
		},
		PickableBundle::default(),    // Makes the entity pickable
		RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
		OnPointer::<Click>::run_callback(handle_plane_clicked),
	));
}
