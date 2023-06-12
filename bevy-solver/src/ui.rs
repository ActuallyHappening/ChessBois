use bevy::prelude::*;

pub struct UiPlugin;
impl Plugin for UiPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_startup_system(spawn_ui)
			// .add_system(despawn_ui)
			// for formatting
			;
	}
}

pub fn spawn_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
	let main_menu_entity = build_ui(&mut commands, &asset_server);
}

// pub fn despawn_ui(commands: &mut Commands, main_menu_entity: Entity) {
// 	commands.despawn_recursive(main_menu_entity);
// }

pub fn build_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
	let main_menu_entity = commands.spawn(NodeBundle { 
		style: Style {
			size: Size::new(Val::Percent(100.), Val::Percent(100.)),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
			..default()
		},
		background_color: Color::RED.into(),
		..default() }).id();

	main_menu_entity
}
