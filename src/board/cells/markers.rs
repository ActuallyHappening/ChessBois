
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