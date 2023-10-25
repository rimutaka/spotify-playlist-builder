// use std::time::Duration;
use super::{models, models::Variables, Result, RetryAfter};
use hex;
use serde_wasm_bindgen;
use sha2::{Digest, Sha256};
use urlencoding;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, WorkerGlobalScope};

/// Number of items (albums, tracks) per page for Spotify requests
const ITEMS_PER_PAGE: u64 = 50;
const OPERATION_NAME_ALBUMS_PLAYLISTS: &str = "libraryV3";
const ID_PREFIX_ALBUM: &str = "spotify:album:";
const ID_PREFIX_PLAYLIST: &str = "spotify:playlist:";

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
        web_sys::console::log_1(&format!( $( $t )* ).into())
    }
}

/// Prepares and executes an HTTP request to spotify,
/// including token and other headers.
async fn execute_http_request<T>(
    auth_header_value: &str,
    token_header_value: &str,
    url: &str,
) -> Result<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
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

    // convert into a rust struct
    let playlist = match serde_wasm_bindgen::from_value::<T>(resp) {
        Ok(v) => v,
        Err(e) => {
            log!("Cannot deser spotify response into rust struct");
            log!("{:?}", e);
            return Err(RetryAfter::Never);
        }
    };

    return Ok(playlist);
}

/**
/// Fetches details of the specified playlist, including its list of tracks.
/// Nothing is returned yet - the data is logged for debugging.
async fn _fetch_playlist(
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

    let playlist = match execute_http_request::<models::playlist::PlaylistRoot>(
        auth_header_value,
        token_header_value,
        url,
    )
    .await
    {
        Ok(v) => v,
        Err(e) if matches!(e, RetryAfter::Never) => {
            return;
        }
        Err(_) => {
            unimplemented!("Retries are not implemented");
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

*/

fn build_url(operation_name: &str, variables: &Variables) -> Result<String> {
    let variables = match serde_json::to_string(variables) {
        Ok(v) => v,
        Err(e) => {
            log!("Failed to serialize variables for {:?}", variables);
            log!("{e}");
            return Err(RetryAfter::Never);
        }
    };

    let variables = urlencoding::encode(&variables).to_string();

    let url = &[
        "https://api-partner.spotify.com/pathfinder/v1/query?operationName=",
        operation_name,
        "&variables=",
        &variables,
    ]
    .concat();

    // hash the URL for using as a cache key
    let mut hasher = Sha256::new();
    hasher.update(url);
    let _hashed_url = hex::encode(hasher.finalize());

    // this is a temp plug because I could not get Spotify to accept my hash
    let hashed_url = "17d801ba80f3a3d7405966641818c334fe32158f97e9e8b38f1a92f764345df9";

    let persisted_query = [
        r#"{"persistedQuery":{"version":1,"sha256Hash":""#,
        &hashed_url,
        r#""}}""#,
    ]
    .concat();

    let url = &[
        url,
        "&extensions=",
        &urlencoding::encode(&persisted_query).to_string(),
    ]
    .concat();

    log!("URL:");
    log!("{url}");

    return Ok(url.to_owned());
}

/// Fetches IDs of all albums and playlists
pub(crate) async fn fetch_all_albums_and_playlists(
    auth_header_value: &str,
    token_header_value: &str,
) {
    log!("fetch_all_albums entered");

    // collect all album IDs

    // request examples
    // https://api-partner.spotify.com/pathfinder/v1/query?operationName=libraryV3&variables=%7B%22filters%22%3A%5B%22Albums%22%5D%2C%22order%22%3Anull%2C%22textFilter%22%3A%22%22%2C%22features%22%3A%5B%22LIKED_SONGS%22%2C%22YOUR_EPISODES%22%5D%2C%22limit%22%3A50%2C%22offset%22%3A0%2C%22flatten%22%3Afalse%2C%22expandedFolders%22%3A%5B%5D%2C%22folderUri%22%3Anull%2C%22includeFoldersWhenFlattening%22%3Atrue%2C%22withCuration%22%3Afalse%7D&extensions=%7B%22persistedQuery%22%3A%7B%22version%22%3A1%2C%22sha256Hash%22%3A%2217d801ba80f3a3d7405966641818c334fe32158f97e9e8b38f1a92f764345df9%22%7D%7D
    // https://api-partner.spotify.com/pathfinder/v1/query?operationName=libraryV3&variables={"filters":["Albums"],"order":null,"textFilter":"","features":["LIKED_SONGS","YOUR_EPISODES"],"limit":50,"offset":0,"flatten":false,"expandedFolders":[],"folderUri":null,"includeFoldersWhenFlattening":true,"withCuration":false}&extensions={"persistedQuery":{"version":1,"sha256Hash":"17d801ba80f3a3d7405966641818c334fe32158f97e9e8b38f1a92f764345df9"}}

    let all_albums = fetch_lib_v3_items(auth_header_value, token_header_value, "Albums").await;

    // remove the repetitive prefix
    let all_albums = all_albums
        .into_iter()
        .map(|v| v.replace(ID_PREFIX_ALBUM, ""))
        .collect::<Vec<String>>();

    log!("{}", all_albums.join("\n"));

    // repeat the same for playlists - collect all playlist IDs

    // request examples
    // https://api-partner.spotify.com/pathfinder/v1/query?operationName=libraryV3&variables=%7B%22filters%22%3A%5B%22Playlists%22%5D%2C%22order%22%3Anull%2C%22textFilter%22%3A%22%22%2C%22features%22%3A%5B%22LIKED_SONGS%22%2C%22YOUR_EPISODES%22%5D%2C%22limit%22%3A50%2C%22offset%22%3A6%2C%22flatten%22%3Afalse%2C%22expandedFolders%22%3A%5B%5D%2C%22folderUri%22%3Anull%2C%22includeFoldersWhenFlattening%22%3Atrue%2C%22withCuration%22%3Afalse%7D&extensions=%7B%22persistedQuery%22%3A%7B%22version%22%3A1%2C%22sha256Hash%22%3A%2217d801ba80f3a3d7405966641818c334fe32158f97e9e8b38f1a92f764345df9%22%7D%7D
    // operationName: libraryV3
    // variables: {"filters":["Playlists"],"order":null,"textFilter":"","features":["LIKED_SONGS","YOUR_EPISODES"],"limit":50,"offset":6,"flatten":false,"expandedFolders":[],"folderUri":null,"includeFoldersWhenFlattening":true,"withCuration":false}

    let all_playlists =
        fetch_lib_v3_items(auth_header_value, token_header_value, "Playlists").await;

    // remove the repetitive prefix
    let all_playlists = all_playlists
        .into_iter()
        .map(|v| v.replace(ID_PREFIX_PLAYLIST, ""))
        .collect::<Vec<String>>();

    log!("{}", all_playlists.join("\n"));
}

/// Returns IDs of either albums or playlists.
///
/// * filter - either Albums or Playlists, goes into request vars
pub(crate) async fn fetch_lib_v3_items(
    auth_header_value: &str,
    token_header_value: &str,
    filter: &str,
) -> Vec<String> {
    log!("fetch_lib_v3_items entered, filter: {filter}");

    // request examples
    // https://api-partner.spotify.com/pathfinder/v1/query?operationName=libraryV3&variables=%7B%22filters%22%3A%5B%22Playlists%22%5D%2C%22order%22%3Anull%2C%22textFilter%22%3A%22%22%2C%22features%22%3A%5B%22LIKED_SONGS%22%2C%22YOUR_EPISODES%22%5D%2C%22limit%22%3A50%2C%22offset%22%3A6%2C%22flatten%22%3Afalse%2C%22expandedFolders%22%3A%5B%5D%2C%22folderUri%22%3Anull%2C%22includeFoldersWhenFlattening%22%3Atrue%2C%22withCuration%22%3Afalse%7D&extensions=%7B%22persistedQuery%22%3A%7B%22version%22%3A1%2C%22sha256Hash%22%3A%2217d801ba80f3a3d7405966641818c334fe32158f97e9e8b38f1a92f764345df9%22%7D%7D
    // operationName: libraryV3
    // variables: {"filters":["Playlists"],"order":null,"textFilter":"","features":["LIKED_SONGS","YOUR_EPISODES"],"limit":50,"offset":6,"flatten":false,"expandedFolders":[],"folderUri":null,"includeFoldersWhenFlattening":true,"withCuration":false}

    // get the first page of albums to see how many there are
    let mut variables = Variables::default();
    variables.filters.push(filter.to_owned());

    let mut url = match build_url(OPERATION_NAME_ALBUMS_PLAYLISTS, &variables) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    // the part of the structure we use for our needs are identical between albums and playlists
    let lib_v3_items = match execute_http_request::<models::albums_playlists::LibV3ItemsRoot>(
        auth_header_value,
        token_header_value,
        &url,
    )
    .await
    {
        Ok(v) => v,
        Err(e) if matches!(e, RetryAfter::Never) => {
            return Vec::new();
        }
        Err(_) => {
            unimplemented!("Retries are not implemented");
        }
    };

    // log!("{:?}", lib_v3_items);

    // get the number of lib_v3_items and calculate the number of pages that can be downloaded
    let total_item_count = lib_v3_items.data.me.library_v3.total_count;
    let total_pages = total_item_count / ITEMS_PER_PAGE;
    let total_pages = if total_item_count % ITEMS_PER_PAGE > 0 {
        total_item_count / ITEMS_PER_PAGE + 1
    } else {
        total_pages
    };
    log!("Items: {total_item_count}, pages: {total_pages}");

    // a collection of all albums from all pages
    let mut all_lib_v3_items = lib_v3_items
        .data
        .me
        .library_v3
        .items
        .into_iter()
        .map(|v| v.item.data.uri)
        .collect::<Vec<String>>();

    // check if there are any more pages to fetch
    if total_item_count <= ITEMS_PER_PAGE {
        log!("Total {filter}: {}", all_lib_v3_items.len());
        return all_lib_v3_items;
    }

    // allocate enough space for all albums since it is known in advance
    all_lib_v3_items.reserve(total_item_count as usize - all_lib_v3_items.len());

    // the next page will start where the first one ended
    variables.offset = variables.limit;
    url = match build_url(OPERATION_NAME_ALBUMS_PLAYLISTS, &variables) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    // fetch the rest of the item pages in a loop
    // this will end on the first error
    // TODO: make a more reliable loop when retries are available
    while let Ok(items) = execute_http_request::<models::albums_playlists::LibV3ItemsRoot>(
        auth_header_value,
        token_header_value,
        &url,
    )
    .await
    {

        // check if spotify returned any items at all
        if items.data.me.library_v3.items.len() == 0 {
            log!("Spotify returned empty items list");
            log!("Total {filter}: {}", all_lib_v3_items.len());
            return all_lib_v3_items;
        }

        let mut items = items
            .data
            .me
            .library_v3
            .items
            .into_iter()
            .map(|v| v.item.data.uri.replace(ID_PREFIX_ALBUM, ""))
            .collect::<Vec<String>>();

        // add the list of items to the local collection
        all_lib_v3_items.append(&mut items);
        // the next page will start where the first one ended
        variables.offset += variables.limit;
        url = match build_url(OPERATION_NAME_ALBUMS_PLAYLISTS, &variables) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };

        // for debugging
        if variables.offset > 200 {
            break;
        }
    }

    log!("Total {filter}: {}", all_lib_v3_items.len());
    all_lib_v3_items
}
