use tonic::{transport::Server, Request, Response, Status};

use squid::squid_server::{Squid, SquidServer};
use squid::{SquidIndexRequest, SquidIndexReply, SquidGetRequest, SquidGetReply};

pub mod squid {
    tonic::include_proto!("squid");
}
pub struct Builder {}

#[tonic::async_trait]
impl Squid for Builder {
    async fn squid_index(
        &self,
        request: Request<SquidIndexRequest>,
    ) -> Result<Response<SquidIndexReply>, Status> {
        let reply = SquidIndexReply {
            message: "test".to_string(),
            error: false
        };
        Ok(Response::new(reply))
    }

    async fn squid_get(
        &self,
        request: Request<SquidGetRequest>,
    ) -> Result<Response<SquidGetReply>, Status> {
        let reply = SquidGetReply {
            items: [].to_vec()
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();


    println!("Server listening on {}", addr);
    Server::builder()
        .add_service(SquidServer::new(Builder {}))
        .serve(addr)
        .await?;

    Ok(())
}
