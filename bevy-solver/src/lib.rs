use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(setup_cam)
			.add_startup_system(setup_shapes);
	}
}

#[derive(Component, Debug)]
pub struct ChessSquare {
	pub x: u8,
	pub y: u8,
}

fn setup_cam(mut commands: Commands) {
	commands.spawn(Camera2dBundle::default());
}

fn setup_shapes(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	// Circle
	commands.spawn(MaterialMesh2dBundle {
		mesh: meshes.add(shape::Circle::new(50.).into()).into(),
		material: materials.add(ColorMaterial::from(Color::PURPLE)),
		transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
		..default()
	});

	// Rectangle
	commands.spawn(SpriteBundle {
		sprite: Sprite {
			color: Color::rgb(0.25, 0.25, 0.75),
			custom_size: Some(Vec2::new(50.0, 100.0)),
			..default()
		},
		transform: Transform::from_translation(Vec3::new(-50., 0., 0.)),
		..default()
	});

	// Quad
	commands.spawn(MaterialMesh2dBundle {
		mesh: meshes
			.add(shape::Quad::new(Vec2::new(50., 100.)).into())
			.into(),
		material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
		transform: Transform::from_translation(Vec3::new(50., 0., 0.)),
		..default()
	});

	// Hexagon
	commands.spawn(MaterialMesh2dBundle {
		mesh: meshes.add(shape::RegularPolygon::new(50., 6).into()).into(),
		material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
		transform: Transform::from_translation(Vec3::new(150., 0., 0.)),
		..default()
	});
}

// #[cfg(target_arch = "wasm32")]
// pub fn init_debug_tools() {
// 	use tracing_subscriber::fmt::format::Pretty;
// 	use tracing_subscriber::fmt::time::UtcTime;
// 	use tracing_subscriber::prelude::*;
// 	use tracing_web::{performance_layer, MakeConsoleWriter};

// 	console_error_panic_hook::set_once();

// 	let fmt_layer = tracing_subscriber::fmt::layer()
// 			.with_ansi(false) // Only partially supported across browsers
// 			.with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
// 			.with_writer(MakeConsoleWriter) // write events to the console
// 			// .with_span_events(FmtSpan::ACTIVE)
// 		;
// 	let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

// 	tracing_subscriber::registry()
// 		.with(fmt_layer)
// 		.with(perf_layer)
// 		.init(); // Install these as subscribers to tracing events
// }

// #[cfg(not(target_arch = "wasm32"))]
// pub fn init_debug_tools() {
// 	use tracing::Level;
// 	use tracing_subscriber::FmtSubscriber;
// 	let subscriber = FmtSubscriber::builder()
// 		.with_max_level(Level::INFO)
// 		.finish();
// 	tracing::subscriber::set_global_default(subscriber).unwrap();
// }
