//! Overall structure
//! Whenever something that could change the visualization happens, send a [NewOptions] event.
//! [NewOptions]:
//! - Handled by [handle_new_options]
//! - Begins new computation
//!
//! Each NewOptions guarantees that the visualization will be voided/de-spawned
//!
//! When computation is required, start with [begin_background_compute]
//! - polls result with [get_computation]
//! - system [handle_automatic_computation] sends [ComputationResult] event + adds as resource when computation is received

use self::{automatic::AutomaticState, manual::ManualState, ui::UiPlugin};
use crate::solver::{
	algs::{Algorithm, Options},
	BoardOptions,
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use top_level_types::OptionsWrapper;

mod automatic;
mod manual;

mod cells;
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
			.add_startup_system(setup);
	}
}

/// What 'shape', dimensions and disabled cells there are
#[derive(Resource, Debug, Clone)]
pub struct CurrentOptions {
	current: Options,
}

#[derive(Debug, Clone)]
pub struct NewOptions {
	new: Options,
}

/// Sets up default resources + sends initial [NewOptions] event
fn setup(mut commands: Commands, mut update_board: EventWriter<NewOptions>) {
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
		force_update: true,
	};
	let current_options = CurrentOptions::from_options(options.clone());

	commands.insert_resource(current_options);

	update_board.send(NewOptions::from_options(options));
}

mod top_level_types {
	use super::*;

	pub trait OptionsWrapper {
		fn into_options(self) -> Options;
		fn as_options(&self) -> &Options;

		fn from_options(options: Options) -> Self;
	}

	impl OptionsWrapper for NewOptions {
		fn into_options(self) -> Options {
			self.new
		}

		fn as_options(&self) -> &Options {
			&self.new
		}

		fn from_options(options: Options) -> Self {
			NewOptions { new: options }
		}
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
