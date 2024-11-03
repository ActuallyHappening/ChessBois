use super::*;
use crate::{
	board::coloured_moves::ColouredMoves,
	solver::Move,
	textmesh::{get_text_mesh, Fonts},
	utils::{EntityCommandsExt, TransformExt},
	ChessPoint, CELL_SIZE, VISUALIZATION_HEIGHT,
};
use std::{f32::consts::TAU, sync::Mutex};

pub use viz_colours::*;
pub use viz_opts::VisualOpts;

mod viz_colours;
mod viz_opts;

/// The lines from one cell to another to show the knights movements, or the
/// moves that the knight should take
#[derive(Component, Debug, Clone, Copy)]
pub enum VisComponent {
	Move {
		from: ChessPoint,
		to: ChessPoint,

		colour: Color,
		number: usize,
	},
	RecommendedMove {
		from: ChessPoint,
		to: ChessPoint,

		number: usize,
	},
}

impl VisComponent {
	fn from(&self) -> &ChessPoint {
		match self {
			VisComponent::Move { from, .. } => from,
			VisComponent::RecommendedMove { from, .. } => from,
		}
	}

	fn to(&self) -> &ChessPoint {
		match self {
			VisComponent::Move { to, .. } => to,
			VisComponent::RecommendedMove { to, .. } => to,
		}
	}
}

static PREVIOUS_RENDER: Mutex<Option<OwnedVisState>> = Mutex::new(None);
impl SharedState {
	pub fn sys_render_viz(
		state: Res<SharedState>,
		visualization: Query<Entity, With<VisComponent>>,

		mut commands: Commands,
		mut mma: ResSpawning,
	) {
		let state = state.into_inner();

		if *PREVIOUS_RENDER.lock().unwrap() != Some(OwnedVisState::clone_new(state)) {
			despawn_visualization(&mut commands, visualization);

			if state.visual_opts.show_visualisation {
				spawn_visualization(
					state.moves.clone(),
					state.board_options.clone(),
					&state.visual_opts,
					&mut commands,
					&mut mma,
				);
			}

			*PREVIOUS_RENDER.lock().unwrap() = Some(OwnedVisState::clone_new(state));
		} else {
			// info!("Skipping visualization re-render");
		}
	}
}

use vis_state::*;
mod vis_state {
	use super::*;

	#[derive(PartialEq, Clone)]
	pub struct OwnedVisState {
		pub moves: Option<ColouredMoves>,
		pub board_options: BoardOptions,
		pub visual_opts: VisualOpts,
	}

	impl OwnedVisState {
		pub fn clone_new(state: &SharedState) -> Self {
			Self {
				moves: state.moves.clone(),
				board_options: state.board_options.clone(),
				visual_opts: state.visual_opts.clone(),
			}
		}
	}
}

/// Actually spawn entities of new solution every frame
fn spawn_visualization(
	moves: Option<ColouredMoves>,
	options: BoardOptions,
	viz_options: &VisualOpts,

	commands: &mut Commands,
	mma: &mut ResSpawning,
) {
	if let Some(moves) = moves {
		for (i, (Move { from, to }, colour)) in moves.iter().enumerate() {
			spawn_path_line(
				VisComponent::Move {
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
	for (i, Move { from, to }) in options.recommended_moves().iter().enumerate() {
		spawn_path_line(
			VisComponent::RecommendedMove {
				from: *from,
				to: *to,
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
	// let VisComponent {
	// 	from,
	// 	to,
	// 	colour,
	// 	number,
	// } = vis;
	let from = *vis.from();
	let to = *vis.to();

	let start_pos = get_spacial_coord_2d(options, from);
	let end_pos = get_spacial_coord_2d(options, to);
	let start_vec = &Vec3::new(start_pos.x, VISUALIZATION_HEIGHT * 1.1, start_pos.y);
	let _end_vec = Vec3::new(end_pos.x, VISUALIZATION_HEIGHT * 1.1, end_pos.y);

	let center = (start_pos + end_pos) / 2.; // ✅
	let mut length = (start_pos - end_pos).length(); // ✅
	if matches!(vis, VisComponent::RecommendedMove { .. }) {
		length *= 0.7;
	}
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

	let material = match vis {
		VisComponent::Move { colour, number, .. } => mat.add({
			let mut mat: StandardMaterial = colour.into();
			mat.depth_bias = number as f32;
			mat
		}),
		VisComponent::RecommendedMove { number, .. } => mat.add({
			let mut mat: StandardMaterial = Color::YELLOW.with_a(0.7).into();
			mat.depth_bias = number as f32 + 10000.0;
			mat
		}),
	};
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
	if viz_options.show_dots && matches!(vis, VisComponent::Move { .. }) {
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
	if let VisComponent::Move { number, .. } = vis {
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
}
