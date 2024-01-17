use std::{collections::HashMap, sync::Arc, time::Duration};

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use parking_lot::Mutex;
use serde::Serialize;
use serde_json::Value;
use tokio::{
    select,
    sync::{mpsc, oneshot},
    time::timeout,
};

use crate::{
    call::{MultiCall, MultiResponse},
    Call, Error, Reply, Result, RpcError, SmallString,
};

type WSMessgae = tokio_tungstenite::tungstenite::Message;
type WSStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

pub trait NotificationCallback {
    fn on_notification(&self, method: SmallString, gid: SmallString);
}

struct EmptyCallback;

impl NotificationCallback for EmptyCallback {
    fn on_notification(&self, _method: SmallString, _gid: SmallString) {}
}

struct RawClient {
    // Sender for request
    message_tx: mpsc::Sender<RpcRequest>,
    // Receiver for shutdown signal
    _shutdown_rx: oneshot::Receiver<()>,
    token: Option<String>,
}

struct RpcRequest {
    method: &'static str,
    params: Value,
    handler: oneshot::Sender<RpcResponse>,
}

enum RpcResponse {
    Success(Value),
    Error(RpcError),
}

impl RpcResponse {
    #[inline]
    fn into_result(self) -> Result<Value> {
        match self {
            Self::Success(v) => Ok(v),
            Self::Error(e) => Err(Error::Rpc(e)),
        }
    }
}

pub struct ConnectionMeta {
    pub url: String,
    pub token: Option<String>,
}

impl RawClient {
    async fn connect<CB: NotificationCallback + Send + Sync + 'static>(
        meta: ConnectionMeta,
        channel_buffer_size: usize,
        notification_cb: CB,
    ) -> Result<Self> {
        let (ws, _) = tokio_tungstenite::connect_async(&meta.url).await?;
        let (message_tx, message_rx) = mpsc::channel(channel_buffer_size);
        let (shutdown_tx, _shutdown_rx) = oneshot::channel();
        tokio::spawn(async move {
            Self::background(ws, meta.url, message_rx, shutdown_tx, notification_cb).await
        });

        let client = Self {
            message_tx,
            _shutdown_rx,
            token: meta.token,
        };
        Ok(client)
    }

    async fn background<CB: NotificationCallback>(
        ws: WSStream,
        url: String,
        mut message_rx: mpsc::Receiver<RpcRequest>,
        mut shutdown_tx: oneshot::Sender<()>,
        cb: CB,
    ) {
        let mut handler_mapping = HashMap::new();
        let (mut sink, mut stream) = ws.split();
        // loop to handle message and reconnect
        loop {
            // loop to handle message(recv, send)
            loop {
                select! {
                    _ = shutdown_tx.closed() => {
                        tracing::info!("background task shutdown");
                        return;
                    }
                    Some(msg) = message_rx.recv() => {
                        let uuid = uuid::Uuid::new_v4().to_string();
                        handler_mapping.insert(uuid.clone(), msg.handler);
                        if let Err(e) = timeout(Duration::from_secs(10), Self::send_request(&mut sink, &uuid, msg.method, &msg.params)).await {
                            tracing::error!("send request error: {e}");
                            break;
                        }
                    }
                    Some(msg) = stream.next() => {
                        let text = match msg {
                            Ok(WSMessgae::Text(text)) => text,
                            Ok(_) => {
                                continue;
                            }
                            Err(e) => {
                                tracing::error!("websocket error: {e}");
                                break;
                            }
                        };
                        Self::handle_response(text, &mut handler_mapping, &cb);
                    }
                }
            }

            // loop to reconnect.
            handler_mapping.clear();
            loop {
                if shutdown_tx.is_closed() {
                    tracing::info!("background task shutdown");
                    return;
                }
                match timeout(
                    Duration::from_secs(10),
                    tokio_tungstenite::connect_async(&url),
                )
                .await
                {
                    Err(e) => {
                        tracing::error!("reconnect error: {e}, will retry in 10 seconds");
                        tokio::time::sleep(Duration::from_secs(10)).await;
                    }
                    Ok(Err(e)) => {
                        tracing::error!("reconnect timeout: {e}, will retry in 10 seconds");
                        tokio::time::sleep(Duration::from_secs(10)).await;
                    }
                    Ok(Ok((ws, _))) => {
                        let (new_sink, new_stream) = ws.split();
                        sink = new_sink;
                        stream = new_stream;
                        break;
                    }
                }
            }
        }
    }

    async fn send_request(
        sink: &mut SplitSink<WSStream, WSMessgae>,
        uuid: &str,
        method: &str,
        call: &Value,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Request<'a> {
            id: &'a str,
            jsonrpc: &'a str,
            method: &'a str,
            params: &'a Value,
        }

        let rpc_req = Request {
            id: uuid,
            jsonrpc: "2.0",
            method,
            params: call,
        };
        sink.send(WSMessgae::Text(serde_json::to_string(&rpc_req)?))
            .await?;
        Ok(())
    }

    fn handle_response<CB: NotificationCallback>(
        text: String,
        handler_mapping: &mut HashMap<String, oneshot::Sender<RpcResponse>>,
        cb: &CB,
    ) {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Response {
            Resp {
                id: String,
                result: Value,
            },
            Notification {
                method: SmallString,
                params: NotificationParam,
            },
            Err {
                id: String,
                error: RpcError,
            },
        }
        #[derive(serde::Deserialize)]
        struct NotificationParam {
            gid: SmallString,
        }

        match serde_json::from_str::<Response>(&text) {
            Ok(Response::Resp { id, result }) => {
                if let Some(handler) = handler_mapping.remove(&id) {
                    let _ = handler.send(RpcResponse::Success(result));
                }
            }
            Ok(Response::Notification { method, params }) => {
                cb.on_notification(method, params.gid);
            }
            Ok(Response::Err { id, error }) => {
                if let Some(handler) = handler_mapping.remove(&id) {
                    let _ = handler.send(RpcResponse::Error(error));
                }
            }
            Err(e) => {
                tracing::error!("parse response error: {e}, origin text: {text}");
            }
        }
    }

    pub async fn call_value<'a, C: Call>(&'a self, call: &'a C) -> Result<Value> {
        let (tx, rx) = oneshot::channel();
        let token = self.token.as_ref().map(AsRef::as_ref);
        let method = call.method();
        let params = call.to_param(token)?;
        self.message_tx
            .send(RpcRequest {
                method,
                params,
                handler: tx,
            })
            .await
            .map_err(|_| Error::ChannelSend)?;
        let rpc_resp = rx.await.map_err(Error::from)?;
        rpc_resp.into_result()
    }

    pub async fn call<'a, C: Call + Reply>(&'a self, call: &'a C) -> Result<C::Reply> {
        let value = self.call_value(call).await?;
        let reply = C::to_reply(value).map_err(Error::from)?;
        Ok(reply)
    }
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<RawClient>,
}

impl Client {
    pub async fn connect(meta: ConnectionMeta, channel_buffer_size: usize) -> Result<Self> {
        let inner = Arc::new(RawClient::connect(meta, channel_buffer_size, EmptyCallback).await?);
        Ok(Self { inner })
    }

    pub async fn connect_with_cb<CB: NotificationCallback + Send + Sync + 'static>(
        meta: ConnectionMeta,
        channel_buffer_size: usize,
        notification_cb: CB,
    ) -> Result<Self> {
        let inner = Arc::new(RawClient::connect(meta, channel_buffer_size, notification_cb).await?);
        Ok(Self { inner })
    }

    pub async fn call_value<'a, C: Call>(&'a self, call: &'a C) -> Result<Value> {
        self.inner.call_value(call).await
    }

    pub async fn call<'a, C: Call + Reply>(&'a self, call: &'a C) -> Result<C::Reply> {
        self.inner.call(call).await
    }
}

struct CallWithRecv {
    call: Box<dyn Call + Send + Sync + 'static>,
    tx: oneshot::Sender<Result<Value>>,
}

#[derive(Clone)]
pub struct BatchClient {
    batch: Arc<Mutex<Vec<CallWithRecv>>>,

    inner: Arc<RawClient>,
    _shutdown_rx: Arc<oneshot::Receiver<()>>,
}

impl BatchClient {
    pub async fn connect(
        meta: ConnectionMeta,
        channel_buffer_size: usize,
        interval: Duration,
    ) -> Result<Self> {
        Self::connect_with_cb(meta, channel_buffer_size, interval, EmptyCallback).await
    }

    pub async fn connect_with_cb<CB: NotificationCallback + Send + Sync + 'static>(
        meta: ConnectionMeta,
        channel_buffer_size: usize,
        interval: Duration,
        notification_cb: CB,
    ) -> Result<Self> {
        let raw = Arc::new(RawClient::connect(meta, channel_buffer_size, notification_cb).await?);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let batch = Arc::new(Mutex::new(Vec::new()));

        let raw_clone = raw.clone();
        let batch_clone = batch.clone();
        tokio::spawn(async move {
            Self::background(raw_clone, batch_clone, interval, shutdown_tx).await
        });
        Ok(Self {
            batch,
            inner: raw,
            _shutdown_rx: Arc::new(shutdown_rx),
        })
    }

    pub async fn call_instantly<'a, C: Call + Reply>(&'a self, call: &'a C) -> Result<C::Reply> {
        let value = self.inner.call_value(call).await?;
        let reply = C::to_reply(value).map_err(Error::from)?;
        Ok(reply)
    }

    pub async fn call_value_instantly<'a, C: Call>(&'a self, call: &'a C) -> Result<Value> {
        self.inner.call_value(call).await
    }

    pub async fn call<C: Call + Reply + Send + Sync + 'static>(&self, call: C) -> Result<C::Reply> {
        let value = self.call_value(call).await?;
        let reply = C::to_reply(value).map_err(Error::from)?;
        Ok(reply)
    }

    pub async fn call_value<C: Call + Send + Sync + 'static>(&self, call: C) -> Result<Value> {
        let (tx, rx) = oneshot::channel();
        self.batch.lock().push(CallWithRecv {
            call: Box::new(call),
            tx,
        });
        let value = rx.await.map_err(Error::from)??;
        Ok(value)
    }

    async fn background(
        client: Arc<RawClient>,
        batch: Arc<Mutex<Vec<CallWithRecv>>>,
        interval: Duration,
        mut shutdown_tx: oneshot::Sender<()>,
    ) {
        let mut call_buffer = Vec::new();
        let mut tx_buffer = Vec::new();
        loop {
            // try send batch call
            {
                let mut batch = batch.lock();
                if !batch.is_empty() {
                    for CallWithRecv { call, tx } in batch.drain(..) {
                        call_buffer.push(call);
                        tx_buffer.push(tx);
                    }
                }
            }

            if !call_buffer.is_empty() {
                tracing::debug!("batch call: len = {len}", len = call_buffer.len());
                let mut multi_call = MultiCall {
                    calls: std::mem::take(&mut call_buffer),
                };
                match client.call_value(&multi_call).await.and_then(|value| {
                    serde_json::from_value::<MultiResponse>(value).map_err(crate::Error::Decode)
                }) {
                    Ok(values) => {
                        for (value, tx) in values.0.into_iter().zip(tx_buffer.drain(..)) {
                            let _ = tx.send(Ok(value));
                        }
                    }
                    Err(e) => {
                        tracing::error!("batch call error: {e}");
                    }
                };
                call_buffer = std::mem::take(&mut multi_call.calls);
                call_buffer.clear();
                tx_buffer.clear();
            }

            // wait
            select! {
                _ = shutdown_tx.closed() => {
                    tracing::info!("background task shutdown");
                    return;
                }
                _ = tokio::time::sleep(interval) => {}
            }
        }
    }
}
