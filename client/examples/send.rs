use client::WebserverClient;
use contracts::{server::SleepParams, JsonRpcRequest, Method};

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().collect();

    let url = args[1].clone();
    let key_name = args[2].clone();
    let key_value = args[3].clone();

    let client = WebserverClient::new(url, key_name, key_value)
        .build()
        .unwrap();

    let request = JsonRpcRequest::new(
        Method::Sleep.to_string(),
        SleepParams::new(1000).unwrap(),
        Some("test".to_string()),
    );

    let response = client.send_request(request).await.unwrap();

    println!("response: '{:?}'", response);
}
