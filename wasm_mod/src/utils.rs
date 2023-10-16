// use std::time::Duration;
use super::{models, Result, RetryAfter};
use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, WorkerGlobalScope};

#[allow(dead_code)]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// pub async fn sleep(duration: Duration) {
//     JsFuture::from(Promise::new(&mut |yes, _| {
//         window()
//             .unwrap()
//             .set_timeout_with_callback_and_timeout_and_arguments_0(
//                 &yes,
//                 duration.as_millis() as i32,
//             )
//             .unwrap();
//     }))
//     .await
//     .unwrap();
// }

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

/// Prepares and executes an HTTP request to spotify,
/// including token and other headers.
pub(crate) async fn execute_http_request(
    auth_header_value: &str,
    token_header_value: &str,
    url: &str,
) -> Result<JsValue> {
    log!("prepare_http_request entered");
    // set request params
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    log!("{url}");

    // create the request
    let request = match Request::new_with_str_and_init(&url, &opts) {
        Ok(v) => v,
        Err(e) => {
            log!("Spotify request creation failed");
            log!("{:?}", e);
            // TODO: may be worth a retry
            return Err(RetryAfter::Never);
        }
    };

    log!("Request created");

    // add headers
    let _ = request.headers().set("Accept", "application/json");
    // these keys were manually extracted from chrome webdev tools
    let _ = request.headers().set("authorization", auth_header_value);
    let _ = request.headers().set("client-token", token_header_value);

    log!("Headers set");

    // WorkerGlobalScope object is needed to perform fetch
    let worker_global_scope = match js_sys::global().dyn_into::<WorkerGlobalScope>() {
        Ok(v) => v,
        Err(e) => {
            log!("WorkerGlobalScope unavailable");
            log!("{:?}", e);
            // TODO: may be worth a retry
            return Err(RetryAfter::Never);
        }
    };

    // execute the fetch promise using Rust's JsFuture
    let resp_value = match JsFuture::from(worker_global_scope.fetch_with_request(&request)).await {
        Ok(v) => v,
        Err(e) => {
            log!("Spotify request failed");
            log!("{:?}", e);
            // TODO: may be worth a retry
            return Err(RetryAfter::Never);
        }
    };

    // exit if the response is not of the expected type
    if !resp_value.is_instance_of::<Response>() {
        log!("Spotify response in not Response");
        log!("{:?}", resp_value);
        // TODO: may be worth a retry
        return Err(RetryAfter::Never);
    };

    // this is unlikely to fail because of the previous type check
    let resp: Response = match resp_value.dyn_into() {
        Ok(v) => v,
        Err(e) => {
            log!("Cannot typecast response to Response");
            log!("{:?}", e);
            // TODO: may be worth a retry
            return Err(RetryAfter::Never);
        }
    };

    // Read the response stream to completion.
    // In theory, the stream may still be open and the op may take some time to complete
    let resp = match resp.json() {
        Ok(v) => JsFuture::from(v).await,
        Err(e) => {
            log!("Cannot convert Promise to Future");
            log!("{:?}", e);
            // TODO: may be worth a retry
            return Err(RetryAfter::Never);
        }
    };

    log!("HTTP request completed");

    // Unwrap the response and handle the error
    let resp = match resp {
        Ok(v) => v,
        Err(e) => {
            log!("Spotify request failed");
            log!("{:?}", e);
            // TODO: may be worth a retry
            return Err(RetryAfter::Never);
        }
    };

    // log!("Resp as string:");
    // log!("{:?}", resp.as_string().unwrap());

    return Ok(resp);
}

pub(crate) async fn fetch_playlist(
    auth_header_value: &str,
    token_header_value: &str,
    playlist_id: &str,
    user_uri: &str,
) {
    log!("fetch_playlist entered");

    // the URL to call
    // https://api-partner.spotify.com/pathfinder/v1/query?operationName=fetchPlaylist&variables={"uri":"spotify:playlist:37i9dQZF1DWZeKCadgRdKQ","offset":0,"limit":25}&extensions={"persistedQuery":{"version":1,"sha256Hash":"5534e86cc2181b9e70be86ae26d514abd8d828be2ee56e5f8b7882dd70204c62"}}
    //         let url = "https://api-partner.spotify.com/pathfinder/v1/query?operationName=fetchPlaylist&variables=%7B%22uri%22%3A%22spotify%3Aplaylist%3A37i9dQZF1DWZeKCadgRdKQ%22%2C%22offset%22%3A0%2C%22limit%22%3A25%7D&extensions=%7B%22persistedQuery%22%3A%7B%22version%22%3A1%2C%22sha256Hash%22%3A%225534e86cc2181b9e70be86ae26d514abd8d828be2ee56e5f8b7882dd70204c62%22%7D%7D";

    let url = &["https://api-partner.spotify.com/pathfinder/v1/query?operationName=fetchPlaylist&variables=%7B%22uri%22%3A%22spotify%3Aplaylist%3A", playlist_id,"%22%2C%22offset%22%3A0%2C%22limit%22%3A25%7D&extensions=%7B%22persistedQuery%22%3A%7B%22version%22%3A1%2C%22sha256Hash%22%3A%225534e86cc2181b9e70be86ae26d514abd8d828be2ee56e5f8b7882dd70204c62%22%7D%7D"].concat();

    log!("{url}");

    let resp = match execute_http_request(auth_header_value, token_header_value, url).await {
        Ok(v) => v,
        Err(e) if matches!(e, RetryAfter::Never) => {
            return;
        }
        Err(_) => {
            unimplemented!("Retries are not implemented");
        }
    };

    // convert into a rust struct
    let playlist = match serde_wasm_bindgen::from_value::<models::PlaylistRoot>(resp) {
        Ok(v) => v,
        Err(e) => {
            log!("Cannot deser playlist");
            log!("{:?}", e);
            return;
        }
    };

    // log!("{:?}", playlist);

    // compare the playlist owner ID with the user ID to check if the playlist can be modified
    let owner_uri = &playlist.data.playlist_v2.owner_v2.data.uri;
    log!("User: {user_uri}, Owner: {owner_uri}");
    if user_uri == &playlist.data.playlist_v2.owner_v2.data.uri {
        log!("User owns the playlist");
    } else {
        log!("User does not own the playlist");
    }
}
