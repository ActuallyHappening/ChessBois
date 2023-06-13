use bevy::prelude::*;

pub struct ChessBoardPlugin;
impl Plugin for ChessBoardPlugin {
	fn build(&self, app: &mut App) {
		app
			// types
    .register_type::<ChessSquare>()
    .register_type::<ChessEngineState>()
		// states
    .add_state::<ChessEngineState>()
		// systems
    .add_startup_system(setup)
    .add_startup_system(spawn_chess_board)
		// for formatting
		;
	}
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2dBundle::default());
}

#[derive(Component, Copy, Clone, Debug, Reflect)]
struct ChessSquare {
	pub x: u8,
	pub y: u8,
}

#[derive(Debug, Reflect, Clone)]
struct ChessBoardProperties {
	width: u8,
	height: u8,
}

#[derive(States, Default, Reflect, Debug, Clone, Eq, PartialEq, Hash)]
enum ChessEngineState {
	#[default]
	PickStartingPosition,

	ViewValidPaths,
}

fn spawn_chess_board(mut commands: Commands) {
	commands.spawn((
		div(Style {
			// justify_content: JustifyContent::FlexStart,
			// align_items: AlignItems::FlexStart,
			..EXPAND_STYLE.clone()
		}, Color::WHITE),
		Name::new("Screen UI"),
	))
	// for formatting
	.with_children(|parent| {
		parent.spawn((
			div(EXPAND_STYLE.clone(), Color::GRAY),
			Name::new("Chess Board Space"),
		))
		.with_children(|parent| {
			build_grid(parent, &ChessBoardProperties { width: 8, height: 8 }, |parent, square| {
				parent.spawn((div(CELL_STYLE.clone(), Color::GREEN), square, Name::new(format!("Cell ({}, {})", square.x, square.y))));
			});
		});
	})
	// for formatting
	;
}

static EXPAND_STYLE: Style = Style {
	size: Size::new(Val::Percent(100.), Val::Percent(100.)),
	justify_content: JustifyContent::Center,
	align_items: AlignItems::Center,
	..Style::DEFAULT
};

const CELL_MARGIN: f32 = 10.;
/// Style applied directly to row
static ROW_STYLE: Style = Style {
	flex_direction: FlexDirection::Row,
	// margin: UiRect::all(Val::Px(10.)),
	margin: UiRect {
		top: Val::Px(CELL_MARGIN),
		bottom: Val::Px(CELL_MARGIN),
		..UiRect::DEFAULT
	},
	..EXPAND_STYLE
};

/// Style for container of rows
static ROWS_STYLE: Style = Style {
	flex_direction: FlexDirection::Column,
	// margin: UiRect::all(Val::Px(10.)),
	..EXPAND_STYLE
};

const CELL_SIZE: f32 = 10.;
static CELL_STYLE: Style = Style {
	min_size: Size::new(Val::Px(CELL_SIZE), Val::Px(CELL_SIZE)),
	aspect_ratio: Some(1.),
	// margin: UiRect::all(Val::Px(CELL_SIZE / 2.)),
	margin: UiRect {
		left: Val::Px(CELL_MARGIN),
		right: Val::Px(CELL_MARGIN),
		..UiRect::DEFAULT
	},
	..EXPAND_STYLE
};

fn div(style: Style, color: Color) -> NodeBundle {
	NodeBundle {
		style,
		background_color: color.into(),
		..default()
	}
}

/// Adds rows (and chess squares into those rows) directly onto the parent
fn build_grid(
	parent: &mut ChildBuilder,
	properties: &ChessBoardProperties,
	mut builder: impl FnMut(&mut ChildBuilder, ChessSquare),
) {
	parent
		.spawn((div(ROWS_STYLE.clone(), Color::BLACK), Name::from("Grid Parent (contains rows)")))
		.with_children(|parent| {
			for row in (0..properties.height).rev() {
				parent
					.spawn((
						div(ROW_STYLE.clone(), Color::WHITE),
						Name::new(format!("Row {}", row)),
					))
					.with_children(|parent| {
						for column in 0..properties.width {
							builder(parent, ChessSquare { x: column, y: row });
						}
					});
			};
		});
}
