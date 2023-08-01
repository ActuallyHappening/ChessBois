use bevy::prelude::{*, Reflect};

const URL: &str = "https://caleb-msrc-q11.netlify.app/";

#[derive(Resource)]
pub struct InitialLoadedID(String);

#[cfg(feature = "web-start")]
pub fn get_url_id() -> Option<String> {
	let window = web_sys::window().expect("To be able to get window");
	let location = window.location();
	let full_href = location.href().expect("Website has no href?");

	let query_params = full_href.split('?').nth(1)?;
	let id_param = query_params
		.split('&').next()?
		.split('=')
		.collect::<Vec<_>>();
	
	if id_param.first()? != &"id" {
		return None;
	}
	let id = id_param.get(2)?;

	Some(id.to_string())
}

pub fn create_url_with_id(id: String) -> String {
	format!("{}?id={}", URL, id)
}