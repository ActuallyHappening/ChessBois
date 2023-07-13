use std::f32::consts::TAU;

use super::{cells::get_spacial_coord_2d, viz_colours::VizColour, *};
use crate::{
	solver::{Move, Moves},
	textmesh::{get_text_mesh, Fonts},
	utils::{EntityCommandsExt, TransformExt},
	ChessPoint, CELL_SIZE, VISUALIZATION_DIMENSIONS, VISUALIZATION_HEIGHT,
};

#[derive(Component, Debug, Clone)]
pub struct VisualizationComponent {
	from: ChessPoint,
	to: ChessPoint,
}

/// Actually spawn entities of new solution
pub fn spawn_visualization(
	moves: Moves,
	options: BoardOptions,
	commands: &mut Commands,
	mma: &mut ResSpawning,
	viz_cols: Vec<VizColour>,
) {
	for (i, Move { from, to }) in moves.iter().enumerate() {
		let colour = (*viz_cols.get(i).expect("Colour to have index")).into();
		spawn_path_line(from, to, &options, colour, i + 1, commands, mma)
	}
}

pub fn despawn_visualization(
	commands: &mut Commands,
	visualization: Query<Entity, With<VisualizationComponent>>,
) {
	for entity in visualization.iter() {
		commands.entity(entity).despawn_recursive();
	}
}

pub fn sys_despawn_visualization(
	mut commands: Commands,
	visualization: Query<Entity, With<VisualizationComponent>>,
) {
	super::visualization::despawn_visualization(&mut commands, visualization);
}

fn spawn_path_line(
	from: &ChessPoint,
	to: &ChessPoint,
	options: &BoardOptions,
	colour: Color,
	number: usize,

	commands: &mut Commands,
	(meshs, mat, _ass): &mut ResSpawning,
) {
	let start_pos = get_spacial_coord_2d(options, *from);
	let end_pos = get_spacial_coord_2d(options, *to);
	let start_vec = &Vec3::new(start_pos.x, VISUALIZATION_HEIGHT * 1.1, start_pos.y);

	let center = (start_pos + end_pos) / 2.; // ✅
	let length = (start_pos - end_pos).length(); // ✅
	let angle: f32 = -(start_pos.y - end_pos.y).atan2(start_pos.x - end_pos.x);

	// assert_eq!(angle, TAU / 8., "Drawing from {from} [{from:?}] [{from_pos}] to {to} [{to:?}] [{to_pos}], Angle: {angle}, 𝚫y: {}, 𝚫x: {}", (to_pos.y - from_pos.y), (to_pos.x - from_pos.x));
	// info!("Angle: {angle}, {}", angle.to_degrees());

	let center = &Vec3::new(center.x, VISUALIZATION_HEIGHT, center.y);
	let transform = Transform::from_translation(*center).with_rotation(Quat::from_rotation_y(angle));

	// info!("Transform: {:?}", transform);
	// info!("Angle: {:?}, Length: {:?}", angle, length);

	let mesh_thin_rectangle = meshs.add(
		shape::Box::new(
			length,
			VISUALIZATION_DIMENSIONS.x,
			VISUALIZATION_DIMENSIONS.y,
		)
		.into(),
	);

	let material = mat.add(colour.into());
	commands.spawn((
		PbrBundle {
			mesh: mesh_thin_rectangle,
			material: material.clone(),
			transform,
			..default()
		},
		VisualizationComponent {
			from: *from,
			to: *to,
		},
	));

	// small dot at start
	let start_transform =
		Transform::from_translation(*start_vec).with_rotation(Quat::from_rotation_y(angle));
	commands
		.spawn(PbrBundle {
			transform: start_transform,
			material,
			mesh: meshs.add(
				shape::Icosphere {
					radius: VISUALIZATION_DIMENSIONS.length(),
					subdivisions: 1,
				}
				.try_into()
				.unwrap(),
			),
			..default()
		})
		.insert(VisualizationComponent {
			from: *from,
			to: *to,
		});

	// text
	let text = format!("{}", number);
	let (mesh, offset) = get_text_mesh(text, CELL_SIZE / 4., Fonts::Light);

	commands
		.spawn((
			PbrBundle {
				mesh: meshs.add(mesh),
				transform: Transform::from_translation(*start_vec)
					.translate(offset)
					.translate(Vec3::X * CELL_SIZE / 4. + Vec3::Y * 1.)
					.with_rotation(Quat::from_rotation_x(-TAU / 4.)),
				material: mat.add(Color::RED.into()),
				..default()
			},
			VisualizationComponent {
				from: *from,
				to: *to,
			},
		))
		.name(format!("Number for {}", *from));
}
