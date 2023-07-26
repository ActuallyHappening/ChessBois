use std::{collections::HashMap, sync::Mutex};

use once_cell::sync::Lazy;

use super::{cells::CellMark, *};
use crate::solver::algs::{self, Computation, ComputeInput};

/// Syncs [SharedState] resource with computations
pub fn compute_from_state(state: ResMut<SharedState>) {
	let compute_state = state.clone().get_compute_state();
}

pub fn get_cached_mark(input: &ComputeInput) -> Option<CellMark> {
	algs::try_get_cached_solution(input).map(|c| c.into())
}

impl SharedState {
	/// Clears the displayed computation if part of State changed that affects computations
	pub fn invalidate(&mut self) -> &mut Self {
		self.moves = None;
		self
	}

	pub fn set_alg(&mut self, alg: Algorithm) -> &mut Self {
		self.alg = alg;
		self.invalidate()
	}

	pub fn set_start(&mut self, start: ChessPoint) -> &mut Self {
		self.start = Some(start);
		self.invalidate()
	}

	pub fn set_board_options(&mut self, new_board_options: BoardOptions) -> &mut Self {
		self.board_options = new_board_options;
		self.invalidate()
	}
}

static COMPUTATIONS_TO_HANDLE: Lazy<Mutex<HashMap<ComputeInput, Computation>>> =
	Lazy::new(|| Mutex::new(HashMap::new()));
	
fn start_executing_task(state: ComputeInput, task: impl FnOnce() -> Computation + Send + 'static) {
	#[cfg(not(target_arch = "wasm32"))]
	{
		use std::thread;

		// create a new thread to run the task on
		thread::spawn(move || {
			let res = task();

			COMPUTATIONS_TO_HANDLE.lock().unwrap().insert(state, res);
		});
	}

	#[cfg(target_arch = "wasm32")]
	{
		let res = task();
		COMPUTATIONS_TO_HANDLE.lock().unwrap().insert(state, res);
	}
	// TODO: Mess around with WebWorkers & don't break audio?
	// futures::executor::block_on(async move {
	// 	{
	// 		use wasm_futures_executor::ThreadPool;

	// 		let pool = ThreadPool::max_threads().await.unwrap();

	// 		pool.spawn_ok(async move {
	// 			let res = task();

	// 			*TASK_RESULT.lock().unwrap() = Some(ComputationResult(res, state));
	// 		});
	// 	}
	// })
}
