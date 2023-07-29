use super::*;

/// Marker for cells
#[derive(Component)]
pub struct CellMarker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, Deref, DerefMut)]
pub struct CellClicked(pub ChessPoint);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, Deref, DerefMut)]
pub struct CellHovered(pub ChessPoint);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, Deref, DerefMut)]
pub struct CellUnhovered(pub ChessPoint);

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
		let colour = compute_colour(&point, state, start);
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
fn compute_colour(point: &ChessPoint, state: &SharedState, start: Option<ChessPoint>) -> Color {
	if state.get_unavailable_points().contains(point) {
		CELL_DISABLED_COLOUR
	} else if Some(*point) == start {
		CELL_SELECTED_COLOUR
	} else if state.visual_opts.show_end_colour && state.moves.as_ref().is_some_and(|moves| {
		moves
			.moves()
			.into_iter()
			.last()
			.is_some_and(|last| last.to == *point)
	}) {
		CELL_END_COLOUR_FACTOR
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
		OnPointer::<Down>::run_callback(cell_clicked),
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
	cells: Query<&ChessPoint, With<CellMarker>>,
	mut send_event: EventWriter<CellHovered>,
) -> Bubble {
	let point = cells.get(event.target).unwrap();

	send_event.send(CellHovered(*point));

	Bubble::Burst
}

/// Just undoes colour change to normal
fn cell_unhovered(
	In(event): In<ListenedEvent<Out>>,
	cells: Query<&ChessPoint>,
	mut send_event: EventWriter<CellUnhovered>,
) -> Bubble {
	let point = cells.get(event.target).unwrap();

	send_event.send(CellUnhovered(*point));

	Bubble::Burst
}

fn cell_clicked(
	In(event): In<ListenedEvent<Down>>,
	mut send_event: EventWriter<CellClicked>,

	cells: Query<&ChessPoint, With<CellMarker>>,
) -> Bubble {
	let point = cells.get(event.target).unwrap();

	send_event.send(CellClicked(*point));

	Bubble::Burst
}
