use tonic::{transport::Server, Request, Response, Status};

use squid::squid_server::{Squid, SquidServer};
use squid::{SquidIndexRequest, SquidIndexReply, SquidGetRequest, SquidGetReply};

mod helpers;
mod config;
mod solr;

pub mod squid {
    tonic::include_proto!("squid");
}
pub struct Builder {}

#[tonic::async_trait]
impl Squid for Builder {
    async fn squid_index(
        &self,
        _request: Request<SquidIndexRequest>,
    ) -> Result<Response<SquidIndexReply>, Status> {
        let reply = SquidIndexReply {
            message: "test".to_string(),
            error: false
        };
        Ok(Response::new(reply))
    }

    async fn squid_get(
        &self,
        _request: Request<SquidGetRequest>,
    ) -> Result<Response<SquidGetReply>, Status> {
        let reply = SquidGetReply {
            items: [].to_vec()
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::read();

    let jetty_home = solr::properties().await?.system_properties.jetty_home;
    helpers::copy_dir("./configset", &format!("{}/solr/configsets/squid/conf", jetty_home))?;
    std::thread::sleep(std::time::Duration::from_secs(1));

    for service in config.services {
        solr::core::create(service.name).await?;
    }

    let addr = "[::1]:50051".parse().unwrap();
    println!("Server listening on {}", addr);
    Server::builder()
        .add_service(SquidServer::new(Builder {}))
        .serve(addr)
        .await?;

    Ok(())
}
