use serde::Serialize;

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
pub(crate) struct VariablesAlbumTracks {
    pub limit: usize,
    pub offset: usize,
    pub uri: String,
    pub locale: String,
}

impl Default for VariablesAlbumTracks {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
            uri: String::new(),
            locale: String::new(),
        }
    }
}

pub(crate) mod playlist {
    use serde::Deserialize;
    // Playlist ownership -------------------------------------
    #[derive(Debug, Deserialize)]
    #[serde(rename(deserialize = "data"))]
    pub struct OwnerV2Data {
        pub uri: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename(deserialize = "ownerV2"))]
    pub struct OwnerV2 {
        pub data: OwnerV2Data,
    }

    // list of tracks ------------------------------------------
    #[derive(Debug, Deserialize)]
    pub struct ItemV2Data {
        pub uri: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct ItemV2 {
        pub data: ItemV2Data,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Item {
        pub item_v2: ItemV2,
    }

    #[derive(Debug, Deserialize)]
    pub struct Content {
        pub items: Vec<Item>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PlaylistV2 {
        pub owner_v2: OwnerV2,
        pub content: Content,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Playlist {
        pub playlist_v2: PlaylistV2,
    }

    // The root node -------------------------------------------
    #[derive(Debug, Deserialize)]
    pub struct PlaylistRoot {
        pub data: Playlist,
    }
}
// --- ALBUMS ---------------------------------------------

// data -> me -> libraryV3 -> items -> item -> data -> uri

pub(crate) mod albums_playlists {
    use serde::Deserialize;
    #[derive(Debug, Deserialize)]
    pub struct Data {
        pub uri: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Item {
        pub data: Data,
    }

    #[derive(Debug, Deserialize)]
    pub struct Items {
        pub item: Item,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LibraryV3 {
        pub items: Vec<Items>,
        pub total_count: usize,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Me {
        pub library_v3: LibraryV3,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RootData {
        pub me: Me,
    }

    #[derive(Debug, Deserialize)]
    pub struct LibV3ItemsRoot {
        pub data: RootData,
    }
}

// data -> albumUnion -> tracks -> items -> [track -> uri]
// ... tracks -> totalCount
// ... track -> playability -> playable

pub(crate) mod album_tracks {
    use serde::Deserialize;
    #[derive(Debug, Deserialize)]
    pub struct Playability {
        pub playable: bool,
    }

    #[derive(Debug, Deserialize)]
    pub struct Track {
        pub uri: String,
        pub playability: Playability,
    }

    #[derive(Debug, Deserialize)]
    pub struct Items {
        pub track: Track,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Tracks {
        pub items: Vec<Items>,
        pub total_count: usize,
    }

    #[derive(Debug, Deserialize)]
    pub struct AlbumUnion {
        pub tracks: Tracks,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RootData {
        pub album_union: AlbumUnion,
    }

    #[derive(Debug, Deserialize)]
    pub struct AlbumTracksRoot {
        pub data: RootData,
    }
}
