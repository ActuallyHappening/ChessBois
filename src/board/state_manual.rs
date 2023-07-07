use crate::solver::Moves;
use crate::solver::Move;
use super::*;

#[derive(
	derive_more::Into,
	derive_more::From,
	derive_more::Deref,
	derive_more::DerefMut,
	Debug,
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
)]
pub struct ManualNextCell {
	pub cell: ChessPoint,
}

/// Resource for storing manual moves to present visualization
#[derive(Resource, Default, derive_more::Into, derive_more::From, Debug, Clone, PartialEq, Eq)]
pub struct ManualMoves {
	pub start: Option<ChessPoint>,
	pub moves: Moves,
}

pub fn handle_manual_visualization(
	mut commands: Commands,
	options: Res<CurrentOptions>,

	mut manual_moves: ResMut<ManualMoves>,

	visualization: Query<Entity, With<VisualizationComponent>>,
	mut mma: ResSpawning,
) {
	let moves = manual_moves.moves.clone();
	super::visualization::despawn_visualization(&mut commands, visualization);
	spawn_visualization(moves, options.into_inner().current.options.clone(), &mut commands, &mut mma);
}

pub fn handle_new_manual_selected(
	mut events: EventReader<ManualNextCell>,
	mut manual_moves: ResMut<ManualMoves>,
) {
	for ManualNextCell { cell } in events.iter() {
		if manual_moves.start.is_none() {
			manual_moves.start = Some(*cell);
			info!("Manually adding start cell: {:?}", manual_moves.start);
		} else if manual_moves.moves.last().is_none() {
			let from = manual_moves.start.unwrap();
			manual_moves.moves.push(Move::new(from, *cell));
		} else {
			let from = manual_moves.moves.last().unwrap().to;
			manual_moves.moves.push(Move::new(from, *cell));
		}
	}
}

pub fn add_empty_manual_moves(mut commands: Commands) {
	commands.insert_resource(ManualMoves::default());
}

pub fn despawn_visualization(
	mut commands: Commands,
	visualization: Query<Entity, With<VisualizationComponent>>,
) {
	super::visualization::despawn_visualization(&mut commands, visualization);
}

pub fn despawn_markers(
	mut commands: Commands,
	markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,
) {
	super::cells::despawn_markers(&mut commands, markers);
}
