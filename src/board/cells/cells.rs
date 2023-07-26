use bevy::prelude::*;
use super::*;

/// Marker for cells
#[derive(Component)]
pub struct CellMarker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, Deref, DerefMut)]
pub struct CellClicked(pub ChessPoint);

impl SharedState {
	pub fn sys_render_cells(
		state: Res<SharedState>,
		mut commands: Commands,
		cells: Query<Entity, (With<CellMarker>, With<ChessPoint>)>,
		mut mma: ResSpawning,
	) {
		despawn_cells(&mut commands, cells);
		spawn_cells(state.into_inner(), &mut commands, &mut mma);
	}
}

fn spawn_cells(state: &SharedState, commands: &mut Commands, mma: &mut ResSpawning) {
	let start = state.start;
	let options = &state.board_options;

	for point in options.get_all_points() {
		let colour = compute_colour(&point, Some(options), start);
		spawn_cell(point, options, colour, commands, mma);
	}
}

fn despawn_cells(
	commands: &mut Commands,
	cells: Query<Entity, (With<CellMarker>, With<ChessPoint>)>,
) {
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

fn spawn_cell(
	at: ChessPoint,
	options: &BoardOptions,
	colour: Color,

	commands: &mut Commands,
	mma: &mut ResSpawning,
) {
	let transform = cell_get_transform(at, options);
	let (meshs, materials, _) = mma;
	let mesh = meshs.add(shape::Box::new(CELL_SIZE, CELL_SIZE, CELL_DEPTH).into());

	let mut cell = commands.spawn((
		PbrBundle {
			mesh,
			transform,
			material: materials.add(StandardMaterial::from(colour)),
			..default()
		},
		Name::new(format!("Chess Square ({}, {})", at.row, at.column)),
		at,
		CellMarker {},
		PickableBundle::default(),    // Makes the entity pickable
		RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
		OnPointer::<Over>::run_callback(cell_hovered),
		OnPointer::<Out>::run_callback(cell_unhovered),
		OnPointer::<Click>::run_callback(cell_clicked),
	));

	// add target symbol
	if options.targets_state().should_show_targets_visual()
		&& options.get(&at)
			== Some(CellOption::Available {
				can_finish_on: true,
			})
	{
		cell.with_children(|parent| {
			let quad = shape::Quad::new(Vec2::new(CELL_SIZE, CELL_SIZE) * 0.5);
			parent
				.spawn(PbrBundle {
					mesh: meshs.add(quad.into()),
					transform: Transform::from_translation(Vec3::Z * 2.),
					material: materials.add(StandardMaterial {
						base_color_texture: Some(mma.2.load("images/TargetSymbol.png")),
						alpha_mode: AlphaMode::Blend,
						..default()
					}),
					..default()
				})
				.insert(CellMarker)
				.name("Target symbol");
		});
	}
}

/// Changes selected cell on hover
fn cell_hovered(
	// The first parameter is always the `ListenedEvent`, passed in by the event listening system.
	In(event): In<ListenedEvent<Over>>,
	options: ResMut<SharedState>,
	cells: Query<(&Handle<StandardMaterial>, &ChessPoint)>,

	mut materials: ResMut<Assets<StandardMaterial>>,
	mut commands: Commands,
) -> Bubble {
	let (mat, point) = cells.get(event.target).unwrap();

	match options.get(point) {
		Some(CellOption::Available { .. }) => {
			// sets colour to selected
			let material = materials.get_mut(mat).unwrap();
			material.base_color = CELL_SELECTED_COLOUR;

			if options.start != Some(*point) {
				let options = options.into_inner();
				options.set_start(*point);
			}
		}
		Some(CellOption::Unavailable) => {
			// unavailable
		}
		None => Error::handle_error(
			LogLevel::Error,
			"Cell selected but no ChessPoint found",
			&mut commands,
		),
	}

	Bubble::Burst
}

/// Just undoes colour change to normal
fn cell_unhovered(
	In(event): In<ListenedEvent<Out>>,
	options: Res<SharedState>,
	square: Query<(&Handle<StandardMaterial>, &ChessPoint)>,

	mut materials: ResMut<Assets<StandardMaterial>>,
) -> Bubble {
	let (mat, point) = square.get(event.target).unwrap();

	// sets colour to selected
	let material = materials.get_mut(mat).unwrap();
	material.base_color = compute_colour(point, Some(&options.board_options), None);

	Bubble::Burst
}

fn cell_clicked(
	In(event): In<ListenedEvent<Click>>,
	mut send_event: EventWriter<CellClicked>,

	cells: Query<&ChessPoint, With<CellMarker>>,

	mut commands: Commands,
) -> Bubble {
	match cells.get(event.target) {
		Ok(point) => {
			send_event.send(CellClicked(*point));
		}
		Err(_) => {
			let err_msg = "Cell clicked but no ChessPoint found";
			commands.insert_resource(Error::new(err_msg.to_owned()));
			panic!("{}", err_msg);
		}
	}
	Bubble::Up
}
