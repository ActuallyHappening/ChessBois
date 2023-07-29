use crate::solver::algs::{Computation};

use super::*;

#[derive(Clone, Copy)]
pub enum CellMark {
	Failed,
	Succeeded,

	GivenUp,
}

impl From<Computation> for CellMark {
	fn from(c: Computation) -> Self {
		match c {
			Computation::Failed { .. } => CellMark::Failed,
			Computation::Successful { .. } => CellMark::Succeeded,
			Computation::GivenUp { .. } => CellMark::GivenUp,
		}
	}
}

impl SharedState {
	pub fn sys_render_markers(
		state: Res<SharedState>,

		mut commands: Commands,
		markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,
		mut mma: ResSpawning,
	) {
		despawn_markers(&mut commands, markers);
		spawn_markers(&state, &mut commands, &mut mma);
	}
}

fn spawn_markers(state: &SharedState, commands: &mut Commands, mma: &mut ResSpawning) {
	for point in state.get_all_points() {
		spawn_mark(
			point,
			state,
			cell_get_transform(point, state),
			commands,
			mma,
		);
	}
}

fn despawn_markers(
	commands: &mut Commands,
	markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,
) {
	for mark in markers.iter() {
		commands.entity(mark).despawn_recursive();
	}
}

fn spawn_mark(
	at: ChessPoint,
	state: &SharedState,
	cell_transform: Transform,

	commands: &mut Commands,
	(meshes, materials, ass): &mut ResSpawning,
) {
	if !state.visual_opts.show_markers {
		return;
	}

	if let Some(mark) = compute::get_cached_mark(&state.clone().into_compute_state_with_start(at)) {
		let quad = shape::Quad::new(Vec2::new(CELL_SIZE, CELL_SIZE) * 0.7);
		let mesh = meshes.add(Mesh::from(quad));

		let mut transform = cell_transform;
		transform.translation += Vec3::Y * CELL_DEPTH / 2.;

		let asset_path = format!(
			"images/{}.png",
			match mark {
				CellMark::Succeeded => "TickMark",
				CellMark::Failed => "XMark",
				CellMark::GivenUp => "WarningMark",
			}
		);
		let material_handle = materials.add(StandardMaterial {
			base_color_texture: Some(ass.load(asset_path)),
			alpha_mode: AlphaMode::Blend,
			..default()
		});
		commands.spawn((
			PbrBundle {
				mesh,
				material: material_handle,
				transform,
				..default()
			},
			at,
			MarkerMarker {},
		));
	}
}
