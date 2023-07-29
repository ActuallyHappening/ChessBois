//! Produces a summary of a computation

use bevy_egui::egui::{Ui, Color32};

use crate::solver::algs::{Computation, try_get_cached_solution};

use super::SharedState;

impl SharedState {
	pub fn summarize(&self, ui: &mut Ui) -> Option<String> {
		let comp = try_get_cached_solution(&self.clone().get_compute_state()?)?;
		Some(match comp {
			Computation::Failed { total_states } => {
				let msg = format!("Failed to find a solution after {} states", total_states);
				ui.colored_label(Color32::RED, msg.clone());
				msg
			}
			Computation::Successful { solution, explored_states } => {
				let msg = format!(
					"Found a solution after {} states, with {} moves",
					explored_states,
					solution.len()
				);
				ui.colored_label(Color32::GREEN, msg.clone());
				msg
			}
			Computation::GivenUp { explored_states } => {
				let msg = format!("Given up after {} states", explored_states);
				ui.colored_label(Color32::YELLOW, msg.clone());
				msg
			}
		})
	}
}
