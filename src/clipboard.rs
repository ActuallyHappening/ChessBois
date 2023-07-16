#[cfg(not(target_arch = "wasm32"))]
pub fn get_from_clipboard() -> String {
	let mut clipboard = arboard::Clipboard::new().expect("Couldn't create clipboard instance");
	clipboard
		.get_text()
		.expect("Couldn't get text from clipboard")
}

// Can't await on JS Future :(

// #[cfg(target_arch = "wasm32")]
// pub fn get_from_clipboard() -> String {
// 	let clipboard = web_sys::window().expect("No window?").navigator().clipboard().expect("No clipboard?");
// 	let res = futures::executor::block_on(wasm_bindgen_futures::JsFuture::from(clipboard.read_text())).expect("Future didn't execute?");
// 	res.as_string().unwrap()
// }

// #[cfg(target_arch = "wasm32")]
// #[tokio::main(flavor = "current_thread")]
// pub async fn set_to_clipboard(str: &str) {
// 	let clipboard = web_sys::window().expect("No window?").navigator().clipboard().expect("No clipboard?");
// 	let res = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(str)).await.expect("Future didn't execute?");
// }
