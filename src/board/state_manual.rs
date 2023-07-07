use crate::solver::Moves;

use super::*;

	#[derive(derive_more::Into, derive_more::From, derive_more::Deref, derive_more::DerefMut, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
	pub struct ManualNextCell {
		pub cell: ChessPoint,
	}

	#[derive(Resource, Default, derive_more::Into, derive_more::From, derive_more::Deref, derive_more::DerefMut, Debug, Clone, PartialEq, Eq)]
	pub struct ManualMoves {
		pub moves: Moves,	
	}

	pub fn add_empty_manual_moves(
		mut commands: Commands,
	) {
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