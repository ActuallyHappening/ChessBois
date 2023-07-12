#[cfg(not(target_arch = "wasm32"))]
pub fn get_from_clipboard() -> String {
	let mut clipboard = arboard::Clipboard::new().expect("Couldn't create clipboard instance");
	clipboard.get_text().expect("Couldn't get text from clipboard")
}

#[cfg(target_arch = "wasm32")]
pub fn get_from_clipboard() -> String {
	let clipboard = web_sys::window().expect("No window?").navigator().clipboard().expect("No clipboard?");
	let res = futures::executor::block_on(wasm_bindgen_futures::JsFuture::from(clipboard.read_text())).expect("Future didn't execute?");
	res.as_string().unwrap()
}