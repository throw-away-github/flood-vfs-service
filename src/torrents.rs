use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Torrents {
    id: i64,
    pub(crate) torrents: HashMap<String, Torrent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Torrent {
    #[serde(rename = "bytesDone")]
    bytes_done: i64,
    #[serde(rename = "dateActive")]
    date_active: i64,
    #[serde(rename = "dateAdded")]
    date_added: i64,
    #[serde(rename = "dateCreated")]
    date_created: i64,
    #[serde(rename = "dateFinished")]
    date_finished: i64,
    pub(crate) directory: PathBuf,
    #[serde(rename = "downRate")]
    down_rate: i64,
    #[serde(rename = "downTotal")]
    down_total: i64,
    eta: i64,
    hash: String,
    #[serde(rename = "isPrivate")]
    is_private: bool,
    #[serde(rename = "isInitialSeeding")]
    is_initial_seeding: bool,
    #[serde(rename = "isSequential")]
    is_sequential: bool,
    message: String,
    pub(crate) name: String,
    #[serde(rename = "peersConnected")]
    peers_connected: i64,
    #[serde(rename = "peersTotal")]
    peers_total: i64,
    #[serde(rename = "percentComplete")]
    pub(crate) percent_complete: f64,
    priority: i64,
    ratio: f64,
    #[serde(rename = "seedsConnected")]
    seeds_connected: i64,
    #[serde(rename = "seedsTotal")]
    seeds_total: i64,
    #[serde(rename = "sizeBytes")]
    size_bytes: i64,
    status: Vec<Status>,
    tags: Vec<Tag>,
    #[serde(rename = "trackerURIs")]
    tracker_ur_is: Vec<String>,
    #[serde(rename = "upRate")]
    up_rate: i64,
    #[serde(rename = "upTotal")]
    up_total: i64,
}

#[derive(Deserialize_enum_str, Serialize_enum_str, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Complete,
    Downloading,
    Active,
    Inactive,
    Seeding,
    Stopped,
    #[serde(other)]
    Other(String),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Tag {
    #[serde(rename = "radarr")]
    Radarr,
    #[serde(rename = "radarr_imported")]
    RadarrImported,
    #[serde(rename = "sonarr")]
    Sonarr,
    #[serde(rename = "sonarr4K")]
    Sonarr4K,
    #[serde(rename = "sonarr4K_imported")]
    Sonarr4KImported,
    #[serde(rename = "sonarr_imported")]
    SonarrImported,
    #[serde(rename = "sonarr4k_imported")]
    TagSonarr4KImported,
}
