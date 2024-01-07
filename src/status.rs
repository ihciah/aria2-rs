use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StatusKey {
    Gid,
    Status,
    TotalLength,
    CompletedLength,
    UploadLength,
    Bitfield,
    DownloadSpeed,
    UploadSpeed,
    InfoHash,
    NumSeeders,
    Seeder,
    PieceLength,
    NumPieces,
    Connections,
    ErrorCode,
    ErrorMessage,
    FollowedBy,
    Following,
    BelongsTo,
    Dir,
    Files,
    Bittorrent,
    VerifiedLength,
    VerifyIntegrityPending,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub gid: Option<SmolStr>,
    pub status: Option<TaskStatus>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub total_length: Option<u64>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub completed_length: Option<u64>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub upload_length: Option<u64>,
    pub bitfield: Option<String>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub download_speed: Option<u64>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub upload_speed: Option<u64>,
    pub info_hash: Option<String>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub num_seeders: Option<u64>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub seeder: Option<bool>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub piece_length: Option<u64>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub num_pieces: Option<u64>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub connections: Option<u64>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub followed_by: Option<Vec<String>>,
    pub following: Option<String>,
    pub belongs_to: Option<String>,
    pub dir: Option<String>,
    pub files: Option<Vec<File>>,
    pub bittorrent: Option<BittorrentStatus>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
#[repr(u8)]
pub enum TaskStatus {
    Active,
    Waiting,
    Paused,
    Error,
    Complete,
    Removed,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct File {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub index: u64,
    pub path: String,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub length: u64,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub completed_length: u64,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub selected: bool,
    pub uris: Vec<Uri>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Uri {
    pub uri: String,
    pub status: UriStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum UriStatus {
    Used,
    Waiting,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BittorrentStatus {
    pub announce_list: Vec<Vec<String>>,
    pub comment: Option<String>,
    #[serde_as(as = "Option<serde_with::TimestampSeconds<i64>>")]
    pub creation_date: Option<SystemTime>,
    pub mode: Option<BitTorrentMode>,
    pub info: Option<BittorrentInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BitTorrentMode {
    Single,
    Multi,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BittorrentInfo {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StatKey {
    DownloadSpeed,
    UploadSpeed,
    NumActive,
    NumWaiting,
    NumStopped,
    NumStoppedTotal,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Stat {
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub download_speed: Option<u64>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub upload_speed: Option<u64>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub num_active: Option<u32>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub num_waiting: Option<u32>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub num_stopped: Option<u32>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub num_stopped_total: Option<u32>,
}
