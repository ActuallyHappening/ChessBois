use bevy::prelude::*;
use bevy_solver::*;
use bevy_solver::ChessPoint;

fn main() {
	App::new()
		.register_type::<ChessPoint>()
		.add_plugins(DefaultPlugins)
		.add_plugin(bevy_editor_pls::prelude::EditorPlugin::default())
		.add_plugin(ChessSolverPlugin)
		.run();
}