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

mod cells;
mod hotkeys;
mod ui;
mod visualization;
mod viz_colours;

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
	// let mut board = BoardOptions::new(2, 3);
	// board.rm((1, 2));
	// board.rm((2, 2));
	// board.rm((2, 1));
	// board.rm((3, 1));
	let board = BoardOptions::new(8, 8);

	let options = Options {
		options: board,
		selected_start: None,
		selected_algorithm: Algorithm::default(),
		requires_updating: true,
	};
	let current_options = CurrentOptions::from_options(options);

	commands.insert_resource(current_options);
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
