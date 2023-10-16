// use std::time::Duration;
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
