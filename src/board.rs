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

use crate::solver::algs::*;
use crate::solver::pieces::StandardKnight;
use crate::solver::{pieces::ChessPiece, BoardOptions, ChessPoint};

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use std::f32::consts::TAU;

use crate::*;

mod automatic;
mod manual;

mod cached_info;
mod cells;
mod compute;
mod coords;
mod state_manual;
mod ui;
mod visualization;
mod viz_colours;

use self::automatic::AutomaticState;
use self::cached_info::update_cache_from_computation;
use self::compute::{begin_background_compute, handle_automatic_computation, ComputationResult};
use self::manual::ManualState;
use self::state_manual::{ManualFreedom, ManualNextCell};
use self::viz_colours::VizColour;
use cells::*;
use coords::*;
use ui::*;
use visualization::*;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_event::<NewOptions>()
			.add_event::<ComputationResult>()
			.add_startup_system(setup)
			.add_system(left_sidebar_ui)
			.add_plugin(AutomaticState)
			.add_plugin(ManualState)
			.add_plugins(
				DefaultPickingPlugins
					.build()
					.disable::<DefaultHighlightingPlugin>()
					.disable::<DebugPickingPlugin>(),
			);
	}
}

#[derive(Resource, Debug, Clone)]
pub struct CurrentOptions {
	current: Options,
}

#[derive(Debug, Clone)]
pub struct NewOptions {
	new: Options,
}

use top_level_types::OptionsWrapper;
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

/// Decides what happens when [NewOptions] event comes in
fn handle_new_options(
	mut options_events: EventReader<NewOptions>,
	old_options: Res<CurrentOptions>,

	cells: Query<Entity, (With<CellMarker>, With<ChessPoint>)>,
	viz: Query<Entity, With<VisualizationComponent>>,
	markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,

	mut commands: Commands,
	mut mma: ResSpawning,
) {
	if let Some(options) = options_events.iter().next() {
		let options = options.clone().into_options();
		let old_options = old_options.clone().into_options();

		if options.force_update {
			info!("Force updating ...")
		}

		if options == old_options && !options.force_update {
			// info!("Ignoring update, options are the same");
			return;
		}

		despawn_visualization(&mut commands, viz);

		// markers
		despawn_markers(&mut commands, markers);
		spawn_markers(&options, &mut commands, &mut mma);

		// if BoardOptions changed, despawn + re-spawn cells
		if options.options != old_options.options || options.force_update {
			// info!("BoardOptions changed, de-spawning + re-spawning cells & markers");
			despawn_cells(&mut commands, cells);

			spawn_cells(&options, &mut commands, &mut mma);
		}

		// begin recomputing visualization
		begin_background_compute(
			options.selected_algorithm,
			&StandardKnight {},
			options.clone(),
			&mut commands,
		);

		// add new options as current
		commands.insert_resource(CurrentOptions::from_options(Options {
			force_update: false,
			..options.clone()
		}));

		options_events.clear();
	}
}

pub fn handle_plane_clicked<T: IsPointerEvent>(
	In(_): In<ListenedEvent<T>>,
	options: Res<CurrentOptions>,
	mut update_board: EventWriter<NewOptions>,
) -> Bubble {
	debug!("Plane clicked");

	update_board.send(NewOptions::from_options(Options {
		selected_start: None,
		..options.clone().into_options()
	}));

	Bubble::Up
}
