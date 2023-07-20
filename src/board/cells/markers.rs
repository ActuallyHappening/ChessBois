use super::*;

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

		let asset_path = format!(
			"images/{}.png",
			match mark {
				CellMark::Succeeded => "TickMark",
				CellMark::Failed => "XMark",
				CellMark::GivenUp => "WarningMark",
			}
		);
		let material_handle = materials.add(StandardMaterial {
			base_color_texture: Some(ass.load(asset_path)),
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
