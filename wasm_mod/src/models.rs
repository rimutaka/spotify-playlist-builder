use serde::Serialize;

/// A serializable struct for adding variables to a spotify request
///
/// {"filters":["Albums"],"order":null,"textFilter":"","features":["LIKED_SONGS","YOUR_EPISODES"],"limit":50,"offset":0,"flatten":false,"expandedFolders":[],"folderUri":null,"includeFoldersWhenFlattening":true,"withCuration":false}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Variables {
    pub filters: Vec<String>,
    pub order: String,
    pub limit: usize,
    pub offset: usize,
    pub text_filter: String,
}

impl Default for Variables {
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

pub(crate) mod albums {
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
        pub total_count: u64,
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
    pub struct Albums {
        pub data: RootData,
    }
}
