use crate::{
	solver::{algs::Algorithm, pieces::ChessPiece, BoardOptions, Moves},
	ChessPoint,
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

mod automatic;
// mod manual;

mod cells;
mod compute;
// mod hotkeys;
// mod ui;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
	fn build(&self, app: &mut App) {
		app
			// .add_plugin(VisualizationPlugin)
			// .add_plugin(UiPlugin)
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
	pub moves: Option<Moves>,
	pub visual_opts: VisualOpts,

	// ui / interactions
	pub on_click: ToggleAction,
}

mod shared_state {
	use super::*;
	use crate::solver::algs::ComputeInput;

	impl Plugin for SharedState {
		fn build(&self, app: &mut App) {
			app.add_systems((SharedState::sys_render_cells,));
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
	}
}

use visuals::*;

use self::{
	automatic::{AutomaticPlugin, ToggleAction},
	cells::CellsPlugin,
};
mod visuals {
	use super::*;

	#[derive(Clone)]
	pub struct VisualOpts {
		pub show_numbers: bool,
		pub show_dots: bool,
		pub show_markers: bool,
		viz_width: f32,
	}

	impl VisualOpts {
		pub const DEFAULT: Self = VisualOpts {
			show_numbers: true,
			show_dots: true,
			show_markers: true,
			viz_width: 0.2,
		};

		pub fn dimensions(&self) -> Vec2 {
			Vec2::new(self.viz_width, self.viz_width)
		}

		pub fn set_width(&mut self, viz_width: f32) {
			match viz_width {
				_x if (0.1..=0.5).contains(&_x) => self.viz_width = viz_width,
				_ => {
					warn!("Setting viz_width to unnacceptable value: {viz_width}");
				}
			}
		}
	}

	impl Default for VisualOpts {
		fn default() -> Self {
			Self::DEFAULT
		}
	}
}

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
