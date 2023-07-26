use super::*;
use crate::{
	solver::{Move, Moves},
	textmesh::{get_text_mesh, Fonts},
	utils::{EntityCommandsExt, TransformExt},
	ChessPoint, CELL_SIZE, VISUALIZATION_HEIGHT,
};
use std::f32::consts::TAU;

pub use colours::*;
pub use viz_opts::VisualOpts;

mod colours;
mod viz_opts;

#[derive(Component, Debug, Clone)]
pub struct VisualizationComponent {
	#[allow(dead_code)]
	from: ChessPoint,

	#[allow(dead_code)]
	to: ChessPoint,
}

impl SharedState {
	pub fn sys_render_viz() {
		todo!()
	}
}

/// Actually spawn entities of new solution
fn spawn_visualization(
	moves: Moves,
	options: BoardOptions,
	commands: &mut Commands,
	mma: &mut ResSpawning,
	viz_cols: Vec<VizColour>,
	viz_options: &VisualOpts,
) {
	for (i, Move { from, to }) in moves.iter().enumerate() {
		let colour = (*viz_cols.get(i).expect("Colour to have index")).into();
		spawn_path_line(from, to, &options, viz_options, colour, i, commands, mma)
	}
}

fn despawn_visualization(
	commands: &mut Commands,
	visualization: Query<Entity, With<VisualizationComponent>>,
) {
	for entity in visualization.iter() {
		commands.entity(entity).despawn_recursive();
	}
}

fn spawn_path_line(
	from: &ChessPoint,
	to: &ChessPoint,
	options: &BoardOptions,
	viz_options: &VisualOpts,
	colour: Color,
	number: usize,

	commands: &mut Commands,
	(meshs, mat, _ass): &mut ResSpawning,
) {
	let start_pos = get_spacial_coord_2d(options, *from);
	let end_pos = get_spacial_coord_2d(options, *to);
	let start_vec = &Vec3::new(start_pos.x, VISUALIZATION_HEIGHT * 1.1, start_pos.y);
	let _end_vec = Vec3::new(end_pos.x, VISUALIZATION_HEIGHT * 1.1, end_pos.y);

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
			viz_options.dimensions().x,
			viz_options.dimensions().y,
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
	if viz_options.show_dots {
		let start_transform =
			Transform::from_translation(*start_vec).with_rotation(Quat::from_rotation_y(angle));
		commands
			.spawn(PbrBundle {
				transform: start_transform,
				material,
				mesh: meshs.add(
					shape::Icosphere {
						radius: viz_options.dimensions().length(),
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
	}

	// text
	if viz_options.show_numbers {
		let text = format!("{}", number);
		let (mesh, offset) = get_text_mesh(text, CELL_SIZE / 3., Fonts::Light);

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
}
