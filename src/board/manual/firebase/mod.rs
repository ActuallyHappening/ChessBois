use derive_more::Deref;
use firebase_rs::Firebase;
use once_cell::sync::Lazy;
use tracing::info;

use crate::board::SharedState;

use super::UnstableSavedState;

static BASE_URL: Lazy<String> = Lazy::new(|| {
	format!(
		"https://chess-analysis-program-default-rtdb.asia-southeast1.firebasedatabase.app/{}/",
		*crate::meta::VERSION
	)
});

#[derive(Debug, Hash, Clone, PartialEq, Eq, Deref)]
pub struct ID(String);

impl ID {
	pub fn new() -> Self {
		Self(uuid::Uuid::new_v4().to_string())
	}
}

#[tokio::main(flavor = "current_thread")]
pub async fn save_to_db(state: UnstableSavedState) {
	// let id = ID::new();
	// let data = state.into_json();
	// let db = Firebase::new(&BASE_URL).unwrap().at(&id);
	// info!("saving to db at {:?}", id);
	// db.set(&data).await.unwrap();
	// info!("finished saving to db");

	let db = Firebase::new(&"https://chess-analysis-program-default-rtdb.asia-southeast1.firebasedatabase.app/cool.json").unwrap();
	let str: i32 = db.get().await.unwrap();
	info!("got {:?}", str);

	let db = Firebase::new(&"https://chess-analysis-program-default-rtdb.asia-southeast1.firebasedatabase.app/new.json").unwrap();
	db.set(&state).await.unwrap();


}
