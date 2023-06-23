#![allow(clippy::type_complexity)]

mod selection;
mod visualization;
use crate::ui::selection::*;
use crate::ui::visualization::*;

use bevy::prelude::*;

pub struct ChessBoardPlugin;
impl Plugin for ChessBoardPlugin {
	fn build(&self, app: &mut App) {
		app
			// types
    .register_type::<ChessEngineState>()
		// states
    .add_state::<ChessEngineState>()
		// systems
    .add_startup_system(setup)
		// ui state plugin
		.add_plugin(UiStatePlugin)
		// visualization state plugin
		.add_plugin(VisualizationStatePlugin)
		// -
		;
	}
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera3dBundle {
		transform: Transform::from_xyz(0., 10., 80.),
		..default()
	});

	commands.spawn(PointLightBundle {
		point_light: PointLight {
			intensity: 100000.0,
			range: 250.,
			// shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(0., 0., 10.),
		..default()
	});

	 
}

#[derive(States, Default, Reflect, Debug, Clone, Eq, PartialEq, Hash)]
enum ChessEngineState {
	#[default]
	PickStartingPosition,

	ViewValidPaths,
}

// fn div(style: Style, color: Color) -> NodeBundle {
// 	NodeBundle {
// 		style,
// 		background_color: color.into(),
// 		..default()
// 	}
// }

// /// Adds rows (and chess squares into those rows) directly onto the parent
// fn build_grid(
// 	parent: &mut ChildBuilder,
// 	properties: &ChessBoardProperties,
// 	mut builder: impl FnMut(&mut ChildBuilder, ChessSquare),
// ) {
// 	parent
// 		.spawn((
// 			div(COLUMNS.clone(), Color::BLACK),
// 			Name::from("Grid Parent (contains rows)"),
// 		))
// 		.with_children(|parent| {
// 			for row in (0..properties.height).rev() {
// 				parent
// 					.spawn((
// 						div(ROWS.clone(), Color::WHITE),
// 						Name::new(format!("Row {}", row)),
// 					))
// 					.with_children(|parent| {
// 						for column in 0..properties.width {
// 							builder(parent, ChessSquare { x: column, y: row });
// 						}
// 					});
// 			}
// 		});
// }
