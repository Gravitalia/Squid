use std::time::{SystemTime, UNIX_EPOCH};
use squid::squid_client::SquidClient;
use squid::SquidIndexRequest;

pub mod squid {
    tonic::include_proto!("squid");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SquidClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(SquidIndexRequest {
        message: "This a #Gravitalia test!".into()
    });

    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let response = client.squid_index(request).await?;
    println!("Response with {:?} in {}ms", response.into_inner().message, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()-start);

    Ok(())
}