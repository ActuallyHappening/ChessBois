use bevy::prelude::*;
use cap_solver::*;

fn main() {
	#[cfg(not(target_arch = "wasm32"))]
	main2(None);

	#[cfg(any(target_arch = "wasm32", feature = "web-start"))]
	{
		if let Some(id) = weburl::get_url_id() {
			info!("Loaded id from URL: {:?}", id);

			let url =
				format!(
			"https://chess-analysis-program-default-rtdb.asia-southeast1.firebasedatabase.app/{}/{}.json", *meta::VERSION_APPEND, id);

			wasm_bindgen_futures::spawn_local(async {
				if let Ok(data) = reqwest::get(url).await {
					if let Ok(data) = data.json().await {
						main2(Some(data));
						return;
					}
				}
				main2(None);
			});
		} else {
			main2(None);
		}
	}
}

fn main2(data: Option<serde_json::Value>) {
	let mut app = App::new();

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

	if let Some(data) = data {
		app.insert_resource(weburl::InitialLoadedID::new(data));
	}

	app.run();
}
