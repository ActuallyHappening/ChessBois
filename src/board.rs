use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy_mod_picking::prelude::*;
use msrc_q11::*;
use msrc_q11::{algs::ImplementedAlgorithms, pieces::ChessPiece, BoardOptions, ChessPoint};
use std::f32::consts::TAU;
use strum::IntoEnumIterator;
use strum::{EnumIter, EnumString, EnumVariantNames, IntoStaticStr};

use crate::*;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_event::<NewCellSelected>()
			.add_event::<NewBoardCellOptions>()
			.add_startup_system(setup)
			.add_system(handle_automatic_computation)
			.add_system(spawn_left_sidebar_ui)
			.add_system(right_sidebar_ui)
			.add_plugins(
				DefaultPickingPlugins
					.build()
					.disable::<DefaultHighlightingPlugin>()
					.disable::<DebugPickingPlugin>(),
			)
			.add_system(handle_new_cell_selected_event)
			.add_system(handle_new_board_event)
			.add_system(handle_spawning_visualization);
	}
}

/// Represents information required to display cells + visual solutions
#[derive(Debug, Clone)]
pub struct Options {
	options: BoardOptions,
	selected_start: Option<ChessPoint>,
}

#[derive(Debug, Clone)]
pub struct NewCellSelected {
	new: ChessPoint,
}

/// Event representing when the board has changed size/shape/<options>, NOT start location!
#[derive(Debug, Clone)]
pub struct NewBoardCellOptions {
	new: BoardOptions,
}

#[derive(Resource, Debug, Clone)]
pub struct CurrentOptions {
	current: Options,
}

#[derive(Resource, Debug, Clone, Default, PartialEq, Eq, EnumIter, IntoStaticStr, Hash)]
pub enum Algorithm {
	#[default]
	Warnsdorf,

	#[strum(serialize = "Brute Force")]
	BruteForce,

	#[strum(serialize = "Brute Force (Not Cached")]
	BruteForceNotCached,
}

impl Algorithm {
	fn to_impl<P: ChessPiece>(&self, piece: P) -> ImplementedAlgorithms<P> {
		match self {
			Algorithm::Warnsdorf => ImplementedAlgorithms::Warnsdorf(piece),
			Algorithm::BruteForce => ImplementedAlgorithms::BruteRecursiveCached(piece),
			Algorithm::BruteForceNotCached => ImplementedAlgorithms::BruteRecursiveNotCached(piece),
		}
	}

	fn get_description(&self) -> &'static str {
		match self {
			Algorithm::Warnsdorf => "A standard knights tour.\
			This algorithm applies Warnsdorf's Rule, which tells you to always move to the square with the fewest available moves. \
			This algorithm is always guaranteed to terminate in finite time, however it sometimes misses solutions e.g. 8x8 board @ (5, 3).\
			Warnsdorf's Rule is very easy to implement and is very popular because of its simplicity. The implementation used is sub-optimal, but should suffice.
			", 
			Algorithm::BruteForce => "A standard knights tour.\
			This algorithm is a recursive brute-force approach, which favours Warnsdorf's Rule first before backtracking.\
			This algorithm is always guaranteed to terminate in finite time, but that time complexity is exponential compared with number of cells, so \
			large boards with no solutions will take a long time to solve. In worst case scenario, since it is brute force, it will check every possible \
			knights tour before exiting with no solution! However, if Warnsdorf's algorithm finds a solution, this program will find that solution first.
			This algorithm uses a cache (because this will often save expensive work) so sometimes you will see 0 states considered. This is because the \
			cache has been hit, to remove cache select the 'Brute Force (Not Cached)' algorithm.
			",
			Algorithm::BruteForceNotCached => "A standard knights tour.\
			Same as the other brute force algorithm, except without the cache. This will likely slow your computer down a bit when computing larger boards.
			",
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

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut ass: ResMut<AssetServer>,
	mut alg: Option<Res<Algorithm>>,
) {
	let options = Options {
		options: BoardOptions::new(8, 8),
		selected_start: Some(ChessPoint::new(9, 9)),
	};
	let current_options = CurrentOptions {
		current: options.clone(),
	};

	commands.insert_resource(current_options);
	commands.init_resource::<Algorithm>();

	spawn_cells(
		&mut commands,
		&options,
		&mut meshes,
		&mut materials,
		&mut ass,
		alg,
	);
	// spawn_visualization_from_options(&options, &mut commands, &mut meshes, &mut materials);

	// spawn_left_sidebar_ui(&mut commands);
}

use cells::*;
mod cells;

mod compute {
	use super::cached_info::CellMark;
	use super::*;
	use bevy::tasks::Task;
	use msrc_q11::algs::Computation;

	#[derive(Resource, Debug)]
	pub struct ComputationTask(Task<Computation>);

	#[derive(Resource, Debug)]
	pub struct ComputationResult(Computation);

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
		let thread_pool = AsyncComputeTaskPool::get();
		let (start, options) = (options.selected_start.unwrap(), options.options);

		commands.insert_resource(ComputationTask(thread_pool.spawn(async move {
			let ret = alg.tour_computation(options.clone(), start).await;
			// panic!("Failed here");
			ret
		})));
	}

	fn get_computation(commands: &mut Commands, task: ResMut<ComputationTask>) -> Computation {
		let task = task.into_inner();
		let task = &mut task.0;

		let comp = futures::executor::block_on(task);

		commands.remove_resource::<ComputationTask>();
		commands.insert_resource(ComputationResult(comp.clone()));

		comp
	}

	/// When / how to run [get_computation], + cached_info
	/// + mark reloading
	pub fn handle_automatic_computation(
		mut commands: Commands,
		task: Option<ResMut<ComputationTask>>,
		alg: Res<Algorithm>,
		options: Res<CurrentOptions>,

		mut ass: ResMut<AssetServer>,
		mut meshes: ResMut<Assets<Mesh>>,
		mut materials: ResMut<Assets<StandardMaterial>>,
	) {
		if let Some(task) = task {
			let alg = alg.into_inner();
			// does the work of computing
			let comp = get_computation(&mut commands, task);

			let options = &options.current;
			cached_info::set(
				&options.selected_start.unwrap(),
				&options.options,
				alg,
				CellMark::from(comp),
			);
			mark_reload_cell(
				&options.selected_start.unwrap(),
				&mut ass,
				alg,
				&mut commands,
				&mut meshes,
				&mut materials,
			)
		}
	}
}

mod cached_info {
	use lru::LruCache;
	use msrc_q11::algs::Computation;
	use once_cell::sync::Lazy;
	use std::{num::NonZeroUsize, sync::Mutex};

	use super::*;

	static CACHE: Lazy<Mutex<LruCache<(BoardOptions, ChessPoint, Algorithm), CellMark>>> =
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

	pub fn get(point: &ChessPoint, options: &BoardOptions, alg: &Algorithm) -> Option<CellMark> {
		let mut cache = CACHE.lock().unwrap();
		cache.get(&(options.clone(), *point, alg.clone())).cloned()
	}
	pub fn set(point: &ChessPoint, options: &BoardOptions, alg: &Algorithm, mark: CellMark) {
		let mut cache = CACHE.lock().unwrap();
		cache.put((options.clone(), *point, alg.clone()), mark);
	}
}

use visualization::*;
mod visualization {
	use super::{
		compute::{begin_background_compute, ComputationResult, ComputationTask},
		*,
	};
	use bevy::transform::commands;
	use msrc_q11::{
		algs::{Computation, ImplementedAlgorithms},
		pieces::StandardKnight,
		Move, Moves,
	};

	#[allow(dead_code)]
	#[derive(Component, Debug, Clone)]
	pub struct VisualizationComponent {
		from: ChessPoint,
		to: ChessPoint,
	}

	/// Consumes [Res<ComputationResult>] and spawns visualization
	pub fn handle_spawning_visualization(
		mut commands: Commands,
		options: Res<CurrentOptions>,
		solution: Option<Res<ComputationResult>>,

		viz: Query<Entity, With<VisualizationComponent>>,

		mut meshes: ResMut<Assets<Mesh>>,
		mut materials: ResMut<Assets<StandardMaterial>>,
	) {
		if let Some(solution) = solution {
			if !solution.is_changed() {
				// not a new solution, don't do anything
				return;
			}

			let solution: Computation = solution.into_comp();
			match solution {
				Computation::Successful {
					solution: moves,
					explored_states: states,
				} => {
					debug!(
						"{} states visited, {} already exists",
						states,
						viz.iter().count()
					);

					if viz.iter().count() == 0 {
						// despawn_visualization(&mut commands, viz);
						spawn_visualization(
							moves,
							options.current.options.clone(),
							&mut commands,
							&mut meshes,
							&mut materials,
						)
					}
				}
				Computation::Failed {
					total_states: states,
				} => {
					// despawn_visualization(&mut commands, viz);
					info!("{} but No solution found!", states);
				}
			}
		}
	}

	pub fn begin_showing_new_visualization(
		options: &Options,
		alg: Res<Algorithm>,
		commands: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
	) {
		if let Some(start) = options.selected_start {
			if options.options.get_unavailable_points().contains(&start) {
				// debug!("Start point is disabled!");
				return;
			}

			let piece = StandardKnight {};
			let solver = alg.to_impl(piece);

			begin_background_compute(solver, options.clone(), commands);

			// match algs.tour_no_repeat(options.clone(), start) {
			// 	Some(moves) => {
			// 		// spawn_visualization(moves, options, commands, meshes, materials)
			// 	}
			// 	None => {
			// 		info!("No solution found!");
			// 	}
			// }
		}

		// spawn_path_line(
		// 	commands,
		// 	meshes,
		// 	materials,
		// 	&start,
		// 	&ChessPoint::new(4, 4),
		// 	&board,
		// )
	}

	pub fn spawn_visualization(
		moves: Moves,
		options: BoardOptions,
		commands: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
	) {
		for Move { from, to } in moves.iter() {
			spawn_path_line(
				from,
				to,
				&options,
				VISUALIZATION_SELECTED_COLOUR,
				commands,
				meshes,
				materials,
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
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
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

		let mesh_thin_rectangle = meshes.add(
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
				material: materials.add(colour.into()),
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

use self::compute::handle_automatic_computation;
mod ui {
	use super::{
		compute::{ComputationResult, ComputationTask},
		*,
	};
	use bevy_egui::{
		egui::{Color32, RichText},
		*,
	};
	use msrc_q11::algs::Computation;
	use strum::VariantNames;

	pub fn spawn_left_sidebar_ui(
		mut commands: Commands,
		mut contexts: EguiContexts,

		current_alg: Res<Algorithm>,
		current_options: ResMut<CurrentOptions>,
		mut new_board_event: EventWriter<NewBoardCellOptions>,
	) {
		egui::SidePanel::left("general_controls_panel").show(contexts.ctx_mut(), |ui| {
			let old_options: BoardOptions = current_options.current.options.clone();
			let current_alg = current_alg.into_inner();

			ui.heading("Controls Panel");
			ui.label("Instructions: Hover over cell to begin knight there. Click cell to toggle availability (red = unavailable). You can alter the dimensions of the board (below) and the algorithm used.");

			// ui.add(egui::Slider::new(&mut my_f32, 3.0..=10.).text("My value"));
			// ui.add(egui::Slider::new(&mut ui_state.value, 0.0..=10.0).text("value"));

			ui.label("Change board dimensions:");
			ui.horizontal(|ui| {
				if ui.button("Wider +1").clicked() {
					let new_options = old_options.clone().update_width(old_options.width() + 1);
					new_board_event.send(NewBoardCellOptions { new: new_options });
				}

				if ui.button("Thinner -1").clicked() {
					let new_options = old_options.clone().update_width(old_options.width() - 1);
					new_board_event.send(NewBoardCellOptions { new: new_options });
				}
			});

			ui.horizontal(|ui| {
				if ui.button("Taller +1").clicked() {
					let new_options = old_options.clone().update_height(old_options.height() + 1);
					new_board_event.send(NewBoardCellOptions { new: new_options });
				}

				if ui.button("Shorter -1").clicked() {
					let new_options = old_options.clone().update_height(old_options.height() - 1);
					new_board_event.send(NewBoardCellOptions { new: new_options });
				}
			});

			ui.label("Select algorithm:");
			ui.horizontal(|ui| {
				// let alg_names = Algorithm::VARIANTS;
				// for name in alg_names {
				// 	let mut btn = ui.button(*name);
				// 	if current_alg ==
				// 	if btn.clicked() {
				// 		commands.insert_resource(Algorithm::from_str(name).unwrap())
				// 	}
				// }
				for alg in Algorithm::iter() {
					let str: &'static str = alg.clone().into();
					let mut text = RichText::new(str);
					if &alg == current_alg {
						text = text.color(UI_ALG_ENABLED_COLOUR);
					}
					if ui.button(text).clicked() {
						commands.insert_resource(alg);
					}
				}
			});
			ui.label(current_alg.get_description());
		});
	}

	pub fn right_sidebar_ui(
		options: Res<CurrentOptions>,
		alg: Res<Algorithm>,
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
				let alg_selected: &str = alg.into_inner().into();
				ui.label(format!(
					"Current info: Starting at {start} with {} algorithm {}",
					alg_selected,
					options.options.get_description()
				));
			}
		});
	}
}
