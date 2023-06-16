use bevy::prelude::*;

use super::ChessEngineState;

pub struct UiStatePlugin;
impl Plugin for UiStatePlugin {
	fn build(&self, app: &mut App) {
		app
			.register_type::<ChessSquare>()
			.add_startup_system(spawn_chess_board)
			// show-hide chess board
			.add_system(on_chess_square_clicked.in_set(OnUpdate(ChessEngineState::PickStartingPosition)))
			.add_system(change_to_pick_state.in_schedule(OnEnter(ChessEngineState::PickStartingPosition)))
			.add_system(change_to_view_state.in_schedule(OnExit(ChessEngineState::PickStartingPosition)))
			// resources
			.init_resource::<ChessSquareSelected>()
			// -
			;
	}
}

#[derive(Component, Copy, Clone, Debug, Reflect)]
struct ChessSquare {
	pub x: u8,
	pub y: u8,
}

/// Chess square selected resource
#[derive(Resource, Copy, Clone, Debug, Default)]
struct ChessSquareSelected {
	pub selected: Option<ChessSquare>,
}

#[derive(Debug, Reflect, Clone)]
struct ChessBoardProperties {
	width: u8,
	height: u8,
}

/// What is shown-hidden when state changes
#[derive(Component)]
struct ChessBoardUIMarker;

#[derive(Component)]
struct UIHeader;

trait SizeExt {
	fn with_width(self, val: Val) -> Size;
}

impl SizeExt for Size {
	fn with_width(mut self, val: Val) -> Size {
		self.width = val;
		self
	}
}

// trait ValExt {
// 	fn full_percent() -> Self;
// }

// impl ValExt for Val {
// 	fn full_percent() -> Self {
// 			Val::Percent(100.)
// 	}
// }

trait UiRectExt {
	fn with_horizontal(self, val: Val) -> Self;
}

impl UiRectExt for UiRect {
	/// Sets both .left and .right
	fn with_horizontal(mut self, val: Val) -> Self {
		self.left = val;
		self.right = val;
		self
	}
}

fn change_to_pick_state(
	mut chess_board: Query<&mut Visibility, With<ChessBoardUIMarker>>,
	mut header: Query<&mut Text, With<UIHeader>>,
) {
	info!("Changing to pick state");
	for mut visibility in chess_board.iter_mut() {
		info!("Showing chess board");
		*visibility = Visibility::Visible;
	}

	for mut text in header.iter_mut() {
		text.sections[0].value = "Select start position on board".to_string();
	}
}

fn change_to_view_state(
	mut chess_board: Query<&mut Visibility, With<ChessBoardUIMarker>>,
	mut header: Query<&mut Text, With<UIHeader>>,
) {
	info!("Changing to view state");
	for mut visibility in chess_board.iter_mut() {
		info!("Hiding chess board");
		*visibility = Visibility::Hidden;
	}

	for mut text in header.iter_mut() {
		text.sections[0].value = "Visualizing movements ...".to_string();
	}
}

fn on_chess_square_clicked(
	mut commands: Commands,
	mut interaction_query: Query<
		(&Interaction, &mut BackgroundColor, &ChessSquare),
		(Changed<Interaction>, With<Button>),
	>,
	mut next_state: ResMut<NextState<ChessEngineState>>,
) {
	for (interaction, mut color, square) in &mut interaction_query {
		match *interaction {
			Interaction::Clicked => {
				*color = PRESSED_BUTTON.into();
				commands.insert_resource(ChessSquareSelected {
					selected: Some(*square),
				});
				next_state.set(ChessEngineState::ViewValidPaths);
			}
			Interaction::Hovered => {
				*color = HOVERED_BUTTON.into();
			}
			Interaction::None => {
				*color = NORMAL_BUTTON.into();
			}
		}
	}
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn spawn_chess_board(mut commands: Commands, ass: Res<AssetServer>) {
	commands
		.spawn((
			NodeBundle {
				style: Style {
					min_size: Size::all(Val::Percent(100.)),
					flex_direction: FlexDirection::Column,
					..default()
				},
				background_color: TRANSPARENT.into(),
				..default()
			},
			Name::from("Screen UI"),
		))
		.with_children(|parent| {
			parent
				.spawn((
					NodeBundle {
						style: Style {
							// size: Size::height(Val::Percent(10.)).with_width(Val::Percent(100.)),
							margin: UiRect::vertical(Val::Percent(5.)).with_horizontal(Val::Percent(15.)),
							size: Size::AUTO,
							..EXPAND_STYLE.clone()
						},
						background_color: Color::GRAY.into(),
						..default()
					},
					Name::from("Header <div>"),
				))
				.with_children(|parent| {
					parent.spawn((
						TextBundle::from_section(
							"Select start position on board",
							TextStyle {
								color: Color::WHITE,
								font: ass.load("fonts/FiraMono-Medium.ttf"),
								font_size: 42., // ..default()
							},
						)
						.with_style(EXPAND_STYLE.clone()),
						// .with_background_color(Color::GREEN),
						Name::new("Header text"),
						UIHeader,
					));
				});

			// -- chess board ==
			parent
				.spawn((
					NodeBundle {
						// background_color: Color::WHITE.into(),
						style: Style {
							..EXPAND_STYLE.clone()
						},
						visibility: Visibility::Hidden,
						..default()
					},
					Name::new("Chess Board"),
					ChessBoardUIMarker,
				))
				.with_children(|parent| {
					for y in 1..=8 {
						parent
							.spawn((
								NodeBundle {
									// background_color: Color::YELLOW.into(),
									style: Style {
										flex_direction: FlexDirection::Column,
										// margin: UiRect::bottom(Val::Px(10.)),
										justify_content: JustifyContent::Center,
										align_items: AlignItems::Center,
										..default()
									},
									..default()
								},
								Name::new(format!("Row {}", y)),
							))
							.with_children(|parent| {
								for x in 1..=8 {
									parent.spawn((
										ButtonBundle {
											background_color: Color::BLACK.into(),
											style: Style {
												margin: UiRect::all(Val::Px(2.)),
												min_size: Size::all(Val::Px(25.)),
												..default()
											},
											..default()
										},
										ChessSquare { x, y },
										Name::new(format!("Square ({}, {})", x, y)),
									));
								}
							});
					}
				});
		});
	// -
}

static TRANSPARENT: Color = Color::Rgba {
	red: 0.,
	green: 0.,
	blue: 0.,
	alpha: 0.,
};

static EXPAND_STYLE: Style = Style {
	size: Size::new(Val::Percent(100.), Val::Percent(100.)),
	justify_content: JustifyContent::Center,
	align_items: AlignItems::Center,
	..Style::DEFAULT
};

// const CELL_MARGIN: f32 = 10.;
// /// Style applied directly to row
// static ROWS: Style = Style {
// 	flex_direction: FlexDirection::Row,
// 	// margin: UiRect::all(Val::Px(10.)),
// 	// margin: UiRect {
// 	// 	top: Val::Px(CELL_MARGIN),
// 	// 	bottom: Val::Px(CELL_MARGIN),
// 	// 	..UiRect::DEFAULT
// 	// },
// 	..EXPAND_STYLE
// };

// /// Style for container of rows
// static COLUMNS: Style = Style {
// 	flex_direction: FlexDirection::Column,
// 	// margin: UiRect::all(Val::Px(10.)),
// 	..EXPAND_STYLE
// };

// const CELL_SIZE: f32 = 10.;
// static CELL_STYLE: Style = Style {
// 	min_size: Size::new(Val::Px(CELL_SIZE), Val::Px(CELL_SIZE)),
// 	aspect_ratio: Some(1.),
// 	// margin: UiRect::all(Val::Px(CELL_SIZE / 2.)),
// 	// margin: UiRect {
// 	// 	left: Val::Px(CELL_MARGIN),
// 	// 	right: Val::Px(CELL_MARGIN),
// 	// 	..UiRect::DEFAULT
// 	// },
// 	..EXPAND_STYLE
// };
