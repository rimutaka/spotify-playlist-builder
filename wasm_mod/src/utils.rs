// use std::time::Duration;
use serde::Deserialize;
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

pub(crate) async fn fetch_playlist(
    auth_header_value: &str,
    token_header_value: &str,
    playlist_id: &str,
    user_uri: &str,
) {
    log!("fetch_playlist entered");
    // set request params
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    // the URL to call
    // https://api-partner.spotify.com/pathfinder/v1/query?operationName=fetchPlaylist&variables={"uri":"spotify:playlist:37i9dQZF1DWZeKCadgRdKQ","offset":0,"limit":25}&extensions={"persistedQuery":{"version":1,"sha256Hash":"5534e86cc2181b9e70be86ae26d514abd8d828be2ee56e5f8b7882dd70204c62"}}
    //         let url = "https://api-partner.spotify.com/pathfinder/v1/query?operationName=fetchPlaylist&variables=%7B%22uri%22%3A%22spotify%3Aplaylist%3A37i9dQZF1DWZeKCadgRdKQ%22%2C%22offset%22%3A0%2C%22limit%22%3A25%7D&extensions=%7B%22persistedQuery%22%3A%7B%22version%22%3A1%2C%22sha256Hash%22%3A%225534e86cc2181b9e70be86ae26d514abd8d828be2ee56e5f8b7882dd70204c62%22%7D%7D";

    let url = &["https://api-partner.spotify.com/pathfinder/v1/query?operationName=fetchPlaylist&variables=%7B%22uri%22%3A%22spotify%3Aplaylist%3A", playlist_id,"%22%2C%22offset%22%3A0%2C%22limit%22%3A25%7D&extensions=%7B%22persistedQuery%22%3A%7B%22version%22%3A1%2C%22sha256Hash%22%3A%225534e86cc2181b9e70be86ae26d514abd8d828be2ee56e5f8b7882dd70204c62%22%7D%7D"].concat();

    log!("{url}");

    // create the request
    let request = match Request::new_with_str_and_init(&url, &opts) {
        Ok(v) => v,
        Err(e) => {
            log!("Spotify request creation failed");
            log!("{:?}", e);
            return;
        }
    };

    log!("Request created");

    // add headers
    let _ = request.headers().set("Accept", "application/json");
    // these keys were manually extracted from chrome webdev tools
    let _ = request.headers().set("authorization", auth_header_value);
    let _ = request.headers().set("client-token", token_header_value);

    log!("Headers set");

    let worker_global_scope = js_sys::global().unchecked_into::<WorkerGlobalScope>();
    let resp_value = match JsFuture::from(worker_global_scope.fetch_with_request(&request)).await {
        Ok(v) => v,
        Err(e) => {
            log!("Spotify request failed");
            log!("{:?}", e);
            return;
        }
    };

    // exit if the response is not of the expected type
    if !resp_value.is_instance_of::<Response>() {
        log!("Spotify response in not Response");
        log!("{:?}", resp_value);
        return;
    };

    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future` and execute it.
    // I think this is where the request is being sent to Spotify
    let resp = match resp.text() {
        Ok(v) => JsFuture::from(v).await,
        Err(e) => {
            log!("Cannot convert Promise to Future");
            log!("{:?}", e);
            return;
        }
    };

    log!("HTTP request completed");

    // Unwrap the response and handle the error
    let resp = match resp {
        Ok(v) => v,
        Err(e) => {
            log!("Spotify request failed");
            log!("{:?}", e);
            return;
        }
    };

    // log!("Resp as string:");
    // log!("{:?}", resp.as_string().unwrap());

    // convert into a rust struct
    let playlist = match serde_wasm_bindgen::from_value::<PlaylistRoot>(resp) {
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

// Playlist ownership -------------------------------------
#[derive(Debug, Deserialize)]
#[serde(rename(deserialize = "data"))]
struct OwnerV2Data {
    uri: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename(deserialize = "ownerV2"))]
struct OwnerV2 {
    data: OwnerV2Data,
}

// list of tracks ------------------------------------------
#[derive(Debug, Deserialize)]
struct ItemV2Data {
    uri: String,
}

#[derive(Debug, Deserialize)]
struct ItemV2 {
    data: ItemV2Data,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    item_v2: ItemV2,
}

#[derive(Debug, Deserialize)]
struct Content {
    items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaylistV2 {
    owner_v2: OwnerV2,
    content: Content,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Playlist {
    playlist_v2: PlaylistV2,
}

// The root node -------------------------------------------
#[derive(Debug, Deserialize)]
struct PlaylistRoot {
    data: Playlist,
}
