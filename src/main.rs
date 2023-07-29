use bevy::prelude::*;
use cap_solver::*;

fn main() {
	let mut app = App::new();
	app
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
		.add_plugin(ChessSolverPlugin);

	#[cfg(feature = "dev")]
	app.add_plugin(bevy_editor_pls::prelude::EditorPlugin::default());

	app.run();
}
