use self::{
	automatic::AutomaticState, cells::CellClicked, hotkeys::HotkeysPlugin, manual::ManualState,
	ui::UiPlugin,
};
use crate::solver::{
	algs::{Algorithm, Options},
	BoardOptions,
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
