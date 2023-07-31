use std::collections::HashMap;

use derive_more::Deref;
use firebase_rs::Firebase;
use once_cell::sync::Lazy;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use crate::board::manual::save::StableSavedState;

use super::UnstableSavedState;

const BASE_URL: &str =
	"https://chess-analysis-program-default-rtdb.asia-southeast1.firebasedatabase.app/";
static VERSION_APPEND: Lazy<String> =
	Lazy::new(|| format!("v0_{}_x/", crate::meta::VERSION_MINOR.to_string()));
static DB: Lazy<Firebase> = Lazy::new(|| {
	Firebase::new(BASE_URL)
		.expect("Cannot create DB path")
		.at(&VERSION_APPEND)
});

#[derive(Debug, Hash, Clone, PartialEq, Eq, Deref, Serialize, Deserialize)]
pub struct ID(String);

impl ID {
	fn new() -> Self {
		// generate a random 8 character string
		let mut rng = rand::thread_rng();
		let id: Vec<u8> = std::iter::repeat(())
			.map(|()| rng.sample(rand::distributions::Alphanumeric))
			.take(13)
			.collect();
		let id = String::from_utf8(id).unwrap();
		Self(id)
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct Payload(serde_json::Value);

impl Payload {
	fn new(id: ID, data: UnstableSavedState) -> Self {
		Self(json!({id.0: data.into_json()}))
	}
}

#[tokio::main(flavor = "current_thread")]
pub async fn save_to_db(state: UnstableSavedState) -> Option<ID> {
	let id = ID::new();
	let payload = Payload::new(id.clone(), state);

	info!("saving to db at {:?}", id.clone());
	DB.update(&payload).await.ok()?;
	DB.at("metadata").update(&json!({id.0: state.}));
	info!("finished saving to db");
	Some(id)
}

#[tokio::main(flavor = "current_thread")]
pub async fn get_from_db(id: ID) -> Option<UnstableSavedState> {
	info!("getting from db at {:?}", id.clone());

	let db = DB.at(&id.0);
	let data: StableSavedState = db.get().await.ok()?;

	info!("finished getting from db");

	Some(data.into())
}