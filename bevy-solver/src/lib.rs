use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};

pub mod ui;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(setup)
			.add_startup_system(setup_shapes)
// .add_system(my_cursor_system)
.add_plugin(ui::UiPlugin)
			// for formatting	
			;
	}
}

#[derive(Component, Debug)]
pub struct ChessSquare {
	pub x: u8,
	pub y: u8,
}

#[derive(Component, Debug)]
struct MainCamera;

fn setup(mut commands: Commands) {
	commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn get_random_colour() -> Color {
	Color::rgb(
		(rand::random::<f32>() * 255.) as f32,
		(rand::random::<f32>() * 255.) as f32,
		(rand::random::<f32>() * 255.) as f32,
	)
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

// fn my_cursor_system(
// 	// need to get window dimensions
// 	window_q: Query<&Window, With<PrimaryWindow>>,

// 	// query to get camera transform
// 	camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
// ) {
// 	// get the camera info and transform
// 	// assuming there is exactly one main camera entity, so query::single() is OK
// 	let (camera, camera_transform) = camera_q.single();

// 	let window: &Window = window_q.single();

// 	// check if the cursor is inside the window and get its position
// 	// then, ask bevy to convert into world coordinates, and truncate to discard Z
// 	if let Some(world_position) = window
// 		.cursor_position()
// 		.and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
// 		.map(|ray| ray.origin.truncate())
// 	{
// 		eprintln!("World coords: {}/{}", world_position.x, world_position.y);
// 	}
// }
