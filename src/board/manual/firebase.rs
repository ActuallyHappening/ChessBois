use bevy::{utils::HashMap, reflect::{Reflect, FromReflect}};
use derive_more::Deref;
#[cfg(not(target_arch = "wasm32"))]
use firebase_rs::Firebase;
use once_cell::sync::Lazy;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use super::{UnstableSavedState, MetaData};

const BASE_URL: &str =
	"https://chess-analysis-program-default-rtdb.asia-southeast1.firebasedatabase.app/";
static VERSION_APPEND: Lazy<String> =
	Lazy::new(|| format!("v0_{}_x/", crate::meta::VERSION_MINOR.to_string()));

#[cfg(not(target_arch = "wasm32"))]
static DB: Lazy<Firebase> = Lazy::new(|| {
	Firebase::new(BASE_URL)
		.expect("Cannot create DB path")
		.at(&VERSION_APPEND)
});

#[derive(Debug, Hash, Clone, PartialEq, Eq, Deref, Serialize, Deserialize, Reflect, FromReflect)]
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

	pub fn inner(&self) -> &str {
		&self.0
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct Payload(serde_json::Value);

impl Payload {
	fn new(id: ID, data: UnstableSavedState) -> Self {
		Self(json!({id.0: data.into_json()}))
	}
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "current_thread")]
pub async fn save_to_db(state: UnstableSavedState) -> Option<ID> {
	let id = ID::new();

	let metadata = state.metadata.clone();
	let payload = Payload::new(id.clone(), state);

	info!("saving to db at {:?}", id.clone());

	DB.update(&payload).await.ok()?;
	DB.at("metadata").update(&json!({ id.0.clone(): metadata })).await.ok()?;

	info!("finished saving to db");
	Some(id)
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "current_thread")]
pub async fn get_from_db(id: ID) -> Option<UnstableSavedState> {
	info!("getting from db at {:?}", id.clone());

	let db = DB.at(&id.0);
	let data: String = db.get().await.ok()?;
	info!("got data from db: {}", data);
	let data = match UnstableSavedState::from_json(&data) {
		Ok(data) => data,
		Err(e) => {
			info!("error parsing data: {}", e);
			return None;
		}
	};

	info!("finished getting from db");

	Some(data)
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "current_thread")]
pub async fn get_metadata_list() -> Option<Vec<MetaData>> {
	info!("getting all metadata");

	let db = DB.at("metadata");
	let data: HashMap<String, MetaData> = db.get().await.ok()?;

	let mut metadatas = Vec::new();
	for (id, mut metadata) in data {
		metadata.id = Some(ID(id));
		metadatas.push(metadata);
	}

	info!("finished getting all metadata");

	Some(metadatas)
}