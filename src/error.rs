#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Encode error: {0}")]
    Encode(#[from] serde_json::Error),
    #[error("Decode error: {0}")]
    Decode(serde_json::Error),
    #[error("Websocket error {0}")]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Rpc error {0}")]
    Rpc(#[from] RpcError),
    #[error("Request send error")]
    ChannelSend,
    #[error("Response send error {0}")]
    ChannelRecv(#[from] tokio::sync::oneshot::error::RecvError),
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct RpcError {
    pub code: u32,
    pub message: String,
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RpcError: {{\"code\": {}, \"message\": \"{}\"}}",
            self.code, self.message
        )
    }
}
impl std::error::Error for RpcError {}
