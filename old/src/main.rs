use bevy::prelude::*;
use tracing::{debug, info};

fn main() {
	// Bevy's default plugins include setting up logging
	// bevy_solver::init_debug_tools();

	info!("Bevy app running ...");

	let mut app = App::new();
	app
		// startup systems
		.add_startup_system(hello_world)
		// plugins
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			primary_window: Some(
				Window {
				title: "Bevy Solver".to_string(),

				// #[cfg(not(target_arch = "wasm32"))]
				// mode: WindowMode::BorderlessFullscreen,

				#[cfg(target_arch = "wasm32")]
				// auto-expands parent
				fit_canvas_to_parent: true,

				..default()
			}, // Default::default()
			),
			..default()
		}))
		// .add_plugin(bevy_solver::GraphicsPlugin)
		.add_plugin(bevy_solver::ui::ChessBoardPlugin);

	// .add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
	#[cfg(feature = "editor")]
	app.add_plugin(bevy_editor_pls::prelude::EditorPlugin::default());
	// run
	app.run();

	debug!("Bevy app finished.");
}

fn hello_world() {
	warn!("Hello world!")
}
