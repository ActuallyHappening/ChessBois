use self::{
	automatic::AutomaticState, cells::CellClicked, hotkeys::HotkeysPlugin, manual::ManualState,
	ui::UiPlugin, visualization::VisualizationPlugin,
};
use crate::{
	solver::{
		algs::{Algorithm, Options},
		BoardOptions,
	},
	ChessPoint,
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use derive_more::From;
use top_level_types::OptionsWrapper;

mod automatic;
mod manual;
pub(crate) use manual::ManualMoves;

mod cells;
mod hotkeys;
mod ui;
mod visualization;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugin(VisualizationPlugin)
			.add_plugin(UiPlugin)
			.add_plugin(AutomaticState)
			.add_plugin(ManualState)
			.add_plugins(
				DefaultPickingPlugins
					.build()
					.disable::<DefaultHighlightingPlugin>()
					.disable::<DebugPickingPlugin>(),
			)
			.add_plugin(HotkeysPlugin)
			.add_event::<CellClicked>()
			.add_startup_system(setup);
	}
}

/// Re-rendered every frame
#[derive(Resource)]
pub struct SharedState {
	// inputs
	/// Shape & structure of board
	pub board_options: BoardOptions,
	pub start: Option<ChessPoint>,

	// visuals
	pub visuals: Visuals,
}

use visuals::*;
mod visuals {
	use super::*;

	pub struct Visuals {
		pub show_numbers: bool,
		pub show_dots: bool,
		pub show_markers: bool,
		viz_width: f32,
	}

	impl Visuals {
		pub const DEFAULT: Self = Visuals {
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
				},
			}
		}
	}

	impl Default for Visuals {
		fn default() -> Self {
			Self::DEFAULT
		}
	}
}

/// What [Options] are currently selected / rendered
#[derive(Resource, Debug, Clone, Deref, DerefMut, PartialEq, Eq, From)]
pub struct CurrentOptions {
	pub current: Options,
}

/// Sets up default resources + sends initial [NewOptions] event
fn setup(mut commands: Commands) {
	let mut default_options = CurrentOptions::from_options(Options::default());

	if let Some(state) = crate::weburl::try_load_state_from_url() {
		info!("Loaded state from URL!");
		default_options.current.options = state.options;
	}

	commands.insert_resource(default_options);
}

mod top_level_types {
	use super::*;

	pub trait OptionsWrapper {
		fn into_options(self) -> Options;
		fn as_options(&self) -> &Options;

		fn from_options(options: Options) -> Self;
	}

	impl OptionsWrapper for CurrentOptions {
		fn into_options(self) -> Options {
			self.current
		}

		fn as_options(&self) -> &Options {
			&self.current
		}

		fn from_options(options: Options) -> Self {
			CurrentOptions { current: options }
		}
	}
}

type ResSpawning<'a> = (
	ResMut<'a, Assets<Mesh>>,
	ResMut<'a, Assets<StandardMaterial>>,
	ResMut<'a, AssetServer>,
);
