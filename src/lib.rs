pub mod call;
mod client;
mod error;
pub mod status;

pub type SmallMap<K, V> = small_map::FxSmallMap<4, K, V>;
pub type SmallVec<T> = smallvec::SmallVec<[T; 4]>;
pub type SmallString = smol_str::SmolStr;
pub type Result<T> = std::result::Result<T, Error>;

pub use call::{Call, MultiResponse, Reply, OK};
pub use client::{BatchClient, Client, ConnectionMeta, NotificationCallback};
pub use error::{Error, RpcError};
