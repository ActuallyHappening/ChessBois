use super::{squares::CellMark, *};
use crate::solver::algs::{self, Computation, OwnedComputeInput};

/// Syncs [SharedState] resource with computations
pub fn compute_from_state(state: ResMut<SharedState>) {
	if let Some(compute_state) = state.clone().get_compute_state() {
		// try get from algs cache
		if let Some(comp) = algs::try_get_cached_solution(&compute_state) {
			if let Computation::Successful { solution, .. } = comp {
				state.into_inner().set_moves(solution);
			}
		} else {
			// not cached
			let comp_state = compute_state.clone();
			start_executing_task(compute_state.clone(), || {
				algs::Algorithm::tour_computation_cached(comp_state)
			});
		}
	}
}

pub fn get_cached_mark(input: &OwnedComputeInput) -> Option<CellMark> {
	algs::try_get_cached_solution(input).map(|c| c.into())
}

impl SharedState {
	/// Clears the displayed computation if part of State changed that affects computations
	pub fn invalidate(&mut self) -> &mut Self {
		// warn!("Invalidating state");
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

// static COMPUTATIONS_TO_HANDLE: Lazy<Mutex<HashMap<ComputeInput, Computation>>> =
// 	Lazy::new(|| Mutex::new(HashMap::new()));

fn start_executing_task(_state: OwnedComputeInput, task: impl FnOnce() -> Option<Computation> + Send + 'static) {
	#[cfg(not(target_arch = "wasm32"))]
	{
		use std::thread;

		// create a new thread to run the task on
		thread::spawn(move || {
			let _res = task();

			// COMPUTATIONS_TO_HANDLE.lock().unwrap().insert(state, res);
		});
	}

	#[cfg(target_arch = "wasm32")]
	{
		let res = task();
		// COMPUTATIONS_TO_HANDLE.lock().unwrap().insert(state, res);
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
