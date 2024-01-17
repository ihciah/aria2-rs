// Copied from aria2-ws-rs(https://github.com/WOo0W/aria2-ws-rs)
// All rights reserved by the original author.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};
use smol_str::SmolStr;

use crate::{SmallMap, SmallVec};

#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct TaskOptions {
    pub header: Option<SmallVec<SmolStr>>,

    #[serde_as(as = "Option<DisplayFromStr>")]
    pub split: Option<i32>,

    pub all_proxy: Option<SmolStr>,

    pub dir: Option<SmolStr>,

    pub out: Option<SmolStr>,

    pub gid: Option<SmolStr>,

    #[serde_as(as = "Option<DisplayFromStr>")]
    pub r#continue: Option<bool>,

    #[serde_as(as = "Option<DisplayFromStr>")]
    pub auto_file_renaming: Option<bool>,

    #[serde_as(as = "Option<DisplayFromStr>")]
    pub check_integrity: Option<bool>,

    /// Close connection if download speed is lower than or equal to this value(bytes per sec).
    ///
    /// 0 means aria2 does not have a lowest speed limit.
    ///
    /// You can append K or M (1K = 1024, 1M = 1024K).
    ///
    /// This option does not affect BitTorrent downloads.
    ///
    /// Default: 0
    pub lowest_speed_limit: Option<SmolStr>,

    /// Set max download speed per each download in bytes/sec. 0 means unrestricted.
    ///
    /// You can append K or M (1K = 1024, 1M = 1024K).
    ///
    /// To limit the overall download speed, use --max-overall-download-limit option.
    ///
    /// Default: 0
    pub max_download_limit: Option<SmolStr>,

    #[serde_as(as = "Option<DisplayFromStr>")]
    pub max_connection_per_server: Option<i32>,

    #[serde_as(as = "Option<DisplayFromStr>")]
    pub max_tries: Option<i32>,

    #[serde_as(as = "Option<DisplayFromStr>")]
    pub timeout: Option<i32>,

    #[serde(flatten)]
    pub extra_options: SmallMap<SmolStr, Value>,
}
