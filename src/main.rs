use bevy::prelude::*;
use bevy_solver::ChessPoint;
use bevy_solver::*;

fn main() {
	App::new()
		.register_type::<ChessPoint>()
		.add_plugins(
			DefaultPlugins
				.set(WindowPlugin {
					primary_window: Some(Window {
						fit_canvas_to_parent: true,
						prevent_default_event_handling: false,
						canvas: Some("#canvas".to_string()),
						..default()
					}),
					..default()
				})
				.build(),
		)
		.add_plugin(bevy_editor_pls::prelude::EditorPlugin::default())
		.add_plugin(ChessSolverPlugin)
		.run();
}
