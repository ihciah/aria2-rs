use std::borrow::Cow;

use serde::{de::DeserializeOwned, ser::SerializeSeq as _, Deserialize, Serializer};
use serde_json::Value;

use crate::{options::TaskOptions, SmallString, SmallVec};

type SerializeSeq = <serde_json::value::Serializer as serde::ser::Serializer>::SerializeSeq;
type JsonError = serde_json::Error;

macro_rules! option {
    ($opt: expr, $serializer: expr) => {
        if let Some(value) = $opt {
            $serializer.serialize_element(value)?;
        }
    };
}
macro_rules! empty {
    ($map: expr, $serializer: expr) => {
        if !$map.is_empty() {
            $serializer.serialize_element($map)?;
        }
    };
}

pub trait Reply {
    type Reply: DeserializeOwned;
    #[inline]
    fn to_reply(value: Value) -> crate::Result<Self::Reply> {
        serde_json::from_value::<Self::Reply>(value).map_err(crate::Error::Decode)
    }
}

pub trait Call {
    // Why not define as a const: to make it object safe.
    fn method(&self) -> &'static str;
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError>;

    fn to_param(&self, token: Option<&str>) -> std::result::Result<Value, serde_json::Error> {
        let mut serializer = serde_json::value::Serializer.serialize_seq(None)?;
        self.serialize_params(&mut serializer, token)?;
        serializer.end()
    }
}

pub struct AddUriCall {
    pub uris: SmallVec<String>,
    pub options: Option<TaskOptions>,
}

impl Reply for AddUriCall {
    type Reply = GidReply;
}

#[derive(Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct GidReply(pub SmallString);

impl Call for AddUriCall {
    fn method(&self) -> &'static str {
        "aria2.addUri"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        option!(token, serializer);
        serializer.serialize_element(&self.uris)?;
        option!(&self.options, serializer);
        Ok(())
    }
}

pub struct AddTorrentCall<'a> {
    pub torrent: Cow<'a, [u8]>,
    pub uris: SmallVec<Cow<'a, str>>,
    pub options: Option<TaskOptions>,
}

impl Reply for AddTorrentCall<'_> {
    type Reply = GidReply;
}

impl Call for AddTorrentCall<'_> {
    fn method(&self) -> &'static str {
        "aria2.addTorrent"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        use base64::{engine::general_purpose, Engine as _};
        let encoded: String = general_purpose::STANDARD.encode(&self.torrent);

        option!(token, serializer);
        serializer.serialize_element(&encoded)?;
        serializer.serialize_element(&self.uris)?;
        option!(&self.options, serializer);
        Ok(())
    }
}

pub struct AddMetalinkCall<'a> {
    pub metalink: Cow<'a, str>,
    pub options: Option<TaskOptions>,
}

impl Reply for AddMetalinkCall<'_> {
    type Reply = Vec<GidReply>;
}

impl Call for AddMetalinkCall<'_> {
    fn method(&self) -> &'static str {
        "aria2.addMetalink"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        option!(token, serializer);
        serializer.serialize_element(&self.metalink)?;
        option!(&self.options, serializer);
        Ok(())
    }
}

pub struct RemoveCall<'a> {
    pub gid: Cow<'a, str>,
}

impl Reply for RemoveCall<'_> {
    type Reply = GidReply;
}

impl Call for RemoveCall<'_> {
    fn method(&self) -> &'static str {
        "aria2.remove"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        option!(token, serializer);
        serializer.serialize_element(&self.gid)?;
        Ok(())
    }
}

pub struct ForceRemoveCall<'a> {
    pub gid: Cow<'a, str>,
}

impl Reply for ForceRemoveCall<'_> {
    type Reply = GidReply;
}

impl Call for ForceRemoveCall<'_> {
    fn method(&self) -> &'static str {
        "aria2.forceRemove"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        option!(token, serializer);
        serializer.serialize_element(&self.gid)?;
        Ok(())
    }
}

pub struct PauseCall<'a> {
    pub gid: Cow<'a, str>,
}

impl Reply for PauseCall<'_> {
    type Reply = GidReply;
}

impl Call for PauseCall<'_> {
    fn method(&self) -> &'static str {
        "aria2.pause"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        option!(token, serializer);
        serializer.serialize_element(&self.gid)?;
        Ok(())
    }
}

pub struct ForcePauseCall<'a> {
    pub gid: Cow<'a, str>,
}

impl Reply for ForcePauseCall<'_> {
    type Reply = GidReply;
}

impl Call for ForcePauseCall<'_> {
    fn method(&self) -> &'static str {
        "aria2.forcePause"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        option!(token, serializer);
        serializer.serialize_element(&self.gid)?;
        Ok(())
    }
}

pub struct UnpauseCall<'a> {
    pub gid: Cow<'a, str>,
}

impl Reply for UnpauseCall<'_> {
    type Reply = GidReply;
}

impl Call for UnpauseCall<'_> {
    fn method(&self) -> &'static str {
        "aria2.unpause"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        option!(token, serializer);
        serializer.serialize_element(&self.gid)?;
        Ok(())
    }
}

pub struct TellStatusCall<'a> {
    pub gid: Cow<'a, str>,
    pub keys: SmallVec<crate::status::StatusKey>,
}

impl Reply for TellStatusCall<'_> {
    type Reply = crate::status::Status;
}

impl Call for TellStatusCall<'_> {
    fn method(&self) -> &'static str {
        "aria2.tellStatus"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        option!(token, serializer);
        serializer.serialize_element(&self.gid)?;
        empty!(&self.keys, serializer);
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct TellActiveCall {
    pub keys: SmallVec<crate::status::StatusKey>,
}

impl Reply for TellActiveCall {
    type Reply = Vec<crate::status::Status>;
}

impl Call for TellActiveCall {
    fn method(&self) -> &'static str {
        "aria2.tellActive"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        option!(token, serializer);
        empty!(&self.keys, serializer);
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct TellWaitingCall {
    pub offset: i32,
    pub num: i32,
    pub keys: SmallVec<crate::status::StatusKey>,
}

impl Reply for TellWaitingCall {
    type Reply = Vec<crate::status::Status>;
}

impl Call for TellWaitingCall {
    fn method(&self) -> &'static str {
        "aria2.tellWaiting"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        option!(token, serializer);
        serializer.serialize_element(&self.offset)?;
        serializer.serialize_element(&self.num)?;
        empty!(&self.keys, serializer);
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct TellStoppedCall {
    pub offset: i32,
    pub num: i32,
    pub keys: SmallVec<crate::status::StatusKey>,
}

impl Reply for TellStoppedCall {
    type Reply = Vec<crate::status::Status>;
}

impl Call for TellStoppedCall {
    fn method(&self) -> &'static str {
        "aria2.tellStopped"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        option!(token, serializer);
        serializer.serialize_element(&self.offset)?;
        serializer.serialize_element(&self.num)?;
        empty!(&self.keys, serializer);
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct GetGlobalStatCall {
    pub keys: SmallVec<crate::status::StatKey>,
}

impl Reply for GetGlobalStatCall {
    type Reply = crate::status::Stat;
}

impl Call for GetGlobalStatCall {
    fn method(&self) -> &'static str {
        "aria2.getGlobalStat"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        option!(token, serializer);
        empty!(&self.keys, serializer);
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PurgeDownloadResultCall;

#[derive(Debug, Clone)]
pub enum OK {
    Ok,
    Err(String),
}

impl<'de> serde::de::Deserialize<'de> for OK {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|s| if s == "OK" { OK::Ok } else { OK::Err(s) })
    }
}

impl Reply for PurgeDownloadResultCall {
    type Reply = OK;
}

impl Call for PurgeDownloadResultCall {
    fn method(&self) -> &'static str {
        "aria2.purgeDownloadResult"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        option!(token, serializer);
        Ok(())
    }
}

#[derive(Default)]
pub struct MultiCall<'a> {
    pub calls: Vec<Box<dyn Call + Send + Sync + 'a>>,
}

#[derive(Debug, Clone)]
pub struct MultiResponse(pub SmallVec<Value>);

impl<'de> Deserialize<'de> for MultiResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = SmallVec::<SmallVec<Value>>::deserialize(deserializer)?;
        Ok(MultiResponse(inner.into_iter().flatten().collect()))
    }
}

impl Reply for MultiCall<'_> {
    type Reply = MultiResponse;
}

impl<'a> MultiCall<'a> {
    pub const fn new() -> Self {
        Self { calls: Vec::new() }
    }
    pub fn push<T: Call + Send + Sync + 'a>(&mut self, call: T) {
        self.calls.push(Box::new(call));
    }
}

impl Call for MultiCall<'_> {
    fn method(&self) -> &'static str {
        "system.multicall"
    }
    fn serialize_params(
        &self,
        serializer: &mut SerializeSeq,
        token: Option<&str>,
    ) -> Result<(), JsonError> {
        #[derive(serde::Serialize)]
        struct MultiCallParam {
            #[serde(rename = "methodName")]
            method_name: &'static str,
            params: Value,
        }

        let mut values = SmallVec::with_capacity(self.calls.len());
        for call in &self.calls {
            let param = MultiCallParam {
                method_name: call.method(),
                params: call.to_param(token)?,
            };
            values.push(param);
        }
        serializer.serialize_element(&values)?;
        Ok(())
    }
}
