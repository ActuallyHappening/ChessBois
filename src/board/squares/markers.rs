use std::sync::Mutex;

use crate::solver::algs::Computation;

use super::*;

/// Marker for Markers lol
#[derive(Component)]
pub struct MarkerMarker;

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

static PREVIOUS_RENDER: Mutex<Option<OwnedMarkersState>> = Mutex::new(None);
impl SharedState {
	pub fn sys_render_markers(
		state: Res<SharedState>,

		mut commands: Commands,
		markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,
		mut mma: ResSpawning,
	) {
		let state = state.into_inner();
		let owned_state = OwnedMarkersState::new(state.clone());
		if *PREVIOUS_RENDER.lock().unwrap() != Some(owned_state.clone()) {
			despawn_markers(&mut commands, markers);
			spawn_markers(&BorrowedMarkersState::new(state), &mut commands, &mut mma);

			*PREVIOUS_RENDER.lock().unwrap() = Some(owned_state);
		} else {
			// info!("Skipping marker re-render");
		}
	}
}


use markers_state::*;
mod markers_state {
	use crate::{board::squares::visualization::VisualOpts, solver::algs::OwnedComputeInput};

use super::*;

	pub struct BorrowedMarkersState<'shared> {
		pub board_options: &'shared BoardOptions,
		pub visual_opts: &'shared VisualOpts,
		pub alg: &'shared Algorithm,
		pub start: &'shared Option<ChessPoint>,
		pub piece: &'shared StandardPieces,
		pub safety_cap: &'shared SafteyCap,
	}

	/// Used to store for later comparisons
	#[derive(PartialEq, Clone)]
	pub struct OwnedMarkersState {
		pub board_options: BoardOptions,
		pub visual_opts: VisualOpts,
		pub alg: Algorithm,
		pub start: Option<ChessPoint>,
		pub piece: StandardPieces,
		pub safety_cap: SafteyCap,
	}

	impl<'shared> BorrowedMarkersState<'shared> {
		pub fn new(state: &'shared SharedState) -> Self {
			Self {
				board_options: &state.board_options,
				visual_opts: &state.visual_opts,
				alg: &state.alg,
				start: &state.start,
				piece: &state.piece,
				safety_cap: &state.safety_cap,
			}
		}

		pub fn clone_into_compute_with_start(&self, start: ChessPoint) -> OwnedComputeInput {
			OwnedComputeInput {
				alg: *self.alg,
				start,
				board_options: self.board_options.clone(),
				piece: (*self.piece).into(),
				safety_cap: self.safety_cap.clone().into(),
			}
		}
	}

	impl OwnedMarkersState {
		pub fn new(state: SharedState) -> Self {
			Self {
				board_options: state.board_options,
				visual_opts: state.visual_opts,
				alg: state.alg,
				start: state.start,
				piece: state.piece,
				safety_cap: state.safety_cap,
			}
		}

		pub fn borrow(&self) -> BorrowedMarkersState {
			BorrowedMarkersState {
				board_options: &self.board_options,
				visual_opts: &self.visual_opts,
				alg: &self.alg,
				start: &self.start,
				piece: &self.piece,
				safety_cap: &self.safety_cap,
			}
		} 
	}

	impl std::ops::Deref for BorrowedMarkersState<'_> {
		type Target = BoardOptions;

		fn deref(&self) -> &Self::Target {
			self.board_options
		}
	}
}

fn spawn_markers(state: &BorrowedMarkersState, commands: &mut Commands, mma: &mut ResSpawning) {
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
	state: &BorrowedMarkersState,
	cell_transform: Transform,

	commands: &mut Commands,
	(meshes, materials, ass): &mut ResSpawning,
) {
	if !state.visual_opts.show_markers {
		return;
	}

	if let Some(mark) = compute::get_cached_mark(&state.clone_into_compute_with_start(at)) {
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
