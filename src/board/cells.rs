use super::{cached_info::CellMark, *};
use crate::CELL_DISABLED_COLOUR;
use msrc_q11::CellOption;

pub fn spawn_cells(
	commands: &mut Commands,
	options: &Options,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
	ass: &mut ResMut<AssetServer>,
	alg: Option<Res<Algorithm>>,
) {
	let start = options.selected_start;
	let options = options.options.clone();
	let alg = alg.map(|al| al.into_inner());

	for point in options.get_all_points() {
		let colour = compute_colour(&point, Some(&options), start);
		spawn_cell(
			point, &options, colour, commands, meshes, materials, ass, alg,
		);
	}
}

fn despawn_cells(commands: &mut Commands, cells: Query<Entity, With<ChessPoint>>) {
	for cell in cells.iter() {
		commands.entity(cell).despawn_recursive();
	}
}

/// Takes as much information as it can get and returns the colour the cell should be.
///
/// - Pass None to options to skip checking if cell is disabled
/// - Pass None to start to skip checking if cell is selected
fn compute_colour(
	point: &ChessPoint,
	options: Option<&BoardOptions>,
	start: Option<ChessPoint>,
) -> Color {
	if options.is_some_and(|options| options.get_unavailable_points().contains(point)) {
		// info!("Point {} is unavailable", point);
		CELL_DISABLED_COLOUR
	} else if Some(*point) == start {
		CELL_SELECTED_COLOUR
	} else {
		point.get_standard_colour()
	}
}

#[allow(clippy::too_many_arguments)]
fn spawn_cell(
	at: ChessPoint,
	options: &BoardOptions,
	colour: Color,
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
	ass: &mut ResMut<AssetServer>,
	alg: Option<&Algorithm>,
) {
	let transform = Transform::from_translation(get_spacial_coord(options, at))
		.with_rotation(Quat::from_rotation_x(-TAU / 4.));
	let mesh = meshes.add(shape::Box::new(CELL_SIZE, CELL_SIZE, CELL_DEPTH).into());

	commands.spawn((
		PbrBundle {
			mesh,
			transform,
			material: materials.add(StandardMaterial::from(colour)),
			..default()
		},
		Name::new(format!("Chess Square ({}, {})", at.row, at.column)),
		at,
		PickableBundle::default(),    // Makes the entity pickable
		RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
		OnPointer::<Over>::run_callback(cell_selected),
		OnPointer::<Out>::run_callback(cell_deselected),
		OnPointer::<Click>::run_callback(toggle_cell_availability),
	));

	if let Some(mark) = cached_info::get(&at, options, &Algorithm::Warnsdorf) {
		// if let Some(mark) = Some(CellMark::Succeeded) {
		let quad = shape::Quad::new(Vec2::new(
			// width
			CELL_SIZE, // height
			CELL_SIZE,
		));
		let mesh = meshes.add(Mesh::from(quad));

		let mut transform = transform;
		transform.translation += Vec3::Y * CELL_DEPTH / 2.;

		match mark {
			CellMark::Failed => {
				let material_handle = materials.add(StandardMaterial {
					base_color_texture: Some(ass.load("images/XMark.png")),
					alpha_mode: AlphaMode::Blend,
					..default()
				});

				commands.spawn((
					PbrBundle {
						mesh,
						material: material_handle,
						transform,
						..default()
					},
					at,
				));
			}
			CellMark::Succeeded => {
				let material_handle = materials.add(StandardMaterial {
					base_color_texture: Some(ass.load("images/TickMark.png")),
					alpha_mode: AlphaMode::Blend,
					..default()
				});

				commands.spawn((
					PbrBundle {
						mesh,
						material: material_handle,
						transform,
						..default()
					},
					at,
				));
			}
		}
	}
}

/// Called to destroy + create a cell, to reload its marks
pub fn mark_reload_cell(
	cell: &ChessPoint,
	cells: Query<(Entity, &ChessPoint)>,
	alg: &Algorithm,

	ass: &mut ResMut<AssetServer>,
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
) {
	// info!("reloading cell {}", cell);

	cells
		.iter()
		.filter(|(_, cp)| cp == &cell)
		.for_each(|(e, _)| commands.entity(e).despawn_recursive())
}

/// Changes selected cell
fn cell_selected(
	// The first parameter is always the `ListenedEvent`, passed in by the event listening system.
	In(event): In<ListenedEvent<Over>>,

	mut materials: ResMut<Assets<StandardMaterial>>,

	cells: Query<(&Handle<StandardMaterial>, &ChessPoint)>,
	current_options: ResMut<CurrentOptions>,

	mut new_cell_selected: EventWriter<NewCellSelected>,
) -> Bubble {
	let (mat, point) = cells.get(event.target).unwrap();

	let options = &current_options.current.options;
	let is_disabled = options.get_unavailable_points().contains(point);

	if !is_disabled {
		// sets colour to selected
		let material = materials.get_mut(mat).unwrap();
		material.base_color = CELL_SELECTED_COLOUR;

		// send event
		new_cell_selected.send(NewCellSelected { new: *point });
	}

	Bubble::Up
}

/// Just undoes colour change to normal
fn cell_deselected(
	In(event): In<ListenedEvent<Out>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	square: Query<(&Handle<StandardMaterial>, &ChessPoint)>,
	options: Res<CurrentOptions>,
) -> Bubble {
	let (mat, point) = square.get(event.target).unwrap();

	// sets colour to selected
	let material = materials.get_mut(mat).unwrap();
	material.base_color = compute_colour(point, Some(&options.current.options), None);

	Bubble::Up
}

fn toggle_cell_availability(
	In(event): In<ListenedEvent<Click>>,
	// mut materials: ResMut<Assets<StandardMaterial>>,
	cells: Query<(&Handle<StandardMaterial>, &ChessPoint)>,
	current_options: ResMut<CurrentOptions>,

	mut new_board: EventWriter<NewBoardCellOptions>,
) -> Bubble {
	let (mat, point) = cells.get(event.target).unwrap();

	let options = &current_options.current.options;
	match options.get(point) {
		Some(CellOption::Available) => {
			// let material = materials.get_mut(mat).unwrap();
			// material.base_color = CELL_DISABLED_COLOUR;

			new_board.send(NewBoardCellOptions {
				new: options.clone().set(point, CellOption::Unavailable),
			})
		}
		Some(CellOption::Unavailable) => {
			// let material = materials.get_mut(mat).unwrap();
			// material.base_color = point.get_standard_colour();

			new_board.send(NewBoardCellOptions {
				new: options.clone().set(point, CellOption::Available),
			})
		}
		None => (),
	}
	Bubble::Up
}

/// When new cell is selected
pub fn handle_new_cell_selected_event(
	mut new_starting_point: EventReader<NewCellSelected>,
	current_options: ResMut<CurrentOptions>,

	vis: Query<Entity, With<VisualizationComponent>>,
	algs: Res<Algorithm>,

	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let current_options = &current_options.current;
	if let Some(new_starting_point) = new_starting_point.into_iter().next() {
		let new_options = Options {
			options: current_options.options.clone(),
			selected_start: Some(new_starting_point.new),
		};
		commands.insert_resource(CurrentOptions {
			current: new_options.clone(),
		});

		// info!("New starting point: {}", new_starting_point.new);
		despawn_visualization(&mut commands, vis);
		begin_showing_new_visualization(
			&new_options,
			algs,
			&mut commands,
			&mut meshes,
			&mut materials,
		);
	}
}

/// When board dimensions change
#[allow(clippy::too_many_arguments)]
pub fn handle_new_board_event(
	mut new_board: EventReader<NewBoardCellOptions>,

	vis: Query<Entity, With<VisualizationComponent>>,
	cells: Query<Entity, With<ChessPoint>>,

	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut ass: ResMut<AssetServer>,
	alg: Res<Algorithm>,
) {
	if let Some(new_options) = new_board.into_iter().next() {
		let new_options = Options {
			options: new_options.new.clone(),
			selected_start: None,
		};
		commands.insert_resource(CurrentOptions {
			current: new_options.clone(),
		});

		despawn_visualization(&mut commands, vis);
		despawn_cells(&mut commands, cells);

		spawn_cells(
			&mut commands,
			&new_options,
			&mut meshes,
			&mut materials,
			&mut ass,
			Some(alg),
		);
	}
}
