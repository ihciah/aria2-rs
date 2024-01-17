use std::time::Duration;

use aria2_rs::{call::AddUriCall, BatchClient, ConnectionMeta, SmallVec};

const WS_RPC_ADDRESS: &str = "wss://TEST/jsonrpc";
const TOKEN: &str = "token:TEST";

#[tokio::main]
async fn main() {
    let client = BatchClient::connect(
        ConnectionMeta {
            url: WS_RPC_ADDRESS.to_string(),
            token: Some(TOKEN.to_string()),
        },
        10,
        Duration::from_secs(5),
    )
    .await
    .unwrap();

    // The following 3 requests will be sent in one batch(multicall).
    let r = tokio::join!(
        client.call(AddUriCall {
            uris: SmallVec::from_iter(["http://example.org/file".to_string()]),
            options: None,
        }),
        client.call(AddUriCall {
            uris: SmallVec::from_iter(["http://example.org/file".to_string()]),
            options: None,
        }),
        client.call(AddUriCall {
            uris: SmallVec::from_iter(["http://example.org/file".to_string()]),
            options: None,
        })
    );
    println!("response: {r:?}");
}
