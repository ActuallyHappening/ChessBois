use bevy::prelude::*;

const CHESS_PIECE_HOVER_BG_COLOUR: Color = Color::rgb(0.5, 0.5, 0.5);
const CHESS_PIECE_SELECTED_BG_COLOUR: Color = Color::rgb(0.25, 0.25, 0.25);
const CHESS_PIECE_BG_COLOUR: Color = Color::rgb(0.0, 0.0, 0.0);

pub struct UiPlugin;
impl Plugin for UiPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(spawn_ui)
			// .add_system(despawn_ui)
			.add_system(hover_chess_piece)
			.add_state::<UiState>()
			// for formatting
			.register_type::<ChessSquare>();
	}
}

#[derive(Component, Debug, Reflect)]
pub struct ChessSquare {
	pub x: u8,
	pub y: u8,
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum UiState {
	#[default]
	PickStartingPosition,

	ViewPaths,
}

#[derive(Resource, Debug, Clone)]
struct ChessBoard {
	starting_position: Entity,
}

#[derive(Component, Debug)]
struct Header {
}

impl ChessBoard {
	pub fn new_with_starting_position(entity: Entity, commands: &mut Commands) -> Self {
		let this = Self {
			starting_position: entity,
		};
		commands.insert_resource(this.clone());
		this
	}
}

pub fn spawn_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
	let main_menu_entity = build_ui(&mut commands, &asset_server);
}

fn random_colour() -> Color {
	Color::rgb(
		rand::random::<f32>(),
		rand::random::<f32>(),
		rand::random::<f32>(),
	)
}

// pub fn despawn_ui(commands: &mut Commands, main_menu_entity: Entity) {
// 	commands.despawn_recursive(main_menu_entity);
// }

fn header_select_start_pos(
	mut commands: Commands,
	header_q: Q,
) {
	
}

fn hover_chess_piece(
	mut commands: Commands,
	mut interaction_query: Query<
		(&Interaction, &ChessSquare, &mut BackgroundColor),
		Changed<Interaction>,
	>,
) {
	for (interaction, chess_square, mut bg) in interaction_query.iter_mut() {
		match *interaction {
			Interaction::Clicked => {
				info!("Clicked on {:?}", chess_square);
			}
			Interaction::Hovered => {
				*bg = CHESS_PIECE_HOVER_BG_COLOUR.into();
			}
			Interaction::None => {
				*bg = CHESS_PIECE_BG_COLOUR.into();
			}
		}
	}
}

pub fn build_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
	let mut header = None;
	let main_menu_entity = commands
		.spawn(NodeBundle {
			style: Style {
				size: Size::new(Val::Percent(100.), Val::Percent(100.)),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				flex_direction: FlexDirection::Column,
				..default()
			},
			background_color: Color::WHITE.into(),
			..default()
		})
		.with_children(|parent| {
			parent
				.spawn(NodeBundle {
					style: Style {
						size: Size::new(Val::Percent(100.), Val::Px(50.)),
						justify_content: JustifyContent::Center,
						align_items: AlignItems::Center,
						flex_shrink: 8.,
						..default()
					},
					background_color: Color::RED.into(),
					..default()
				})
				.with_children(|p| header = Some(build_header_ui(p, asset_server)));
			// build_header_ui(parent);
			parent
				.spawn(NodeBundle {
					style: Style {
						justify_content: JustifyContent::Center,
						align_items: AlignItems::Center,
						flex_direction: FlexDirection::Column,
						..default()
					},
					..default()
				})
				.with_children(|parent| {
					for y in (1..=8).rev() {
						parent
							.spawn(NodeBundle {
								style: Style {
									// size: Size::new(Val::Px(200.), Val::Px(200.)),
									justify_content: JustifyContent::Center,
									// arrange items horizontally, instead of vertically
									align_items: AlignItems::Center,
									// flex_direction: FlexDirection::RowReverse,
									..default()
								},
								..default()
							})
							.with_children(|parent| {
								for x in 1..=8 {
									parent.spawn((
										ButtonBundle {
											style: Style {
												size: Size::new(Val::Px(50.), Val::Px(50.)),
												justify_content: JustifyContent::Center,
												align_items: AlignItems::Center,
												margin: UiRect::all(Val::Px(2.)),
												..default()
											},
											background_color: CHESS_PIECE_BG_COLOUR.into(),
											..default()
										},
										ChessSquare { x, y },
									));
								}
							});
					}
				});
		})
		.id();

	if let Some(header) = header {
		commands.insert_resource(Header { header });
	}

	main_menu_entity
}

pub fn build_header_ui(parent: &mut ChildBuilder, asset_server: &Res<AssetServer>) -> Entity {
	parent.spawn((
		TextBundle::from_section(
			"Header",
			TextStyle {
				font_size: 25.0,
				color: Color::GREEN,
				font: asset_server.load("fonts/FiraMono-Medium.ttf"),
			},
		),
		Name::new("Header Text"),
	)).id()

	// parent
	// 	.spawn(ButtonBundle {
	// 		style: Style {
	// 			size: Size::new(Val::Px(150.), Val::Px(65.)),
	// 			// horizontally center child text
	// 			justify_content: JustifyContent::Center,
	// 			// vertically center child text
	// 			align_items: AlignItems::Center,
	// 			..default()
	// 		},
	// 		background_color: Color::DARK_GRAY.into(),
	// 		..default()
	// 	})
	// 	.with_children(|parent| {
	// 		parent.spawn(TextBundle::from_section(
	// 			"Play",
	// 			TextStyle {
	// 				font_size: 40.0,
	// 				color: Color::GREEN,
	// 				..default()
	// 			},
	// 		));
	// 	});
	// parent
	// 	.spawn(NodeBundle {
	// 		style: Style {
	// 			size: Size::width(Val::Percent(100.0)),
	// 			..default()
	// 		},
	// 		background_color: Color::rgb(0.15, 0.15, 0.15).into(),
	// 		..default()
	// 	})
	// 	.with_children(|parent| {
	// 		// text
	// 		parent.spawn((
	// 			TextBundle::from_section(
	// 				"Text Example",
	// 				TextStyle {
	// 					font: asset_server.load("fonts/FiraMono-Medium.ttf"),
	// 					font_size: 30.0,
	// 					color: Color::WHITE,
	// 				},
	// 			)
	// 			.with_style(Style {
	// 				margin: UiRect::all(Val::Px(5.0)),
	// 				..default()
	// 			}),
	// 			// Because this is a distinct label widget and
	// 			// not button/list item text, this is necessary
	// 			// for accessibility to treat the text accordingly.
	// 			Label,
	// 		));
	// 	});
}
