use tonic::{transport::Server, Request, Response, Status};

use squid::squid_server::{Squid, SquidServer};
use squid::{SquidReply, SquidRequest, SquidGetRequest, SquidMessage};

use hyper::{Client, client::{HttpConnector, connect::dns::GaiResolver}, Body};
use hyper_tls::HttpsConnector;

pub mod squid {
    tonic::include_proto!("squid");
}

pub struct HashTag {
    client: Client<hyper_tls::HttpsConnector<HttpConnector<GaiResolver>>, Body>
}

#[tonic::async_trait]
impl Squid for HashTag {
    async fn squid_index(
        &self,
        request: Request<SquidRequest>,
    ) -> Result<Response<SquidMessage>, Status> {
        let reply = SquidMessage {
            message: "test".to_string(),
            error: false
        };
        Ok(Response::new(reply))
    }

    async fn squid_get(
        &self,
        request: Request<SquidGetRequest>,
    ) -> Result<Response<SquidReply>, Status> {
        let reply = SquidReply {
            items: []
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();

    let mut already_crawled: Vec<String> = [].to_vec();

    println!("Server listening on {}", addr);
    Server::builder()
        .add_service(SquidServer::new(HashTag { client: Client::builder().build::<_, hyper::Body>(HttpsConnector::new()) }))
        .serve(addr)
        .await?;

    Ok(())
}
