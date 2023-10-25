#[macro_use]
mod utils;
mod models;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace=console)]
    fn log(s: &str);
}

// Will be called in content.js
#[wasm_bindgen]
pub fn hello_content() {
    alert("Hello from the content script!");
}

// Will be called in background.js
#[wasm_bindgen]
pub fn hello_background() {
    log("Hello from the background script!");
}

#[wasm_bindgen]
pub async fn rebuild_playlist(
    auth_header_value: &str,
    token_header_value: &str,
    _playlist_id: &str,
    _user_uri: &str,
) {
    // TODO: add top level error handling to allow for ? in underlying code
    utils::fetch_all_albums_and_playlists(auth_header_value, token_header_value).await;
    // utils::fetch_playlist(auth_header_value, token_header_value, playlist_id, user_uri).await;
}

/// All error handling in this crate is based on either retrying a request after some time
/// or exiting gracefully.
#[derive(Debug, Clone)]
pub enum RetryAfter {
    Seconds(i64),
    Never,
}

/// The result type that should be used in place of std::Result
/// throughout the app
pub type Result<T> = std::result::Result<T, RetryAfter>;
