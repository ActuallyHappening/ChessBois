static BASE_URL: Lazy<String> = Lazy::new(
	|| {
		format!("https://chess-analysis-program-default-rtdb.asia-southeast1.firebasedatabase.app/{}/")
	},
	crate::meta::VERSION_MINOR.parse().unwrap(),
);
