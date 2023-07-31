use std::sync::Mutex;

use super::{visualization::VisualOpts, *};

/// Marker for cells
#[derive(Component)]
pub struct CellMarker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, Deref, DerefMut)]
pub struct CellClicked(pub ChessPoint);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, Deref, DerefMut)]
pub struct CellHovered(pub ChessPoint);

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, Deref, DerefMut)]
pub struct CellUnhovered(pub ChessPoint);

use cells_state::*;
mod cells_state {
	use super::*;

	/// Used in implementation blocks
	#[derive(PartialEq, Clone)]
	pub struct BorrowedCellsState<'shared> {
		pub board_options: &'shared BoardOptions,
		pub visual_opts: &'shared VisualOpts,
		pub moves: &'shared Option<ColouredMoves>,
		pub start: &'shared Option<ChessPoint>,
	}

	/// Used to store for later comparisons
	#[derive(PartialEq, Clone)]
	pub struct OwnedCellsState {
		pub board_options: BoardOptions,
		pub visual_opts: VisualOpts,
		pub moves: Option<ColouredMoves>,
		pub start: Option<ChessPoint>,
	}

	impl<'shared> BorrowedCellsState<'shared> {
		pub fn new(state: &'shared SharedState) -> Self {
			Self {
				board_options: &state.board_options,
				visual_opts: &state.visual_opts,
				moves: &state.moves,
				start: &state.start,
			}
		}
	}

	impl OwnedCellsState {
		pub fn new(state: SharedState) -> Self {
			Self {
				board_options: state.board_options,
				visual_opts: state.visual_opts,
				moves: state.moves,
				start: state.start,
			}
		}
	}

	impl std::ops::Deref for BorrowedCellsState<'_> {
		type Target = BoardOptions;

		fn deref(&self) -> &Self::Target {
			self.board_options
		}
	}
}

static PREVIOUS_RENDER: Mutex<Option<OwnedCellsState>> = Mutex::new(None);
impl SharedState {
	pub fn sys_render_cells(
		state: Res<SharedState>,

		mut commands: Commands,
		cells: Query<Entity, (With<CellMarker>, With<ChessPoint>)>,
		mut mma: ResSpawning,
	) {
		let state = state.into_inner();
		let current_state = OwnedCellsState::new(state.clone());

		if *PREVIOUS_RENDER.lock().unwrap() != Some(current_state.clone()) {
			despawn_cells(&mut commands, cells);
			spawn_cells(&BorrowedCellsState::new(state), &mut commands, &mut mma);

			PREVIOUS_RENDER.lock().unwrap().replace(current_state);
		} else {
			// info!("Skipping re-rendering cells");
		}
	}
}

fn spawn_cells(state: &BorrowedCellsState, commands: &mut Commands, mma: &mut ResSpawning) {
	let start = state.start.as_ref();
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
fn compute_colour(point: &ChessPoint, state: &BorrowedCellsState, start: Option<&ChessPoint>) -> Color {
	if state.get_unavailable_points().contains(point) {
		CELL_DISABLED_COLOUR
	} else if Some(point) == start {
		CELL_SELECTED_COLOUR
	} else if state.visual_opts.show_end_colour
		&& state.moves.as_ref().is_some_and(|moves| {
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
