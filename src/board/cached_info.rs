use crate::solver::algs::Computation;
use lru::LruCache;
use once_cell::sync::Lazy;
use std::{num::NonZeroUsize, sync::Mutex};

use super::*;

static CACHE: Lazy<Mutex<LruCache<Options, CellMark>>> =
	Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(10_000).unwrap())));

#[derive(Clone)]
pub enum CellMark {
	Failed,
	Succeeded,

	GivenUp,
}

impl From<Computation> for CellMark {
	fn from(value: Computation) -> Self {
		match value {
			Computation::Successful { .. } => CellMark::Succeeded,
			Computation::Failed { .. } => CellMark::Failed,
			Computation::GivenUp { .. } => CellMark::GivenUp,
		}
	}
}

pub fn get(options: &Options) -> Option<CellMark> {
	let mut cache = CACHE.lock().unwrap();
	trace!(
		"Getting info cache for alg: {:?} at {}",
		options.selected_algorithm,
		options.selected_start.unwrap()
	);
	cache.get(options).cloned()
}
fn set(options: Options, mark: CellMark) {
	let mut cache = CACHE.lock().unwrap();
	trace!(
		"Setting info cache for alg: {:?} at {}",
		options.selected_algorithm,
		options.selected_start.unwrap()
	);
	cache.put(options, mark);
}

pub fn update_cache_from_computation(
	mut computations: EventReader<ComputationResult>,
	mut commands: Commands,

	_markers: Query<Entity, (With<MarkerMarker>, With<ChessPoint>)>,
	mut mma: ResSpawning,
) {
	// if !computations.is_empty() {
	// 	despawn_markers(&mut commands, markers);
	// }
	for comp in computations.iter() {
		let (comp, options) = comp.clone().get();
		let mark = CellMark::from(comp);

		debug!("Updating info cache");
		set(options.clone(), mark);

		spawn_markers(&options, &mut commands, &mut mma)
	}
}
