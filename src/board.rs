use crate::{
	board::manual::UnstableSavedState,
	solver::{algs::Algorithm, pieces::StandardPieces, BoardOptions, Moves},
	ChessPoint, ProgramState,
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

pub use cam_zoom::CAMERA_HEIGHT;
pub use hotkeys::Hotkeyable;
use serde_json::Value;

mod automatic;
mod cam_zoom;
mod coloured_moves;
mod compute;
mod hotkeys;
mod manual;
mod saftey_cap;
mod shared;
mod squares;
mod ui;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
	fn build(&self, app: &mut App) {
		app
			// .add_plugin(VisualizationPlugin)
			.add_plugin(UiPlugin)
			.add_plugin(AutomaticPlugin)
			.add_plugin(ManualState)
			.add_plugin(SharedPlugin)
			.add_plugin(HotkeysPlugin)
			.add_plugin(SharedState::default())
			.add_plugin(SquaresPlugin)
			.add_plugin(CamZoomPlugin)
			.add_plugins(
				DefaultPickingPlugins
					.build()
					.disable::<DefaultHighlightingPlugin>()
					.disable::<DebugPickingPlugin>(),
			)
			.register_type::<SharedState>()
			.add_startup_system(setup);
	}
}

/// Re-rendered every frame
#[derive(Resource, Default, Clone, Reflect)]
#[reflect(Resource)]
#[non_exhaustive]
pub struct SharedState {
	// inputs
	/// Set using [set_alg]
	pub alg: Algorithm,
	pub safety_cap: SafteyCap,

	/// Set using [set_board_options]
	pub board_options: BoardOptions,

	/// Set using [set_start]
	pub start: Option<ChessPoint>,
	pub piece: StandardPieces,

	// visuals
	pub moves: Option<ColouredMoves>,
	pub visual_opts: squares::visualization::VisualOpts,
	pub cell_colouring: squares::CellColouring,
	pub cam_zoom: CameraZoom,

	// ui / interactions
	// auto
	pub on_click: ToggleAction,

	// manual
	pub manual_freedom: ManualFreedom,
	pub save_state: SaveState,
	/// Colour of next move
	pub viz_colour: VizColour,

	// web vis
	pub is_web_vis_first_render: bool,
	pub web_vis: Option<manual::MetaData>,
}

pub use self::shared_state::StateInvalidated;
mod shared_state {
	use super::{squares::visualization, *};
	use crate::solver::algs::OwnedComputeInput;

	impl Plugin for SharedState {
		fn build(&self, app: &mut App) {
			app.add_systems((
				SharedState::sys_render_cells,
				SharedState::sys_render_viz,
				SharedState::sys_render_markers,
			));
		}
	}

	#[must_use = "Not using this skips invalidating state, use .invalidates(state) to fix"]
	pub enum StateInvalidated {
		Invalidated,
		InvalidatedAndClearStart,
		Valid,
	}

	impl std::ops::Deref for SharedState {
		type Target = BoardOptions;

		fn deref(&self) -> &Self::Target {
			&self.board_options
		}
	}

	impl std::ops::DerefMut for SharedState {
		fn deref_mut(&mut self) -> &mut Self::Target {
			&mut self.board_options
		}
	}

	impl TryFrom<SharedState> for OwnedComputeInput {
		type Error = ();

		fn try_from(value: SharedState) -> Result<Self, Self::Error> {
			value.get_compute_state().ok_or(())
		}
	}

	impl SharedState {
		pub fn get_compute_state(self) -> Option<OwnedComputeInput> {
			Some(OwnedComputeInput {
				alg: self.alg,
				start: self.start?,
				board_options: self.board_options,
				piece: self.piece.into(),
				safety_cap: self.safety_cap.into(),
			})
		}

		/// Gets the [ComputeInput] from [SharedState] guarenteed given a start point.
		/// Used to 'imagine' starting on a square
		pub fn into_compute_state_with_start(self, start: ChessPoint) -> OwnedComputeInput {
			OwnedComputeInput {
				alg: self.alg,
				start,
				board_options: self.board_options,
				piece: self.piece.into(),
				safety_cap: self.safety_cap.into(),
			}
		}

		// doesn't invalidate
		pub fn remove_start(&mut self) -> &mut Self {
			self.start = None;
			self
		}

		pub fn set_coloured_moves(&mut self, moves: ColouredMoves) -> &mut Self {
			self.moves = Some(moves);
			self
		}

		/// Sets moves using default colour
		/// suitable in automatic mode
		pub fn set_moves(&mut self, moves: Moves) -> &mut Self {
			self.set_coloured_moves(moves.using_colour(visualization::VizColour::default()));
			self
		}
	}

	impl StateInvalidated {
		pub fn invalidates(self, state: &mut SharedState) {
			if matches!(
				self,
				StateInvalidated::Invalidated | StateInvalidated::InvalidatedAndClearStart
			) {
				state.invalidate();
			}
			if let StateInvalidated::InvalidatedAndClearStart = self {
				state.start = None;
			}
		}
	}
}

use self::{
	automatic::{AutomaticPlugin, ToggleAction},
	cam_zoom::{CamZoomPlugin, CameraZoom},
	coloured_moves::ColouredMoves,
	hotkeys::HotkeysPlugin,
	manual::{ManualFreedom, ManualState, SaveState},
	saftey_cap::SafteyCap,
	shared::SharedPlugin,
	squares::{visualization::VizColour, SquaresPlugin},
	ui::UiPlugin,
};

/// Sets up default resources + sends initial [NewOptions] event
fn setup(
	mut commands: Commands,
	data: Option<Res<crate::weburl::InitialLoadedID>>,
	mut to_manual: ResMut<NextState<ProgramState>>,
) {
	let mut state = SharedState::default();

	if let Some(data) = data {
		let data: serde_json::Value = data.into_inner().clone().into();
		match data {
			Value::String(data) => {
				info!("App main running with data json: {:?}", data);
				match UnstableSavedState::from_json(&data) {
					Ok(data) => {
						to_manual.set(ProgramState::Manual);
						info!("Loaded data: {:?}", data);
						data.apply_to_state(&mut state);

						info!("State now contains {} moves", state.moves.clone().unwrap().len());
					}
					Err(err) => {
						error!("Failed to load data: {:?}", err);
					}
				}
			}
			_ => {
				error!("No data with that ID found");
			}
		}
	} else {
		info!("App running without data");
	}

	commands.insert_resource(state);
}

type ResSpawning<'a> = (
	ResMut<'a, Assets<Mesh>>,
	ResMut<'a, Assets<StandardMaterial>>,
	ResMut<'a, AssetServer>,
);
