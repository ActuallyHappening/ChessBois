use crate::solver::algs::*;
use crate::solver::pieces::StandardKnight;
use crate::solver::{algs::ImplementedAlgorithms, pieces::ChessPiece, BoardOptions, ChessPoint};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy_mod_picking::prelude::*;
use std::f32::consts::TAU;

use crate::*;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_event::<NewOptions>()
			.add_event::<ComputationResult>()
			.add_startup_system(setup)
			.add_system(handle_automatic_computation)
			.add_system(update_cache_from_computation)
			.add_system(handle_spawning_visualization)
			.add_system(handle_new_options)
			.add_system(spawn_left_sidebar_ui)
			.add_system(right_sidebar_ui)
			.add_plugins(
				DefaultPickingPlugins
					.build()
					.disable::<DefaultHighlightingPlugin>()
					.disable::<DebugPickingPlugin>(),
			);
		// .add_system(handle_new_cell_selected_event)
		// .add_system(handle_new_board_event)
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

use coords::*;
mod coords {
	use super::*;

	/// Returns spacial coordinates of center of cell mesh
	fn get_spacial_coord_normalized(board: &BoardOptions, chess_position: ChessPoint) -> Vec2 {
		let ChessPoint { row: y, column: x } = chess_position;
		let width = board.width() as f32;
		let height = board.height() as f32;
		let x = x as f32;
		let y = y as f32;

		// normalized: (column, row) = (x, y)
		// Adjusted = ((x - 1) -X Delta, (y - 1) - Y Delta)
		// X Delta = (width - 1) / 2

		let x_adjusted = (x - 1.) - (width - 1.) / 2.;
		let y_adjusted = (y - 1.) - (height - 1.) / 2.;

		Vec2::new(x_adjusted, y_adjusted)
	}

	pub fn get_spacial_coord(board: &BoardOptions, chess_position: ChessPoint) -> Vec3 {
		let normalized = get_spacial_coord_normalized(board, chess_position) * CELL_SIZE;

		Vec3::new(normalized.x, CELL_HEIGHT, -normalized.y)
	}

	pub fn get_spacial_coord_2d(board: &BoardOptions, chess_position: ChessPoint) -> Vec2 {
		let normalized = get_spacial_coord_normalized(board, chess_position) * CELL_SIZE;

		Vec2::new(normalized.x, -normalized.y)
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_coords_center() {
			let coords = get_spacial_coord_normalized(&BoardOptions::new(3, 3), ChessPoint::new(2, 2));

			assert_eq!(coords, Vec2::new(0., 0.));
		}

		#[test]
		fn test_coords_bl_2() {
			let coords = get_spacial_coord_normalized(&BoardOptions::new(2, 2), ChessPoint::new(1, 1));

			assert_eq!(coords, Vec2::new(-0.5, -0.5));
		}

		#[test]
		fn test_coords_bl_5() {
			let coords = get_spacial_coord_normalized(&BoardOptions::new(5, 5), ChessPoint::new(1, 1));

			assert_eq!(coords, Vec2::new(-2., -2.));
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
	let options = Options {
		options: BoardOptions::new(8, 8),
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
			options.selected_algorithm.to_impl(StandardKnight {}),
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

use cells::*;
mod cells;

mod compute {
	use std::sync::Mutex;

	use super::*;
	use crate::solver::algs::Computation;
	use bevy::tasks::Task;

	/// If this resource is present, then computation is underway
	#[derive(Resource, Debug)]
	pub struct ComputationTask(Task<Computation>, Options);

	#[derive(Resource, Debug, Clone, PartialEq, Eq)]
	pub struct ComputationResult(Computation, Options);

	impl ComputationResult {
		pub fn get(self) -> (Computation, Options) {
			(self.0, self.1)
		}
	}

	// impl From<ComputationResult> for Computation {
	// 	fn from(result: ComputationResult) -> Self {
	// 		result.0
	// 	}
	// }

	impl ComputationResult {
		pub fn into_comp(&self) -> Computation {
			self.0.clone()
		}
	}

	pub fn begin_background_compute<P: ChessPiece + Send + Sync + 'static>(
		alg: ImplementedAlgorithms<P>,
		options: Options,
		commands: &mut Commands,
	) {
		let state = options.clone();
		if let Some(_) = options.selected_start {
			let thread_pool = AsyncComputeTaskPool::get();

			commands.insert_resource(ComputationTask(
				thread_pool.spawn(async move { alg.tour_computation_cached(options.clone()).unwrap() }),
				state,
			));
		}
	}

	static TASK_RESULT: Mutex<TaskResult> = Mutex::new(TaskResult::Empty);

	#[derive(Clone)]
	enum TaskResult {
		Empty,
		Result(Computation),
	}

	fn start_executing_task(task: &mut Task<Computation>) {
		let res = futures::executor::block_on(task);
		*TASK_RESULT.lock().unwrap() = TaskResult::Result(res);
	}
	fn poll_computation_result() -> TaskResult {
		(*TASK_RESULT.lock().unwrap()).clone()
	}

	/// adds [ComputationResult]; Polls [ComputationTask] and returns [Computation] if ready.
	/// Theoretically non-blocking
	fn get_computation(
		commands: &mut Commands,
		task: ResMut<ComputationTask>,
	) -> Option<ComputationResult> {
		let task = task.into_inner();
		let state = task.1.clone();
		let task = &mut task.0;

		match poll_computation_result() {
			TaskResult::Empty => {
				start_executing_task(task);
				None
			}
			TaskResult::Result(comp) => {
				let res = ComputationResult(comp, state);

				commands.remove_resource::<ComputationTask>();
				Some(res)
			}
		}
	}

	/// When / how to run [get_computation], sends [ComputationResult] event
	/// Validates computation is valid for current Options
	pub fn handle_automatic_computation(
		mut commands: Commands,
		task: Option<ResMut<ComputationTask>>,
		options: Res<CurrentOptions>,

		mut update_computation: EventWriter<ComputationResult>,
	) {
		if let Some(task) = task {
			// does the work of computing
			if let Some(comp) = get_computation(&mut commands, task) {
				let state = options.as_options();
				if &comp.1 != state {
					info!("Ignoring received computation as state has changed since");
					return;
				}

				// let message get out to everybody
				update_computation.send(comp.clone());
				commands.insert_resource(comp);
			}
		}
	}
}

mod cached_info {
	use crate::solver::algs::Computation;
	use lru::LruCache;
	use once_cell::sync::Lazy;
	use std::{num::NonZeroUsize, sync::Mutex};

	use super::*;

	static CACHE: Lazy<Mutex<LruCache<Options, CellMark>>> =
		Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(10_000).unwrap())));

	#[derive(Clone)]
	pub enum CellMark {
		Failed,
		Succeeded,
	}

	impl From<Computation> for CellMark {
		fn from(value: Computation) -> Self {
			match value {
				Computation::Successful { .. } => CellMark::Succeeded,
				Computation::Failed { .. } => CellMark::Failed,
			}
		}
	}

	pub fn get(options: &Options) -> Option<CellMark> {
		let mut cache = CACHE.lock().unwrap();
		trace!(
			"Getting info cache for alg: {:?} at {}",
			options.selected_algorithm,
			options.selected_start.unwrap()
		);
		cache.get(options).cloned()
	}
	fn set(options: Options, mark: CellMark) {
		let mut cache = CACHE.lock().unwrap();
		trace!(
			"Setting info cache for alg: {:?} at {}",
			options.selected_algorithm,
			options.selected_start.unwrap()
		);
		cache.put(options, mark);
	}

	pub fn update_cache_from_computation(
		mut computations: EventReader<ComputationResult>,
		mut commands: Commands,

		// markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,
		mut mma: ResSpawning,
	) {
		for comp in computations.iter() {
			let (comp, options) = comp.clone().get();
			let mark = CellMark::from(comp);

			debug!("Updating info cache");
			set(options.clone(), mark);

			// despawn_markers(&mut commands, markers);
			spawn_markers(&options, &mut commands, &mut mma)
		}
	}
}

use visualization::*;
mod visualization {
	use super::{
		compute::{begin_background_compute, ComputationResult},
		*,
	};
	use crate::solver::{algs::Computation, pieces::StandardKnight, Move, Moves};

	#[derive(Component, Debug, Clone)]
	pub struct VisualizationComponent {
		from: ChessPoint,
		to: ChessPoint,
	}

	/// Consumes [EventReader<ComputationResult>] and actually spawns concrete visualization
	pub fn handle_spawning_visualization(
		mut commands: Commands,
		mut solutions: EventReader<ComputationResult>,

		viz: Query<Entity, With<VisualizationComponent>>,

		mut mma: ResSpawning,
	) {
		if let Some(solution) = solutions.iter().next() {
			let (solution, Options { options, .. }) = solution.clone().get();
			if let Computation::Successful {
				solution: moves, ..
			} = solution
			{
				spawn_visualization(moves, options, &mut commands, &mut mma);
			}

			solutions.clear()
		}
	}

	/// Actually spawn entities of new solution
	pub fn spawn_visualization(
		moves: Moves,
		options: BoardOptions,
		commands: &mut Commands,
		mma: &mut ResSpawning,
	) {
		for Move { from, to } in moves.iter() {
			spawn_path_line(
				from,
				to,
				&options,
				VISUALIZATION_SELECTED_COLOUR,
				commands,
				mma,
			)
		}
	}

	pub fn despawn_visualization(
		commands: &mut Commands,
		visualization: Query<Entity, With<VisualizationComponent>>,
	) {
		for entity in visualization.iter() {
			commands.entity(entity).despawn_recursive();
		}
	}

	fn spawn_path_line(
		from: &ChessPoint,
		to: &ChessPoint,
		options: &BoardOptions,
		colour: Color,

		commands: &mut Commands,
		// meshes: &mut ResMut<Assets<Mesh>>,
		// materials: &mut ResMut<Assets<StandardMaterial>>,
		mma: &mut ResSpawning,
	) {
		let start_pos = get_spacial_coord_2d(options, *from);
		let end_pos = get_spacial_coord_2d(options, *to);

		let center = (start_pos + end_pos) / 2.; // ✅
		let length = (start_pos - end_pos).length(); // ✅
		let angle: f32 = -(start_pos.y - end_pos.y).atan2(start_pos.x - end_pos.x);

		// assert_eq!(angle, TAU / 8., "Drawing from {from} [{from:?}] [{from_pos}] to {to} [{to:?}] [{to_pos}], Angle: {angle}, 𝚫y: {}, 𝚫x: {}", (to_pos.y - from_pos.y), (to_pos.x - from_pos.x));
		// info!("Angle: {angle}, {}", angle.to_degrees());

		let transform =
			Transform::from_translation(Vec3::new(center.x, VISUALIZATION_HEIGHT, center.y))
				.with_rotation(Quat::from_rotation_y(angle));

		// info!("Transform: {:?}", transform);
		// info!("Angle: {:?}, Length: {:?}", angle, length);

		let mesh_thin_rectangle = mma.0.add(
			shape::Box::new(
				length,
				VISUALIZATION_DIMENSIONS.x,
				VISUALIZATION_DIMENSIONS.y,
			)
			.into(),
		);

		commands.spawn((
			PbrBundle {
				mesh: mesh_thin_rectangle,
				material: mma.1.add(colour.into()),
				transform,
				..default()
			},
			VisualizationComponent {
				from: *from,
				to: *to,
			},
		));
	}
}

use ui::*;

use self::cached_info::update_cache_from_computation;
use self::compute::{begin_background_compute, handle_automatic_computation, ComputationResult};
mod ui {
	use super::{compute::ComputationResult, *};
	use crate::solver::algs::Computation;
	use bevy_egui::{
		egui::{Color32, RichText},
		*,
	};
	use strum::IntoEnumIterator;

	pub fn spawn_left_sidebar_ui(
		mut commands: Commands,
		mut contexts: EguiContexts,

		options: ResMut<CurrentOptions>,
		mut new_board_event: EventWriter<NewOptions>,
	) {
		egui::SidePanel::left("general_controls_panel").show(contexts.ctx_mut(), |ui| {
			let options = options.clone().into_options();
			let current_alg = &options.selected_algorithm;

			ui.heading("Controls Panel");
			ui.label("Instructions: Hover over cell to begin knight there. Click cell to toggle availability (red = unavailable). You can alter the dimensions of the board (below) and the algorithm used.");

			ui.add(egui::Slider::from_get_set((2.)..=10., |val| {
				let mut options = options.clone();
				if let Some(new_val) = val {
					options.options = options.options.update_width(new_val as u8);
					options.selected_start = None;
					new_board_event.send(NewOptions::from_options(options));
					new_val
				} else {
					options.options.width() as f64
				}
			}).text("Width"));
			ui.add(egui::Slider::from_get_set((2.)..=10., |val| {
				let mut options = options.clone();
				if let Some(new_val) = val {
					options.options = options.options.update_height(new_val as u8);
					options.selected_start = None;
					new_board_event.send(NewOptions::from_options(options));
					new_val
				} else {
					options.options.height() as f64
				}
			}).text("Height"));

			ui.label("Select algorithm:");
			ui.horizontal(|ui| {
				for alg in Algorithm::iter() {
					let str: &'static str = alg.clone().into();
					let mut text = RichText::new(str);
					if &alg == current_alg {
						text = text.color(UI_ALG_ENABLED_COLOUR);
					}
					if ui.button(text).clicked() {
						new_board_event.send(NewOptions::from_options(Options {
							selected_algorithm: alg,
							selected_start: None,
							..options.clone()
						}));
					}
				}
			});
			ui.label(current_alg.get_description());
		});
	}

	pub fn right_sidebar_ui(
		options: Res<CurrentOptions>,
		computation: Option<Res<ComputationResult>>,

		mut commands: Commands,
		mut contexts: EguiContexts,
	) {
		let options = &options.current;

		let solution = computation.map(|comp| comp.into_comp());

		egui::SidePanel::right("right_sidebar").show(contexts.ctx_mut(), |ui| {
			ui.heading("Results Panel");

			if let Some(solution) = solution {
				match solution {
					Computation::Successful {
						explored_states: states,
						..
					} => {
						ui.label(
							RichText::new(format!("Solution found in {} states considered", states))
								.color(Color32::GREEN),
						);
						ui.label("Notes: if states =0 it is because the solution was cached");
					}
					Computation::Failed {
						total_states: states,
					} => {
						ui.label(
							RichText::new(format!(
								"No solution found, with {} states considered",
								states
							))
							.color(Color32::RED),
						);
					}
				}
			}

			if let Some(start) = &options.selected_start {
				let alg_selected: &str = options.selected_algorithm.clone().into();
				ui.label(format!(
					"Current info: Starting at {start} with {} algorithm {}",
					alg_selected,
					options.options.get_description()
				));
			}
		});
	}
}
