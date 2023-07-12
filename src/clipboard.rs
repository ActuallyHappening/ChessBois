#[cfg(not(target_os = "wasm32"))]
pub fn get_from_clipboard() -> String {
	let mut clipboard = arboard::Clipboard::new().expect("Couldn't create clipboard instance");
	clipboard.get_text().expect("Couldn't get text from clipboard")
}

#[cfg(target_os = "wasm32")]
pub fn get_from_clipboard() -> String {
	let clipboard = web_sys::Clipboard::new();
	clipboard.read_text()
}