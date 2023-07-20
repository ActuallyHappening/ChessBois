/// Sets url params, reloads page
pub fn set_url_params(param_str: String) {
	// sets window.location.search
	// https://developer.mozilla.org/en-US/docs/Web/API/Window/location

	let window = web_sys::window().expect("To be able to get window");
	let location = window.location();
	let full_href = location.href().expect("Website has no href?");
	// remove search params from href
	let base_href = full_href
		.split('?')
		.next()
		.expect("To be able to split href");

	let concat_href = format!("{}?{}", base_href, param_str);
	location
		.set_href(&concat_href)
		.expect("To be able to set href");

	// will reload?
}

pub fn get_url_params() -> Option<String> {
	let window = web_sys::window().expect("To be able to get window");
	let location = window.location();
	location.search().ok()
}

use bevy::prelude::*;

use crate::board::ManualMoves;

pub fn try_load_state_from_url() -> Option<ManualMoves> {
	let url_params = get_url_params()?;
	let moves = serde_qs::from_str(&url_params).ok()?;
	Some(moves)
}
