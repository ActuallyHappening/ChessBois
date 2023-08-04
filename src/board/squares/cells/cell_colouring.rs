use std::{
	collections::{HashMap, HashSet},
	sync::Mutex, fs::File, io::Write,
};

use bevy::{prelude::*, reflect::FromReflect};
use bevy_egui::egui::{epaint::Hsva, Rgba, Ui};
use once_cell::sync::Lazy;
use petgraph::{prelude::UnGraph, Graph, visit::Bfs, dot::{self, Dot}};
use strum::EnumIs;
use crate::solver::Move;

use crate::{
	board::SharedState,
	solver::{
		pieces::{ChessPiece, StandardPieces},
		BoardOptions,
	},
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

static COLS: Lazy<Vec<Color>> =
	Lazy::new(|| vec![Color::RED, Color::GREEN, Color::BLUE, Color::YELLOW].into_iter().map(|c| c * 0.5).collect());

/// Uses BFS to colour all cells connected by any number of knights moves the same colour
fn compute_colourings(input: &ComputeInput) -> Val {
	let piece = &input.piece;
	let start = input.start;
	let mut graph = UnGraph::<ChessPoint, Move>::new_undirected();

	let start_index = graph.add_node(start);

	let mut available_points = HashMap::new();
	for point in input.board_options.get_available_points() {
		available_points.insert(point, graph.add_node(point));
	}

	for available_point in input.board_options.get_available_points() {
		let available_point_index= available_points[&available_point];
		for point in input.board_options.get_valid_adjacent_points(available_point, piece) {
			let point_index = available_points[&point];
			graph.add_edge(available_point_index, point_index, Move::new(available_point, point));
		}
	}

	let mut bfs = Bfs::new(&graph, start_index);
	while let Some(next_point) = bfs.next(&graph) {
		eprintln!("Point: {:?}; from graph: {:?}", next_point, graph[next_point]);
	}

	// write vis file to debug
	let data = Dot::new(&graph);
	let mut file = File::create("graph.dot").unwrap();
	let data = format!("{}", data);
	file.write_all(data.as_bytes()).unwrap();

	todo!()
}

#[test]
fn test_compute() {
	let input = ComputeInput {
		board_options: Default::default(),
		piece: StandardPieces::StandardKnight.into(),
		start: ChessPoint::new(0, 0),
	};
	let val = compute_colourings(&input);
	assert_eq!(val.len(), 64);
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
