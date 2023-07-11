
	use std::sync::Mutex;

	use super::*;
	use crate::solver::algs::Computation;

	/// When sent as an event, indicates that this computation has just finished NOT that it is current!
	/// Check current Options against state to see if it is current.
	///
	/// When as a resource, indicates that is is current computation
	#[derive(Resource, Debug, Clone, PartialEq, Eq)]
	pub struct ComputationResult(Computation, Options);

	impl ComputationResult {
		pub fn get(self) -> (Computation, Options) {
			(self.0, self.1)
		}
	}

	// impl From<ComputationResult> for Computation {
	// 	fn from(result: ComputationResult) -> Self {
	// 		result.0
	// 	}
	// }

	impl ComputationResult {
		pub fn into_comp(&self) -> Computation {
			self.0.clone()
		}
	}

	pub fn begin_background_compute<P: ChessPiece + Copy + Send + Sync + 'static>(
		alg: Algorithm,
		piece: &P,
		options: Options,
		_commands: &mut Commands,
	) {
		let state = options.clone();
		if let Some(start) = options.selected_start {
			if options.options.get_available_points().contains(&start) {
				let piece: P = *piece;
				start_executing_task(state, move || {
					trace!("About to compute");
					alg
						.tour_computation_cached(&piece, options.clone())
						.unwrap()
				})
			}
		}
	}

	static TASK_RESULT: Mutex<Option<ComputationResult>> = Mutex::new(None);

	fn start_executing_task(state: Options, task: impl FnOnce() -> Computation + Send + 'static) {
		#[cfg(not(target_arch = "wasm32"))]
		{
			use std::thread;

			// create a new thread to run the task on
			thread::spawn(move || {
				let res = task();

				*TASK_RESULT.lock().unwrap() = Some(ComputationResult(res, state));
			});
		}

		#[cfg(target_arch = "wasm32")]
		{
			let res = task();
			*TASK_RESULT.lock().unwrap() = Some(ComputationResult(res, state));
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
	fn poll_computation_result() -> Option<ComputationResult> {
		(*TASK_RESULT.lock().unwrap()).clone()
	}

	/// Returns successful computation ONCE (else None) immediately (doesn't block)
	fn get_computation() -> Option<ComputationResult> {
		match poll_computation_result() {
			Some(comp) => {
				*TASK_RESULT.lock().unwrap() = None;
				Some(comp)
			}
			None => None,
		}
	}

	/// Polls for and handles raw [ComputationResult]
	pub fn handle_automatic_computation(
		mut commands: Commands,
		options: Res<CurrentOptions>,

		mut update_computation: EventWriter<ComputationResult>,
	) {
		// does the work of computing
		if let Some(comp) = get_computation() {
			let state = options.as_options();
			if &comp.1 == state {
				// only set as current if state is valid
				commands.insert_resource(comp.clone());
			}

			// let message get out to everybody, even if state is invalid
			update_computation.send(comp);
		}
	}
