use serde::Serialize;

pub(crate) mod album;
pub(crate) mod albums_playlists;
pub(crate) mod playlist;

/// A serializable struct for adding variables to a spotify request
///
/// {"filters":["Albums"],"order":null,"textFilter":"","features":["LIKED_SONGS","YOUR_EPISODES"],"limit":50,"offset":0,"flatten":false,"expandedFolders":[],"folderUri":null,"includeFoldersWhenFlattening":true,"withCuration":false}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VariablesV3Items {
    pub filters: Vec<String>,
    pub order: String,
    pub limit: usize,
    pub offset: usize,
    pub text_filter: String,
}

impl Default for VariablesV3Items {
    fn default() -> Self {
        Self {
            filters: Vec::new(),
            order: "Recently Added".to_owned(),
            limit: 50,
            offset: 0,
            text_filter: String::new(),
        }
    }
}

/// A serializable struct for adding variables to a spotify request
///
/// {"uri":"spotify:album:7h5qFidHM4sqhcCHSbiMzL","locale":"","offset":0,"limit":50}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VariablesAlbumOrPlaylistTracks {
    pub limit: usize,
    pub offset: usize,
    pub uri: String,
    pub locale: String,
}

impl Default for VariablesAlbumOrPlaylistTracks {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
            uri: String::new(),
            locale: String::new(),
        }
    }
}
