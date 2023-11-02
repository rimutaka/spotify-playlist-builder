use std::collections::HashSet;

// use std::time::Duration;
use crate::{
    api_wrappers::{fetch_album_tracks, fetch_lib_v3_items, fetch_playlist_tracks},
    constants,
    constants::log,
};
use rand::seq::SliceRandom;

/// Fetches IDs of all albums and playlists
pub(crate) async fn fetch_all_albums_and_playlists(
    auth_header_value: &str,
    token_header_value: &str,
) {
    log!("fetch_all_albums entered");

    // collect all album IDs
    let all_albums = fetch_lib_v3_items(auth_header_value, token_header_value, "Albums").await;

    // remove the repetitive prefix
    let mut all_albums = all_albums
        .into_iter()
        .map(|v| v.replace(constants::ID_PREFIX_ALBUM, ""))
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
    let mut all_playlists = all_playlists
        .into_iter()
        .map(|v| v.replace(constants::ID_PREFIX_PLAYLIST, ""))
        .collect::<Vec<String>>();

    log!("{}", all_playlists.join("\n"));

    // randomize the list of albums and playlists
    let mut rng = rand::thread_rng();
    all_playlists.shuffle(&mut rng);
    all_albums.shuffle(&mut rng);

    // a list of tracks selected for the addition to the new playlist
    let mut selected_tracks: HashSet<String> = HashSet::new();

    // a list of tracks per that were not selected
    let mut unselected_tracks: HashSet<String> = HashSet::new();

    // go thru all albums
    for album_id in all_albums {
        // get album tracks, shuffle and add top N tracks to the selected list
        let mut album_tracks =
            fetch_album_tracks(auth_header_value, token_header_value, &album_id, 50).await;

        if album_tracks.is_empty() {
            log!("Empty album {album_id}");
            continue;
        } else if album_tracks.len() <= constants::MIN_TRACKS_PER_ALBUM {
            // the album is too small and all tracks should be added
            log!(
                "Sel: {}, stash: {}, adding all {} tracks from album {album_id}",
                selected_tracks.len(),
                unselected_tracks.len(),
                album_tracks.len(),
            );
            let _: Vec<_> = album_tracks
                .into_iter()
                .map(|v| selected_tracks.insert(v))
                .collect();
        } else {
            // select N random tracks
            album_tracks.shuffle(&mut rng);
            let _: Vec<_> = album_tracks
                .drain(..constants::MIN_TRACKS_PER_ALBUM)
                .map(|v| selected_tracks.insert(v))
                .collect();

            // stash the remaining tracks
            let _: Vec<_> = album_tracks
                .into_iter()
                .map(|v| unselected_tracks.insert(v))
                .collect();
            log!(
                "Sel: {}, stash: {}, added {} tracks from album {album_id}",
                selected_tracks.len(),
                unselected_tracks.len(),
                constants::MIN_TRACKS_PER_ALBUM,
            );
        }

        // exit if there are enough tracks for the playlist
        if selected_tracks.len() >= constants::TARGET_PLAYLIST_SIZE {
            break;
        }
    }

    log!(
        "Selected tracks: {}, stash tracks: {}",
        selected_tracks.len(),
        unselected_tracks.len(),
    );

    // go thru all playlists
    // TODO: merge this with the album loop, but I do not know how to do it in terms of priorities and what to pick from where in what order
    // to make it a more representative sample. Large playlists may dominate and skew the results.
    for playlist_id in all_playlists {
        // get album tracks, shuffle and add top N tracks to the selected list
        let mut tracks =
            fetch_playlist_tracks(auth_header_value, token_header_value, &playlist_id, 50).await;

        if tracks.is_empty() {
            log!("Empty playlist {playlist_id}");
            continue;
        } else if tracks.len() <= constants::MIN_TRACKS_PER_ALBUM {
            // the playlist is too small and all tracks should be added
            log!(
                "Sel: {}, stash: {}, adding all {} tracks from playlist {playlist_id}",
                selected_tracks.len(),
                unselected_tracks.len(),
                tracks.len(),
            );
            let _: Vec<_> = tracks
                .into_iter()
                .map(|v| selected_tracks.insert(v))
                .collect();
        } else {
            // select N random tracks
            tracks.shuffle(&mut rng);
            let _: Vec<_> = tracks
                .drain(..constants::MIN_TRACKS_PER_ALBUM)
                .map(|v| selected_tracks.insert(v))
                .collect();

            // stash the remaining tracks
            let _: Vec<_> = tracks
                .into_iter()
                .map(|v| unselected_tracks.insert(v))
                .collect();
            log!(
                "Sel: {}, stash: {}, added {} tracks from playlist {playlist_id}",
                selected_tracks.len(),
                unselected_tracks.len(),
                constants::MIN_TRACKS_PER_ALBUM,
            );
        }

        // exit if there are enough tracks for the playlist
        if selected_tracks.len() >= constants::TARGET_PLAYLIST_SIZE {
            break;
        }
    }

    log!(
        "Selected tracks: {}, stashed tracks: {}",
        selected_tracks.len(),
        unselected_tracks.len()
    );

    log!("Selected tracks:");
    log!(
        "{}",
        selected_tracks
            .into_iter()
            .collect::<Vec<String>>()
            .join("\n")
    );

    // start getting tracks per album / playlist
}
