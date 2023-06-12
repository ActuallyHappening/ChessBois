use bevy::prelude::*;
use tracing::{debug, info};

fn main() {
	// Bevy's default plugins include setting up logging
	// bevy_solver::init_debug_tools();

	info!("Bevy app running ...");

	App::new()
		// startup systems
		.add_startup_system(hello_world)
		// plugins	
		.add_plugins(DefaultPlugins)
		// run
		.run();

	debug!("Bevy app finished.");
}

fn hello_world() {
	println!("Hello world!")
}
