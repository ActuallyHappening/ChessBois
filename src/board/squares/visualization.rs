use super::*;
use crate::{
	board::coloured_moves::ColouredMoves,
	solver::Move,
	textmesh::{get_text_mesh, Fonts},
	utils::{EntityCommandsExt, TransformExt},
	ChessPoint, CELL_SIZE, VISUALIZATION_HEIGHT,
};
use std::f32::consts::TAU;

pub use colours::*;
pub use viz_opts::VisualOpts;

mod colours;
mod viz_opts;

#[derive(Component, Debug, Clone, Copy)]
pub struct VisComponent {
	from: ChessPoint,
	to: ChessPoint,

	colour: Color,
	number: usize,
}

impl SharedState {
	pub fn sys_render_viz(
		state: Res<SharedState>,
		visualization: Query<Entity, With<VisComponent>>,

		mut commands: Commands,
		mut mma: ResSpawning,
	) {
		despawn_visualization(&mut commands, visualization);
		if !state.visual_opts.show_visualisation {
			return;
		}
		if let Some(moves) = &state.moves {
			spawn_visualization(
				moves.clone(),
				state.board_options.clone(),
				&state.visual_opts,
				&mut commands,
				&mut mma,
			);
		}
	}
}

/// Actually spawn entities of new solution
fn spawn_visualization(
	moves: ColouredMoves,
	options: BoardOptions,
	viz_options: &VisualOpts,

	commands: &mut Commands,
	mma: &mut ResSpawning,
) {
	for (i, (Move { from, to }, colour)) in moves.iter().enumerate() {
		spawn_path_line(
			VisComponent {
				from: *from,
				to: *to,
				colour: (*colour).into(),
				number: i,
			},
			&options,
			viz_options,
			commands,
			mma,
		)
	}
}

fn despawn_visualization(
	commands: &mut Commands,
	visualization: Query<Entity, With<VisComponent>>,
) {
	for entity in visualization.iter() {
		commands.entity(entity).despawn_recursive();
	}
}

fn spawn_path_line(
	vis: VisComponent,
	options: &BoardOptions,
	viz_options: &VisualOpts,

	commands: &mut Commands,
	(meshs, mat, _ass): &mut ResSpawning,
) {
	let VisComponent {
		from,
		to,
		colour,
		number,
	} = vis;
	let start_pos = get_spacial_coord_2d(options, from);
	let end_pos = get_spacial_coord_2d(options, to);
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

	let material = mat.add({
		let mut mat: StandardMaterial = colour.into();
		mat.depth_bias = number as f32;
		mat
	});
	commands.spawn((
		PbrBundle {
			mesh: mesh_thin_rectangle,
			material: material.clone(),
			transform,
			..default()
		},
		vis,
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
			.insert(vis);
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
				vis,
			))
			.name(format!("Number for {}", from));
	}
}
