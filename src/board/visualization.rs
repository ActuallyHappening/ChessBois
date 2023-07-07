use super::{
	compute::{begin_background_compute, ComputationResult},
	*,
};
use crate::solver::{algs::Computation, pieces::StandardKnight, Move, Moves};

#[derive(Component, Debug, Clone)]
pub struct VisualizationComponent {
	from: ChessPoint,
	to: ChessPoint,
}

/// Consumes [EventReader<ComputationResult>] and actually spawns concrete visualization if state is correct
pub fn handle_spawning_visualization(
	mut commands: Commands,
	mut solutions: EventReader<ComputationResult>,
	current_options: Res<CurrentOptions>,

	viz: Query<Entity, With<VisualizationComponent>>,

	mut mma: ResSpawning,
) {
	if let Some(solution) = solutions.iter().next() {
		let (solution, options) = solution.clone().get();
		if &options != current_options.as_options() {
			// warn!("Not rendering visualization for computation of non-valid state");
			return;
		}

		if let Computation::Successful {
			solution: moves, ..
		} = solution
		{
			spawn_visualization(moves, options.options, &mut commands, &mut mma);
		}

		solutions.clear()
	}
}

/// Actually spawn entities of new solution
pub fn spawn_visualization(
	moves: Moves,
	options: BoardOptions,
	commands: &mut Commands,
	mma: &mut ResSpawning,
) {
	for Move { from, to } in moves.iter() {
		spawn_path_line(
			from,
			to,
			&options,
			VISUALIZATION_SELECTED_COLOUR,
			commands,
			mma,
		)
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

fn spawn_path_line(
	from: &ChessPoint,
	to: &ChessPoint,
	options: &BoardOptions,
	colour: Color,

	commands: &mut Commands,
	// meshes: &mut ResMut<Assets<Mesh>>,
	// materials: &mut ResMut<Assets<StandardMaterial>>,
	mma: &mut ResSpawning,
) {
	let start_pos = get_spacial_coord_2d(options, *from);
	let end_pos = get_spacial_coord_2d(options, *to);

	let center = (start_pos + end_pos) / 2.; // ✅
	let length = (start_pos - end_pos).length(); // ✅
	let angle: f32 = -(start_pos.y - end_pos.y).atan2(start_pos.x - end_pos.x);

	// assert_eq!(angle, TAU / 8., "Drawing from {from} [{from:?}] [{from_pos}] to {to} [{to:?}] [{to_pos}], Angle: {angle}, 𝚫y: {}, 𝚫x: {}", (to_pos.y - from_pos.y), (to_pos.x - from_pos.x));
	// info!("Angle: {angle}, {}", angle.to_degrees());

	let transform = Transform::from_translation(Vec3::new(center.x, VISUALIZATION_HEIGHT, center.y))
		.with_rotation(Quat::from_rotation_y(angle));

	// info!("Transform: {:?}", transform);
	// info!("Angle: {:?}, Length: {:?}", angle, length);

	let mesh_thin_rectangle = mma.0.add(
		shape::Box::new(
			length,
			VISUALIZATION_DIMENSIONS.x,
			VISUALIZATION_DIMENSIONS.y,
		)
		.into(),
	);

	commands.spawn((
		PbrBundle {
			mesh: mesh_thin_rectangle,
			material: mma.1.add(colour.into()),
			transform,
			..default()
		},
		VisualizationComponent {
			from: *from,
			to: *to,
		},
	));
}
