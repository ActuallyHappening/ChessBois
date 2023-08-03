use std::{
	collections::{HashMap, HashSet},
	sync::Mutex,
};

use bevy::{prelude::*, reflect::FromReflect};
use bevy_egui::egui::{epaint::Hsva, Rgba, Ui};
use once_cell::sync::Lazy;
use strum::EnumIs;

use crate::{
	board::SharedState,
	solver::{pieces::{ChessPiece, StandardPieces}, BoardOptions},
	ChessPoint,
};

use super::cells_state::BorrowedCellsState;

#[derive(Clone, Default, Reflect, FromReflect, PartialEq, EnumIs)]
pub enum CellColouring {
	#[default]
	/// Black and white
	StandardChessBoard,

	AllOneColour(Color),
	/// Depends on [ComputeInput], so board_options and start
	ComputeColour,
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
			CellColouring::ComputeColour => compute(state)
				.map(|map| map.get(point).cloned().unwrap_or(INVALID))
				.unwrap_or(INVALID),
		}
	}

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
		ui.selectable_value(self, CellColouring::ComputeColour, "Compute colours");

		if let CellColouring::AllOneColour(colour) = self {
			let col = colour.as_rgba_f32();
			let col: Rgba = Rgba::from_rgba_unmultiplied(col[0], col[1], col[2], col[3]);
			let mut col: Hsva = col.into();
			ui.color_edit_button_hsva(&mut col);
			let rgb = col.to_rgb();
			*colour = Color::rgba(rgb[0], rgb[1], rgb[2], 1.0);
		}
	}
}

#[derive(Hash, Clone, PartialEq, Eq)]
struct ComputeInput {
	board_options: BoardOptions,
	piece: ChessPiece,
	start: ChessPoint,
}

type Key = ComputeInput;
type Val = HashMap<ChessPoint, Color>;

static CACHE: Lazy<Mutex<HashMap<ComputeInput, HashMap<ChessPoint, Color>>>> =
	Lazy::new(|| Mutex::new(HashMap::new()));

fn get(key: &ComputeInput) -> Option<Val> {
	let cache = CACHE.lock().unwrap();
	cache.get(key).cloned()
}
fn set(key: ComputeInput, val: Val) {
	let mut cache = CACHE.lock().unwrap();
	cache.insert(key, val);
}

const COLS: [Color; 4] = [Color::RED, Color::GREEN, Color::BLUE, Color::YELLOW];

/// Finds all pieces that can reach the start point in one move
fn search_one(start: &ChessPoint, board_options: &BoardOptions, piece: &ChessPiece) -> HashSet<ChessPoint> {
	board_options.get_valid_adjacent_points(*start, piece).into_iter().collect()
}

/// Recursively finds all pieces that can reach the start point in any number of moves
fn search(start: &ChessPoint, board_options: &BoardOptions, piece: &ChessPiece, available_set: HashSet<ChessPoint>) -> HashSet<ChessPoint> {
	let mut new_set = HashSet::new();
	for point in &available_set {
		let new_points = search_one(&point, board_options, piece);
		for new_point in new_points {
			new_set.insert(new_point);
		}
	}
	let new_set = new_set.difference(&available_set).cloned().collect::<HashSet<_>>();
	if new_set.is_empty() {
		available_set
	} else {
		search(start, board_options, piece, available_set.union(&new_set).cloned().collect())
	}
}

#[test]
fn test_search() {
	let board_options = BoardOptions::new(8, 8);
	let start = ChessPoint::new(0, 0);
	let piece: ChessPiece = StandardPieces::StandardKnight.into();
	let available_set = search_one(&start, &board_options, &piece);
	let reachable = search(&start, &board_options, &piece, available_set);

	assert_eq!(reachable.len(), 64);
	eprintln!("reachable: {:?}", reachable);
}

/// Uses BFS to colour all cells connected by any number of knights moves the same colour
fn compute_colourings(input: &ComputeInput) -> Val {
	let all_points = input.board_options.get_available_points();
	let mut all_points: HashSet<ChessPoint> = all_points.iter().cloned().collect();
	assert!(all_points.contains(&input.start), "start point is valid");

	let mut groups: Vec<HashSet<ChessPoint>> = Vec::new();

	loop {
		if all_points.is_empty() { break; }
		let start = *all_points.iter().next().unwrap();
		let reachable = search(&start, &input.board_options, &input.piece, search_one(&start, &input.board_options, &input.piece));
		all_points = all_points.difference(&reachable).cloned().collect();
		groups.push(reachable);
	}

	// todo: generalise

	// convert from groups into colours
	let mut final_map = HashMap::new();
	for (i, group) in groups.into_iter().enumerate() {
		let colour = COLS[i % COLS.len()];
		for point in group {
			final_map.insert(point, colour);
		}
	}
	final_map
}


fn compute(input: &BorrowedCellsState<'_>) -> Option<Val> {
	if let Ok(key) = ComputeInput::try_from(input) {
		let val = get(&key).unwrap_or_else(|| {
			let val = compute_colourings(&key);
			set(key, val.clone());
			val
		});
		Some(val)
	} else {
		None
	}
}

impl TryFrom<&BorrowedCellsState<'_>> for ComputeInput {
	type Error = &'static str;

	fn try_from(state: &BorrowedCellsState) -> Result<Self, Self::Error> {
		let start = *state.start.as_ref().ok_or("No start point selected")?;
		let board_options = state.board_options.clone();
		Ok(Self {
			board_options,
			start,
			piece: (*state.piece).into(),
		})
	}
}
