use bevy::prelude::*;
use cap_solver::*;

fn main() {
	#[cfg(not(target_arch = "wasm32"))]
	main2(None);

	#[cfg(all(target_arch = "wasm32", feature = "web-start"))]
	{
		let id = weburl::get_url_id();
		info!("Loaded id from URL: {:?}", id);

		let url = format!(
			"https://chess-analysis-program-default-rtdb.asia-southeast1.firebasedatabase.app/{}.json", id);

		wasm_bindgen_futures::spawn_local(async {
			let data = reqwest::get(URL).await.unwrap().text().await.unwrap();

			main2(data)
		});
	}
}

fn main2(data: Option<String>) {
	let mut app = App::new();

	info!("App main running with data: {:?}", data);

	app
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
