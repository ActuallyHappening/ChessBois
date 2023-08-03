use std::collections::HashMap;

use bevy::{prelude::*, reflect::FromReflect};
use bevy_egui::egui::{Ui, epaint::Hsva, Rgba};
use strum::EnumIs;

use crate::ChessPoint;

use super::cells_state::BorrowedCellsState;

#[derive(Clone, Default, Reflect, FromReflect, PartialEq, EnumIs)]
pub enum CellColouring {
	#[default]
	/// Black and white
	StandardChessBoard,

	AllOneColour(Color),
	ComputeColour {
		map: HashMap<ChessPoint, Color>,
	},
}

const SELECTED_COLOUR: Color = Color::PURPLE;
const DISABLED_COLOUR: Color = Color::RED;
const END_COLOUR_FACTOR: Color = Color::BLUE;

const INVALID: Color = Color::BLACK;
const DEFAULT_ALL_COLOUR: Color = Color::WHITE;

impl CellColouring {
	/// Takes as much information as it can get and returns the colour the cell should be.
	pub fn compute_colour(&self, point: &ChessPoint, state: &BorrowedCellsState) -> Color {
		let start = state.start.as_ref();
		match self {
			CellColouring::StandardChessBoard => {
				if state.get_unavailable_points().contains(point) {
					DISABLED_COLOUR
				} else if Some(point) == start {
					SELECTED_COLOUR
				} else if state.visual_opts.show_end_colour
					&& state.moves.as_ref().is_some_and(|moves| {
						moves
							.moves()
							.into_iter()
							.last()
							.is_some_and(|last| last.to == *point)
					}) {
					END_COLOUR_FACTOR
				} else {
					point.get_standard_colour()
				}
			}
			CellColouring::AllOneColour(colour) => *colour,
			CellColouring::ComputeColour { map: colours } => {
				if let Some(colour) = colours.get(point) {
					*colour
				} else {
					INVALID
				}
			}
		}
	}
}

impl CellColouring {
	pub fn ui(&mut self, ui: &mut Ui) {
		ui.selectable_value(
			self,
			CellColouring::StandardChessBoard,
			"Standard chess colouring (black & white)",
		);
		if ui
			.selectable_label(self.is_all_one_colour(), "Board all one colour")
			.clicked()
			&& !self.is_all_one_colour()
		{
			*self = CellColouring::AllOneColour(DEFAULT_ALL_COLOUR);
		}

		if let CellColouring::AllOneColour(colour) = self {
			let col = colour.as_rgba_f32();
			let col: Rgba = Rgba::from_rgba_unmultiplied(col[0], col[1], col[2], col[3]);
			let mut col: Hsva = col.into();
			ui.color_edit_button_hsva(&mut col);
			let rgb = col.to_rgb();
			*colour = Color::rgba(rgb[0], rgb[1], rgb[2], 1.0);
		}

		if self.is_compute_colour() {
			ui.selectable_label(true, "Compute colour (see select-algorithm)").clicked();
		}
	}
}
