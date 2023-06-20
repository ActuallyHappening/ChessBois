use bevy::prelude::*;
use bevy_solver::*;
use msrc_q11::ChessPoint;

fn main() {
	App::new()
    .register_type::<ChessPoint>()
		.add_plugins(DefaultPlugins)
		.add_plugin(bevy_editor_pls::prelude::EditorPlugin::default())
		.add_startup_system(setup)
		.run();
}