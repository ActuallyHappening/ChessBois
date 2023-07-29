use crate::{
	solver::{algs::Algorithm, pieces::ChessPiece, BoardOptions, Moves},
	ChessPoint,
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

mod automatic;
// mod manual;

mod cells;
mod coloured_moves;
mod compute;
// mod hotkeys;
mod ui;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
	fn build(&self, app: &mut App) {
		app
			// .add_plugin(VisualizationPlugin)
			.add_plugin(UiPlugin)
			.add_plugin(AutomaticPlugin)
			// .add_plugin(ManualState)
			.add_plugin(SharedState::default())
			.add_plugin(CellsPlugin)
			.add_plugins(
				DefaultPickingPlugins
					.build()
					.disable::<DefaultHighlightingPlugin>()
					.disable::<DebugPickingPlugin>(),
			)
			// .add_plugin(HotkeysPlugin)
			.add_startup_system(setup);
	}
}

/// Re-rendered every frame
#[derive(Resource, Default, Clone)]
pub struct SharedState {
	// inputs
	/// Set using [set_alg]
	pub alg: Algorithm,

	/// Set using [set_board_options]
	pub board_options: BoardOptions,

	/// Set using [set_start]
	pub start: Option<ChessPoint>,
	pub piece: ChessPiece,

	// visuals
	pub moves: Option<ColouredMoves>,
	pub visual_opts: cells::visualization::VisualOpts,

	// ui / interactions
	pub on_click: ToggleAction,
}

mod shared_state {
	use super::{cells::visualization, *};
	use crate::solver::algs::ComputeInput;

	impl Plugin for SharedState {
		fn build(&self, app: &mut App) {
			app.add_systems((
				SharedState::sys_render_cells,
				SharedState::sys_render_viz,
				SharedState::sys_render_markers,
			));
		}
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

	impl TryFrom<SharedState> for ComputeInput {
		type Error = ();

		fn try_from(value: SharedState) -> Result<Self, Self::Error> {
			value.get_compute_state().ok_or(())
		}
	}

	impl SharedState {
		pub fn get_compute_state(self) -> Option<ComputeInput> {
			Some(ComputeInput {
				alg: self.alg,
				start: self.start?,
				board_options: self.board_options,
				piece: self.piece,
			})
		}

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
}

use self::{
	automatic::{AutomaticPlugin, ToggleAction},
	cells::CellsPlugin,
	coloured_moves::ColouredMoves,
	ui::UiPlugin,
};

/// Sets up default resources + sends initial [NewOptions] event
fn setup(mut commands: Commands) {
	let state = SharedState::default();

	// if let Some(state) = crate::weburl::try_load_state_from_url() {
	// 	info!("Loaded state from URL!");
	// 	default_options.options = state.options;
	// }

	commands.insert_resource(state);
}

type ResSpawning<'a> = (
	ResMut<'a, Assets<Mesh>>,
	ResMut<'a, Assets<StandardMaterial>>,
	ResMut<'a, AssetServer>,
);
