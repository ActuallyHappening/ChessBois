use std::f32::consts::PI;

use bevy::prelude::*;

pub fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	// cam
	commands.spawn(Camera3dBundle {
		transform: Transform::from_xyz(0., 25., 0.).with_rotation(Quat::from_rotation_x(-PI / 2.)),
		..default()
	});

	// light
	commands.spawn(PointLightBundle {
		point_light: PointLight {
			intensity: 1000.0,
			range: 250.,
			// shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(0., 25., 0.),
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
