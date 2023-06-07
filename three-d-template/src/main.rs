use three_d::*;

pub fn init_debug_tools() {
	cfg_if::cfg_if! { if	#[cfg(target_arch = "wasm32")] {
		use tracing_subscriber::fmt::format::Pretty;
		use tracing_subscriber::fmt::time::UtcTime;
		use tracing_subscriber::prelude::*;
		use tracing_web::{performance_layer, MakeConsoleWriter};

		console_error_panic_hook::set_once();

		let fmt_layer = tracing_subscriber::fmt::layer()
			.with_ansi(false) // Only partially supported across browsers
			.with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
			.with_writer(MakeConsoleWriter) // write events to the console
			// .with_span_events(FmtSpan::ACTIVE)
		;
		let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

		tracing_subscriber::registry()
			.with(fmt_layer)
			.with(perf_layer)
			.init(); // Install these as subscribers to tracing events
	} else {
		use tracing::Level;
		use tracing_subscriber::FmtSubscriber;
		let subscriber = FmtSubscriber::builder()
			.with_max_level(Level::TRACE)
			.finish();
		tracing::subscriber::set_global_default(subscriber).unwrap();
	}}
}

pub fn main() {
	init_debug_tools();

	let window = Window::new(WindowSettings {
		title: "Shapes!".to_string(),
		max_size: Some((1280, 720)),
		..Default::default()
	})
	.unwrap();
	let context = window.gl();

	let mut camera = Camera::new_perspective(
		window.viewport(),
		vec3(5.0, 2.0, 2.5),
		vec3(0.0, 0.0, -0.5),
		vec3(0.0, 1.0, 0.0),
		degrees(45.0),
		0.1,
		1000.0,
	);
	let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

	let mut sphere = Gm::new(
		Mesh::new(&context, &CpuMesh::sphere(16)),
		PhysicalMaterial::new_transparent(
			&context,
			&CpuMaterial {
				albedo: Color {
					r: 255,
					g: 0,
					b: 0,
					a: 200,
				},
				..Default::default()
			},
		),
	);
	sphere.set_transformation(Mat4::from_translation(vec3(0.0, 1.3, 0.0)) * Mat4::from_scale(0.2));
	let mut cylinder = Gm::new(
		Mesh::new(&context, &CpuMesh::cylinder(16)),
		PhysicalMaterial::new_transparent(
			&context,
			&CpuMaterial {
				albedo: Color {
					r: 0,
					g: 255,
					b: 0,
					a: 200,
				},
				..Default::default()
			},
		),
	);
	cylinder.set_transformation(Mat4::from_translation(vec3(1.3, 0.0, 0.0)) * Mat4::from_scale(0.2));
	let mut cube = Gm::new(
		Mesh::new(&context, &CpuMesh::cube()),
		PhysicalMaterial::new_transparent(
			&context,
			&CpuMaterial {
				albedo: Color {
					r: 0,
					g: 0,
					b: 255,
					a: 100,
				},
				..Default::default()
			},
		),
	);
	cube.set_transformation(Mat4::from_translation(vec3(0.0, 0.0, 1.3)) * Mat4::from_scale(0.2));
	let axes = Axes::new(&context, 0.1, 2.0);
	let bounding_box_sphere = Gm::new(
		BoundingBox::new(&context, sphere.aabb()),
		ColorMaterial {
			color: Color::BLACK,
			..Default::default()
		},
	);
	let bounding_box_cube = Gm::new(
		BoundingBox::new(&context, cube.aabb()),
		ColorMaterial {
			color: Color::BLACK,
			..Default::default()
		},
	);
	let bounding_box_cylinder = Gm::new(
		BoundingBox::new(&context, cylinder.aabb()),
		ColorMaterial {
			color: Color::BLACK,
			..Default::default()
		},
	);

	let light0 = DirectionalLight::new(&context, 1.0, Color::WHITE, &vec3(0.0, -0.5, -0.5));
	let light1 = DirectionalLight::new(&context, 1.0, Color::WHITE, &vec3(0.0, 0.5, 0.5));

	window.render_loop(move |mut frame_input| {
		camera.set_viewport(frame_input.viewport);
		control.handle_events(&mut camera, &mut frame_input.events);

		frame_input
			.screen()
			.clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
			.render(
				&camera,
				sphere
					.into_iter()
					.chain(&cylinder)
					.chain(&cube)
					.chain(&axes)
					.chain(&bounding_box_sphere)
					.chain(&bounding_box_cube)
					.chain(&bounding_box_cylinder),
				&[&light0, &light1],
			);

		FrameOutput::default()
	});
}
