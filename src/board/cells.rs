use super::{cached_info::CellMark, *};
use crate::CELL_DISABLED_COLOUR;
use msrc_q11::CellOption;

#[derive(Component)]
pub struct CellMarkMarker;

pub fn spawn_cells(
	commands: &mut Commands,
	options: &Options,
	alg: Option<&Algorithm>,
	mma: &mut ResSpawning,
) {
	let start = options.selected_start;
	let options = options.options.clone();

	for point in options.get_all_points() {
		let colour = compute_colour(&point, Some(&options), start);
		spawn_cell(
			point, &options, colour, alg, commands, mma,
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

fn cell_get_transform(at: ChessPoint, options: &BoardOptions) -> Transform {
	Transform::from_translation(get_spacial_coord(options, at))
		.with_rotation(Quat::from_rotation_x(-TAU / 4.))
}

fn spawn_cell(
	at: ChessPoint,
	options: &BoardOptions,
	colour: Color,
	alg: Option<&Algorithm>,
	commands: &mut Commands,
	(meshes, materials, ass): &mut ResSpawning,
) {
	let transform = cell_get_transform(at, options);
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

	spawn_mark(
		at, options, transform, alg, commands, (meshes, materials, ass),
	);
}

fn spawn_mark(
	at: ChessPoint,
	options: &BoardOptions,
	transform: Transform,
	alg: Option<&Algorithm>,

	commands: &mut Commands,
	(meshes, materials, ass): &mut ResSpawning,
) {
	if let Some(alg) = alg {
		if let Some(mark) = cached_info::get(&at, options, alg) {
			// if let Some(mark) = Some(CellMark::Succeeded) {
			let quad = shape::Quad::new(Vec2::new(
				CELL_SIZE,
				CELL_SIZE,
			) / 0.9);
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
						CellMarkMarker {},
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
						CellMarkMarker {},
					));
				}
			}
		}
	}
}

/// Called to destroy + create a cell, to reload its marks
pub fn mark_reload_cell(
	cell: &ChessPoint,
	marks: Query<(Entity, &ChessPoint), With<CellMarkMarker>>,
	alg: &Algorithm,
	options: &Options,

	commands: &mut Commands,
	mma: &mut ResSpawning
) {
	info!("reloading cell {}", cell);

	// remove
	marks
		.iter()
		.filter(|(_, cp)| cp == &cell)
		.inspect(|g| println!("G: {g:?}"))
		.for_each(|(e, _)| commands.entity(e).despawn_recursive());

	let colour = compute_colour(cell, Some(&options.options), options.selected_start);
	let transform = cell_get_transform(*cell, &options.options);
	spawn_mark(
		*cell,
		&options.options,
		transform,
		Some(alg),
		commands,
		mma
	);
}

/// Changes selected cell
fn cell_selected(
	// The first parameter is always the `ListenedEvent`, passed in by the event listening system.
	In(event): In<ListenedEvent<Over>>,
	options: Res<CurrentOptions>,
	cells: Query<(&Handle<StandardMaterial>, &ChessPoint)>,
	mut update_board: EventWriter<NewOptions>,

	mut materials: ResMut<Assets<StandardMaterial>>,
) -> Bubble {
	let (mat, point) = cells.get(event.target).unwrap();

	let is_disabled = options.into_options().get_unavailable_points().contains(point);

	if !is_disabled {
		// sets colour to selected
		// let material = materials.get_mut(mat).unwrap();
		// material.base_color = CELL_SELECTED_COLOUR;

		let mut options = options.clone().into_options();
		options.selected_start = Some(*point);

		// send event
		update_board.send(NewOptions::from_options(options));
	}

	Bubble::Up
}

/// Just undoes colour change to normal
fn cell_deselected(
	In(event): In<ListenedEvent<Out>>,
	options: Res<CurrentOptions>,
	square: Query<(&Handle<StandardMaterial>, &ChessPoint)>,

	mut materials: ResMut<Assets<StandardMaterial>>,
) -> Bubble {
	let (mat, point) = square.get(event.target).unwrap();

	// sets colour to selected
	let material = materials.get_mut(mat).unwrap();
	material.base_color = compute_colour(point, Some(&options.current.options), None);

	Bubble::Up
}

fn toggle_cell_availability(
	In(event): In<ListenedEvent<Click>>,
	cells: Query<(&Handle<StandardMaterial>, &ChessPoint)>,
	options: ResMut<CurrentOptions>,

	mut update_board: EventWriter<NewOptions>,
) -> Bubble {
	let (mat, point) = cells.get(event.target).unwrap();

	let mut options = options.current.clone();
	match options.get(point) {
		Some(CellOption::Available) => {
			// let material = materials.get_mut(mat).unwrap();
			// material.base_color = CELL_DISABLED_COLOUR;

			options.options.rm(*point);
			update_board.send(NewOptions::from_options(options));
		}
		Some(CellOption::Unavailable) => {
			// let material = materials.get_mut(mat).unwrap();
			// material.base_color = point.get_standard_colour();

			options.options.add(*point);
			update_board.send(NewOptions::from_options(options));
		}
		None => panic!("Tried to change availability of cell that doesn't exist"),
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
