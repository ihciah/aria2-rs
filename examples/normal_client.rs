use aria2_rs::{
    call::{AddUriCall, MultiCall},
    Client, ConnectionMeta, SmallVec,
};

const WS_RPC_ADDRESS: &str = "wss://TEST/jsonrpc";
const TOKEN: &str = "token:TEST";

#[tokio::main]
async fn main() {
    let client = Client::connect(
        ConnectionMeta {
            url: WS_RPC_ADDRESS.to_string(),
            token: Some(TOKEN.to_string()),
        },
        10,
    )
    .await
    .unwrap();
    let r = client
        .call(&AddUriCall {
            uris: SmallVec::from_iter(["http://example.org/file".to_string()]),
            options: None,
        })
        .await
        .unwrap();
    println!("response: {r:?}");

    let add_uri = AddUriCall {
        uris: SmallVec::from_iter(["http://example.org/file".to_string()]),
        options: None,
    };
    let mut multi = MultiCall::new();
    multi.push(add_uri);
    let r = client.call(&multi).await.unwrap();
    println!("response: {r:?}");
}
