use std::f32::consts::TAU;

use super::automatic::cached_info::CellMark;
use super::*;
use crate::board::automatic::cached_info;
use crate::errors::Error;
use crate::*;
use crate::{ChessPoint, CELL_DISABLED_COLOUR};
use derive_more::{From, Into};

mod coords;
pub use coords::{get_spacial_coord, get_spacial_coord_2d};

/// Marker for Markers lol
#[derive(Component)]
pub struct MarkerMarker;

/// Marker for cells
#[derive(Component)]
pub struct CellMarker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, Deref, DerefMut)]
pub struct CellClicked(pub ChessPoint);

// pub fn refresh_cells_on_new_options(options: Res<CurrentOptions>) {
// 	if options.is_changed() {
// 		warn!("Refreshing all cells ...")
// 	}
// }

pub fn spawn_cells(options: &Options, commands: &mut Commands, mma: &mut ResSpawning) {
	let start = options.selected_start;
	let options = options.options.clone();

	for point in options.get_all_points() {
		let colour = compute_colour(&point, Some(&options), start);
		spawn_cell(point, &options, colour, commands, mma);
	}
}

pub fn spawn_markers(options: &Options, commands: &mut Commands, mma: &mut ResSpawning) {
	for point in options.options.get_all_points() {
		spawn_mark(
			point,
			options,
			cell_get_transform(point, &options.options),
			commands,
			mma,
		);
	}
}

pub fn despawn_cells(
	commands: &mut Commands,
	cells: Query<Entity, (With<CellMarker>, With<ChessPoint>)>,
) {
	for cell in cells.iter() {
		commands.entity(cell).despawn_recursive();
	}
}

pub fn despawn_markers(
	commands: &mut Commands,
	markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,
) {
	for mark in markers.iter() {
		commands.entity(mark).despawn_recursive();
	}
}

pub fn sys_despawn_markers(
	mut commands: Commands,
	markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,
) {
	super::cells::despawn_markers(&mut commands, markers);
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
	commands: &mut Commands,
	mma: &mut ResSpawning,
) {
	let transform = cell_get_transform(at, options);
	let (meshes, materials, _) = mma;
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
		CellMarker {},
		PickableBundle::default(),    // Makes the entity pickable
		RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
		OnPointer::<Over>::run_callback(cell_selected),
		OnPointer::<Out>::run_callback(cell_deselected),
		OnPointer::<Click>::run_callback(toggle_cell_availability),
	));
}

fn spawn_mark(
	at: ChessPoint,
	options: &Options,
	cell_transform: Transform,

	commands: &mut Commands,
	(meshes, materials, ass): &mut ResSpawning,
) {
	if let Some(mark) = cached_info::get(&options.with_start(at)) {
		let quad = shape::Quad::new(Vec2::new(CELL_SIZE, CELL_SIZE) * 0.7);
		let mesh = meshes.add(Mesh::from(quad));

		let mut transform = cell_transform;
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
					MarkerMarker {},
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
					MarkerMarker {},
				));
			}
			CellMark::GivenUp => {
				let material_handle = materials.add(StandardMaterial {
					base_color_texture: Some(ass.load("images/WarningMark.png")),
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
					MarkerMarker {},
				));
			}
		}
	}
}

/// Changes selected cell
fn cell_selected(
	// The first parameter is always the `ListenedEvent`, passed in by the event listening system.
	In(event): In<ListenedEvent<Over>>,
	options: ResMut<CurrentOptions>,
	cells: Query<(&Handle<StandardMaterial>, &ChessPoint)>,

	mut materials: ResMut<Assets<StandardMaterial>>,
) -> Bubble {
	let (mat, point) = cells.get(event.target).unwrap();

	let is_disabled = options
		.as_options()
		.options
		.get_unavailable_points()
		.contains(point);
	if !is_disabled {
		// sets colour to selected
		let material = materials.get_mut(mat).unwrap();
		material.base_color = CELL_SELECTED_COLOUR;

		let mut options = options.into_inner();
		options.current.selected_start = Some(*point);
	}

	Bubble::Burst
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

	Bubble::Burst
}

fn toggle_cell_availability(
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
			let err_msg = "Cell clicked but no ChessPoint found".to_string();
			commands.insert_resource(Error::new(err_msg));
		}
	}
	Bubble::Up
}
