/// Recursively solves a knights tour
fn try_move_recursive(
  tour_type: TourType,
  num_moves_required: u16,
  piece: &impl ChessPiece,
  attempting_board: Board,
  current_pos: ChessPoint,
  state_counter: &mut u128,
) -> PartialComputation {
  *state_counter += 1;
  if *state_counter >= *ALG_STATES_CAP.lock().unwrap() {
    // base case to avoid excessive computation
    return PartialComputation::GivenUp;
  }

  if num_moves_required == 0 {
    // base case
    if let Some(state) = attempting_board.get(&current_pos) {
      match state {
        // If you can finish on this square.
        // If a target is present, this may be false.
        // this check rejects solutions that don't end on a target.
        CellState::NeverOccupied {
          can_finish_on: true,
        } => {
          return PartialComputation::Successful {
            solution: vec![].into(),
          };
        }
        CellState::NeverOccupied {
          can_finish_on: false,
        } => {
          return PartialComputation::Failed;
        }
        CellState::PreviouslyOccupied => panic!("What, trying to end on point already moved to?"),
      }
    }
  }

  let mut available_moves = attempting_board.get_available_moves_from(&current_pos, piece);
  if available_moves.is_empty() {
    // stuck, no where to move
    return PartialComputation::Failed;
  }

  // sort by degree
  // this implicitely applies Warnsdorf algorithm
  available_moves.sort_by_cached_key(|p| attempting_board.get_degree(p, piece));

  match tour_type {
    TourType::Weak => {
      // IMPORTANT: Only considers moves with the lowest degree. To make brute force, remove this
      let lowest_degree = attempting_board.get_degree(&available_moves[0], piece);
      available_moves.retain(|p| attempting_board.get_degree(p, piece) == lowest_degree);
    }
    TourType::BruteForce => {}
  }

  let mut moves = None;

  for potential_next_move in available_moves {
    let mut board_with_potential_move = attempting_board.clone();

    // imagine making the move
    board_with_potential_move.set(current_pos, CellState::PreviouslyOccupied);

    // now imagine the future of making the move (recursion)
    let result = try_move_recursive(
      tour_type.clone(),
      num_moves_required - 1,
      piece,
      board_with_potential_move,
      potential_next_move,
      state_counter,
    );

    match result {
      PartialComputation::Failed => { /* Continue looping, try to find a non-failed solution */ }
      PartialComputation::Successful {
        solution: mut working_moves,
      } => {
        // initially, working_moves will be empty
        // first iteration must add move from current_pos to potential_next_move
        // this repeats
        working_moves.push(Move::new(current_pos, potential_next_move));

        // found a solution, set to moves, stop looping and return success!
        moves = Some(working_moves);
        break;
      }
      PartialComputation::GivenUp => {
        // If a child recursive call has reached the call stack limit, give up as well
        return PartialComputation::GivenUp;
      }
    };
  }

  PartialComputation::from(moves)
}