use derive_more::Deref;
use firebase_rs::Firebase;
use once_cell::sync::Lazy;

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

pub async fn save_to_db(state: UnstableSavedState) {
	let id = ID::new();
	let data = state.into_json();
	let db = Firebase::new(&BASE_URL).unwrap().at(&id);
	db.set(&data).await.unwrap();
}
