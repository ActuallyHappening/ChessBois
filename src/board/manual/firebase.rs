use std::collections::HashMap;

use derive_more::Deref;
use firebase_rs::Firebase;
use once_cell::sync::Lazy;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use super::UnstableSavedState;

const BASE_URL: &str = "https://chess-analysis-program-default-rtdb.asia-southeast1.firebasedatabase.app/";
static VERSION_APPEND: Lazy<String> = Lazy::new(|| {
	format!(
		"v0_{}_x/",
		crate::meta::VERSION_MINOR.to_string()
	)
});

#[derive(Debug, Hash, Clone, PartialEq, Eq, Deref, Serialize, Deserialize)]
pub struct ID(String);

impl ID {
	fn new() -> Self {
		// generate a random 8 character string
		let mut rng = rand::thread_rng();
		let id: Vec<u8> = std::iter::repeat(())
			.map(|()| rng.sample(rand::distributions::Alphanumeric))
			.take(8)
			.collect();
		let id = String::from_utf8(id).unwrap();
		Self(id)
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct Payload(HashMap<ID, String>);

impl Payload {
	fn new(id: ID, data: String) -> Self {
		let mut map = HashMap::new();
		map.insert(id, data);
		Self(map)
	}
}

#[tokio::main(flavor = "current_thread")]
pub async fn save_to_db(state: UnstableSavedState) {
	let id = ID::new();
	let data = state.into_json();
	let db = Firebase::new(BASE_URL).unwrap().at(&VERSION_APPEND);
	let payload = json!({id.clone().0: data});

	info!("saving to db at {:?}", id);
	db.update(&payload).await.unwrap();
	info!("finished saving to db");
}
