#[deprecated]
mod cache {
	use crate::{pieces::ChessPiece, BoardOptions, ChessPoint, Move, Moves};
	use lru::LruCache;
	use once_cell::sync::Lazy;
	use std::num::NonZeroUsize;
	use std::ops::{Deref, DerefMut};
	use std::{any::TypeId, collections::HashMap, sync::Mutex};
	use tracing::info;

	use super::warnsdorf_tour_repeatless;

	fn warnsdorf_tour_repeatless_cached<P: ChessPiece + 'static>(
		piece: &P,
		options: BoardOptions,
		start: ChessPoint,
	) -> Option<Moves> {
		if let Some(cached_cycle) = try_get_cached_cycle::<P>(&options) {
			info!("Using cached cycle");
			let ret = cached_cycle.generate_moves_starting_at(start);
			// info!("Cached first move is {}", ret);
			Some(ret)
		} else {
			let moves = warnsdorf_tour_repeatless(piece, options.clone(), start);
			if let Some(moves) = moves {
				add_cycle_to_cache::<P>(options, moves.clone());
				Some(moves)
			} else {
				None
			}
		}
	}

	static CYCLE_CACHE: Lazy<Mutex<HashMap<TypeId, LruCache<BoardOptions, Cycle>>>> =
		Lazy::new(|| Mutex::new(HashMap::new()));

	#[derive(Debug, Clone, PartialEq, Eq, Hash)]
	pub struct Cycle {
		// options:
		original_moves: Moves,
	}

	impl Cycle {
		pub fn new(mut original_moves: Moves) -> Self {
			// add move from end of last move to start of first move
			let last_move = original_moves.last().unwrap();
			let first_move = original_moves.first().unwrap();
			let last_to_first = Move::new(last_move.to, first_move.from);
			original_moves.push(last_to_first);

			Self { original_moves }
		}

		pub fn generate_moves_starting_at(self, start: ChessPoint) -> Moves {
			let mut moves = self.original_moves;

			// find index of start
			let start_index = moves
				.iter()
				.position(|m| m.from == start)
				.unwrap_or_else(|| {
					let moves_starting_positions = moves.iter().map(|m| m.from).collect::<Vec<_>>();
					let does_contain = moves_starting_positions.iter().any(|p| p == &start);
					let moves_str: String = moves_starting_positions.iter().map(|p| format!("{}, ", p)).collect();
					panic!(
						"Starting move to be in cycle if board options are constant\n Moves:\n {:?} (len: {moves_len}), \nstart: \n{}, \n Contains: {does_contain}",
						moves_str, start, moves_len = moves.moves.len(), does_contain = does_contain
					)
				});

			// reorder moves so that start is first, and wrap around
			let moves: &mut Vec<Move> = moves.deref_mut();

			moves.rotate_left(start_index);
			// moves.push(moves[0]);
			// moves.rotate_left(1);

			let mut moves = moves.deref().clone();

			// remove last item of moves vector
			moves.pop();

			moves.into()
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_gen_moves_from_cycle() {
			let initial = Moves::new(vec![
				Move::new(ChessPoint::new(1, 1), ChessPoint::new(2, 2)),
				Move::new(ChessPoint::new(2, 2), ChessPoint::new(3, 3)),
				Move::new(ChessPoint::new(3, 3), ChessPoint::new(4, 4)),
				Move::new(ChessPoint::new(4, 4), ChessPoint::new(1, 1)),
			]);
			let new_start_pos = ChessPoint::new(3, 3);
			let expected = Moves::new(vec![
				Move::new(ChessPoint::new(3, 3), ChessPoint::new(4, 4)),
				Move::new(ChessPoint::new(4, 4), ChessPoint::new(1, 1)),
				Move::new(ChessPoint::new(1, 1), ChessPoint::new(2, 2)),
				Move::new(ChessPoint::new(2, 2), ChessPoint::new(3, 3)),
			]);

			let cycle = Cycle::new(initial);
			let actual = cycle.generate_moves_starting_at(new_start_pos);

			println!("Expected: \n{expected}, got: \n{actual}");

			assert_eq!(actual, expected)
		}
	}

	pub fn try_get_cached_cycle<P: ChessPiece + 'static>(options: &BoardOptions) -> Option<Cycle> {
		let mut caches = CYCLE_CACHE.lock().unwrap();
		let id = TypeId::of::<P>();

		let cache = match caches.get_mut(&id) {
			Some(cache) => cache,
			None => {
				let cache = LruCache::new(NonZeroUsize::new(100).unwrap());
				caches.insert(id, cache);
				caches.get_mut(&id).unwrap()
			}
		};

		cache.get(options).cloned()
	}

	pub fn add_cycle_to_cache<P: ChessPiece + 'static>(options: BoardOptions, moves: Moves) {
		let mut caches = CYCLE_CACHE.lock().unwrap();
		let id = TypeId::of::<P>();

		let cache = match caches.get_mut(&id) {
			Some(cache) => cache,
			None => {
				let cache = LruCache::new(NonZeroUsize::new(100).unwrap());
				caches.insert(id, cache);
				caches.get_mut(&id).unwrap()
			}
		};

		info!("Putting a solution in the cache");
		cache.put(options, Cycle::new(moves));
	}
}
