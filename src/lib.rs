use bevy::prelude::*;
use bevy_mod_picking::prelude::RaycastPickCamera;
use std::f32::consts::PI;
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
const VISUALIZATION_COLOUR: Color = Color::GREEN;

pub fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	// cam
	commands.spawn((
		Camera3dBundle {
			transform: Transform::from_xyz(0., CAMERA_HEIGHT, 0.)
				.with_rotation(Quat::from_rotation_x(-PI / 2.)),
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
	commands.spawn(PbrBundle {
		mesh: meshes.add(shape::Plane::from_size(500.0).into()),
		material: materials.add(Color::SILVER.into()),
		// transform to be behind, xy plane
		transform: Transform::from_xyz(0., 0., 0.),
		..default()
	});
}
