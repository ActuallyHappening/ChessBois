use bevy::prelude::*;

pub struct UiPlugin;
impl Plugin for UiPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(spawn_ui)
			// .add_system(despawn_ui)
			.add_system(chess_piece_interactions)
			// for formatting
			.register_type::<ChessSquare>();
	}
}

#[derive(Component, Debug, Reflect)]
pub struct ChessSquare {
	pub x: u8,
	pub y: u8,
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

fn chess_piece_interactions(
	mut commands: Commands,
	mut interaction_query: Query<(&Interaction, &ChessSquare, &mut BackgroundColor), (Changed<Interaction>)>,
) {
	// info!("chess_piece_interactions");
	// info!("interaction_query: {:?}", interaction_query.iter().count());
	for (interaction, chess_square, mut bg) in interaction_query.iter_mut() {
		// info!("Interaction: {:?} on {:?}", interaction, chess_square);
		match *interaction {
			Interaction::Clicked => {
				info!("Clicked on {:?}", chess_square);
				// commands.entity(entity).despawn_recursive();
				*bg = random_colour().into();
			}
			_ => {}
		}
	}
}

pub fn build_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
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
			for y in 0..=8 {
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
						for x in 0..=8 {
							parent.spawn((
								ButtonBundle {
									style: Style {
										size: Size::new(Val::Px(50.), Val::Px(50.)),
										justify_content: JustifyContent::Center,
										align_items: AlignItems::Center,
										margin: UiRect::all(Val::Px(2.)),
										..default()
									},
									background_color: Color::BLACK.into(),
									..default()
								},
								ChessSquare { x, y },
							));
						}
					});
			}
		})
		.id();

	main_menu_entity
}
