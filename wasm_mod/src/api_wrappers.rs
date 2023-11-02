use std::fmt::Debug;

// use std::time::Duration;
use crate::{
    constants,
    constants::log,
    models,
    models::{VariablesAlbumOrPlaylistTracks, VariablesV3Items},
    Result, RetryAfter,
};
use serde_wasm_bindgen;
use urlencoding;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, WorkerGlobalScope};

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

fn build_url<T>(operation_name: &str, variables: &T, persisted_query_hash: &str) -> Result<String>
where
    T: ?Sized + serde::Serialize + Debug,
{
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

    let persisted_query = [
        r#"{"persistedQuery":{"version":1,"sha256Hash":""#,
        persisted_query_hash,
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

/// Returns IDs of all album tracks.
pub(crate) async fn fetch_album_tracks(
    auth_header_value: &str,
    token_header_value: &str,
    album_id: &str,
    max_number_of_tracks: usize,
) -> Vec<String> {
    log!("fetch_album_tracks for: {album_id}");

    // request examples
    // https://api-partner.spotify.com/pathfinder/v1/query?operationName=getAlbum&variables=%7B%22uri%22%3A%22spotify%3Aalbum%3A4kWmZoSHYtMxvejnCwv3ID%22%2C%22locale%22%3A%22%22%2C%22offset%22%3A0%2C%22limit%22%3A50%7D&extensions=%7B%22persistedQuery%22%3A%7B%22version%22%3A1%2C%22sha256Hash%22%3A%2246ae954ef2d2fe7732b4b2b4022157b2e18b7ea84f70591ceb164e4de1b5d5d3%22%7D%7D
    // operationName: getAlbum
    // variables: {"uri":"spotify:album:4kWmZoSHYtMxvejnCwv3ID","locale":"","offset":0,"limit":50}
    // extensions: {"persistedQuery":{"version":1,"sha256Hash":"46ae954ef2d2fe7732b4b2b4022157b2e18b7ea84f70591ceb164e4de1b5d5d3"}}

    // get the first page of albums to see how many there are
    let mut variables = VariablesAlbumOrPlaylistTracks {
        uri: [constants::ID_PREFIX_ALBUM, album_id].concat(),
        ..Default::default()
    };

    let mut url = match build_url(
        constants::operations::ALBUM_TRACKS,
        &variables,
        constants::persistent_queries::GET_ALBUM,
    ) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    // get the list of album tracks from Spotify
    let album_tracks = match execute_http_request::<models::album::AlbumTracksRoot>(
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
    let total_track_count = album_tracks.data.album_union.tracks.total_count;
    let total_pages = total_track_count / constants::ITEMS_PER_PAGE;
    let total_pages = if total_track_count % constants::ITEMS_PER_PAGE > 0 {
        total_track_count / constants::ITEMS_PER_PAGE + 1
    } else {
        total_pages
    };
    log!("Tracks: {total_track_count}, pages: {total_pages}");

    // a collection of all albums from all pages
    let mut album_tracks = album_tracks
        .data
        .album_union
        .tracks
        .items
        .into_iter()
        .filter_map(|v| {
            if v.track.playability.playable {
                Some(v.track.uri.replace(constants::ID_PREFIX_TRACK, ""))
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

    // check if there are any more pages to fetch
    if total_track_count <= constants::ITEMS_PER_PAGE {
        log!("Playable tracks in {album_id}: {}", album_tracks.len());
        return album_tracks;
    }

    // allocate enough space for all tracks since it is known in advance
    album_tracks.reserve(total_track_count as usize - album_tracks.len());

    // the next page will start where the first one ended
    variables.offset = variables.limit;
    url = match build_url(
        constants::operations::ALBUM_TRACKS,
        &variables,
        constants::persistent_queries::GET_ALBUM,
    ) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    // fetch the rest of the track pages in a loop
    // this will end on the first error
    // TODO: make a more reliable loop when retries are available
    while let Ok(items) = execute_http_request::<models::album::AlbumTracksRoot>(
        auth_header_value,
        token_header_value,
        &url,
    )
    .await
    {
        // check if spotify returned any items at all
        if items.data.album_union.tracks.items.len() == 0 {
            log!("Spotify returned empty items list");
            log!("Playable tracks in {album_id}: {}", album_tracks.len());
            return album_tracks;
        }

        let mut items = items
            .data
            .album_union
            .tracks
            .items
            .into_iter()
            .filter_map(|v| {
                if v.track.playability.playable {
                    Some(v.track.uri.replace(constants::ID_PREFIX_TRACK, ""))
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        // add the list of items to the local collection
        album_tracks.append(&mut items);
        // the next page will start where the first one ended
        variables.offset += variables.limit;
        url = match build_url(
            constants::operations::ALBUM_TRACKS,
            &variables,
            constants::persistent_queries::GET_ALBUM,
        ) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };

        // protect against very large playlists
        if variables.offset >= max_number_of_tracks {
            log!(
                "Too many tracks ({}), limit {}",
                variables.offset,
                max_number_of_tracks
            );
            break;
        }

        // for debugging
        if variables.offset > 20 {
            break;
        }
    }

    log!("Playable tracks in {album_id}: {}", album_tracks.len());
    album_tracks
}

/// Returns IDs of all playlist tracks.
pub(crate) async fn fetch_playlist_tracks(
    auth_header_value: &str,
    token_header_value: &str,
    playlist_id: &str,
    max_number_of_tracks: usize,
) -> Vec<String> {
    log!("fetch_playlist_tracks for: {playlist_id}");

    // request examples
    // https://api-partner.spotify.com/pathfinder/v1/query?operationName=fetchPlaylist&variables=%7B%22uri%22%3A%22spotify%3Aplaylist%3A60C3LAHXga4hw73HeyU7U0%22%2C%22offset%22%3A0%2C%22limit%22%3A25%7D&extensions=%7B%22persistedQuery%22%3A%7B%22version%22%3A1%2C%22sha256Hash%22%3A%2273a3b3470804983e4d55d83cd6cc99715019228fd999d51429cc69473a18789d%22%7D%7D
    // operationName: fetchPlaylist
    // variables: {"uri":"spotify:playlist:60C3LAHXga4hw73HeyU7U0","offset":0,"limit":25}
    // extensions: {"persistedQuery":{"version":1,"sha256Hash":"73a3b3470804983e4d55d83cd6cc99715019228fd999d51429cc69473a18789d"}}

    // get the first page of albums to see how many there are
    let mut variables = VariablesAlbumOrPlaylistTracks {
        uri: [constants::ID_PREFIX_PLAYLIST, playlist_id].concat(),
        ..Default::default()
    };

    let mut url = match build_url(
        constants::operations::PLAYLIST_TRACKS,
        &variables,
        constants::persistent_queries::FETCH_PLAYLIST,
    ) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    // get the list of album tracks from Spotify
    let tracks = match execute_http_request::<models::playlist::PlaylistRoot>(
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
    let total_track_count = tracks.data.playlist_v2.content.total_count;
    let total_pages = total_track_count / constants::ITEMS_PER_PAGE;
    let total_pages = if total_track_count % constants::ITEMS_PER_PAGE > 0 {
        total_track_count / constants::ITEMS_PER_PAGE + 1
    } else {
        total_pages
    };
    log!("Tracks: {total_track_count}, pages: {total_pages}");

    // a collection of all albums from all pages
    let mut tracks = tracks
        .data
        .playlist_v2
        .content
        .items
        .into_iter()
        .filter_map(|v| {
            // if the track is playable and is actually a track, then it should be included
            // it may be possible to have other items other than tracks in a playlist (not sure)
            if v.item_v2.data.playability.playable
                && v.item_v2.data.uri.starts_with(constants::ID_PREFIX_TRACK)
            {
                Some(v.item_v2.data.uri.replace(constants::ID_PREFIX_TRACK, ""))
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

    // check if there are any more pages to fetch
    if total_track_count <= constants::ITEMS_PER_PAGE {
        log!("Playable tracks in {playlist_id}: {}", tracks.len());
        return tracks;
    }

    // allocate enough space for all tracks since it is known in advance
    tracks.reserve(total_track_count as usize - tracks.len());

    // the next page will start where the first one ended
    variables.offset = variables.limit;
    url = match build_url(
        constants::operations::PLAYLIST_TRACKS,
        &variables,
        constants::persistent_queries::FETCH_PLAYLIST,
    ) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    // fetch the rest of the track pages in a loop
    // this will end on the first error
    // TODO: make a more reliable loop when retries are available
    while let Ok(items) = execute_http_request::<models::playlist::PlaylistRoot>(
        auth_header_value,
        token_header_value,
        &url,
    )
    .await
    {
        // check if spotify returned any items at all
        if items.data.playlist_v2.content.items.len() == 0 {
            log!("Spotify returned empty items list");
            log!("Playable tracks in {playlist_id}: {}", tracks.len());
            return tracks;
        }

        let mut items = items
            .data
            .playlist_v2
            .content
            .items
            .into_iter()
            .filter_map(|v| {
                if v.item_v2.data.playability.playable && v.item_v2.data.uri.starts_with(constants::ID_PREFIX_TRACK) {
                    Some(v.item_v2.data.uri.replace(constants::ID_PREFIX_TRACK, ""))
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        // add the list of items to the local collection
        tracks.append(&mut items);
        // the next page will start where the first one ended
        variables.offset += variables.limit;
        url = match build_url(
            constants::operations::PLAYLIST_TRACKS,
            &variables,
            constants::persistent_queries::FETCH_PLAYLIST,
        ) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };

        // protect against very large playlists
        if variables.offset >= max_number_of_tracks {
            log!(
                "Too many tracks ({}), limit {}",
                variables.offset,
                max_number_of_tracks
            );
            break;
        }

        // for debugging
        if variables.offset > 20 {
            break;
        }
    }

    log!("Playable tracks in {playlist_id}: {}", tracks.len());
    tracks
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
    let mut variables = VariablesV3Items::default();
    variables.filters.push(filter.to_owned());

    let mut url = match build_url(
        constants::operations::ALBUMS_PLAYLISTS,
        &variables,
        constants::persistent_queries::LIBRARY_V3,
    ) {
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
    let total_pages = total_item_count / constants::ITEMS_PER_PAGE;
    let total_pages = if total_item_count % constants::ITEMS_PER_PAGE > 0 {
        total_item_count / constants::ITEMS_PER_PAGE + 1
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
    if total_item_count <= constants::ITEMS_PER_PAGE {
        log!("Total {filter}: {}", all_lib_v3_items.len());
        return all_lib_v3_items;
    }

    // allocate enough space for all albums since it is known in advance
    all_lib_v3_items.reserve(total_item_count as usize - all_lib_v3_items.len());

    // the next page will start where the first one ended
    variables.offset = variables.limit;
    url = match build_url(
        constants::operations::ALBUMS_PLAYLISTS,
        &variables,
        constants::persistent_queries::LIBRARY_V3,
    ) {
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
            .map(|v| v.item.data.uri.replace(constants::ID_PREFIX_ALBUM, ""))
            .collect::<Vec<String>>();

        // add the list of items to the local collection
        all_lib_v3_items.append(&mut items);
        // the next page will start where the first one ended
        variables.offset += variables.limit;
        url = match build_url(
            constants::operations::ALBUMS_PLAYLISTS,
            &variables,
            constants::persistent_queries::LIBRARY_V3,
        ) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };

        // for debugging
        if variables.offset > 50 {
            break;
        }
    }

    log!("Total {filter}: {}", all_lib_v3_items.len());
    all_lib_v3_items
}
