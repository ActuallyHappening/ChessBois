use bevy::prelude::Resource;
use derive_more::{Constructor, Into};

const URL: &str = "https://caleb-msrc-q11.netlify.app/";

#[derive(Resource, Clone, Constructor, Into)]
pub struct InitialLoadedID(serde_json::Value);

#[cfg(feature = "web-start")]
pub fn get_url_id() -> Option<String> {
	let window = web_sys::window().expect("To be able to get window");
	let location = window.location();
	let full_href = location.href().expect("Website has no href?");

	extract_url(full_href)
}

fn extract_url(full_href: String) -> Option<String> {
	// id=jhlfjsdh&junk=hjklhlkj
	let query_params = full_href.split('?').nth(1)?;

	// id=hjklh
	let id_param = query_params
		.split('&')
		.next()?
		.split('=')
		.collect::<Vec<_>>();

	if id_param.first()? != &"id" {
		return None;
	}
	let id = id_param.get(1)?;

	Some(id.to_string())
}

pub fn create_url_with_id(id: String) -> String {
	format!("{}?id={}", URL, id)
}

#[test]
fn test_extract_url() {
	let test_url = "http://0.0.0.0:6969/?id=xZET4CwfRm2zd";
	let id = extract_url(test_url.into());
	assert_eq!(id, Some("xZET4CwfRm2zd".to_string()));
}
