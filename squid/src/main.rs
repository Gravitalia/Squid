#![forbid(unsafe_code)]

mod helpers;
mod models;

use squid::{
    squid_server::{Squid, SquidServer},
    {AddRequest, LeaderboardRequest, Ranking, Void, Word},
};
use squid_tokenizer::tokenize;
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::{transport::Server, Request, Response, Status};

pub mod squid {
    tonic::include_proto!("squid");
}

struct SuperSquid {
    algorithm: helpers::database::Algorithm,
    instance: squid_db::Instance<models::database::Entity>,
}

#[tonic::async_trait]
impl Squid for SuperSquid {
    async fn leaderboard(
        &self,
        request: Request<LeaderboardRequest>,
    ) -> Result<Response<Ranking>, Status> {
        Ok(Response::new(Ranking {
            word: helpers::database::rank(
                self.algorithm.clone(),
                request.into_inner().length as usize,
            )
            .iter()
            .map(|(word, occurence)| Word {
                word: word.to_string(),
                occurence: (*occurence).try_into().unwrap_or_default(),
            })
            .collect::<Vec<_>>(),
        }))
    }
    
    async fn add(
        &self,
        _request: Request<AddRequest>,
    ) -> Result<Response<Void>, Status> {
        Ok(Response::new(Void {}))
    }
}

#[tokio::main]
async fn main() {
    // Set logger with Fern.
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                helpers::format::format_rfc3339(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs()
                ),
                record.level(),
                message,
            ))
        })
        .level(if cfg!(debug_assertions) {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Info
        })
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let config = helpers::config::read();

    // Start database.
    let db_instance: squid_db::Instance<models::database::Entity> =
        squid_db::Instance::new().unwrap();
    log::info!(
        "Loaded instance with {} entities.",
        db_instance.data.0.len()
    );

    // Chose algorithm.
    let mut algo = squid_algorithm::hashtable::MapAlgorithm::default();

    for data in db_instance.data.0.clone() {
        for str in data.post_processing_text.split_whitespace() {
            algo.set(str)
        }
    }

    let addr = format!("0.0.0.0:{}", config.port.unwrap_or(50051))
        .parse()
        .unwrap();

    log::info!("Server started on {}", addr);

    Server::builder()
        .add_service(SquidServer::new(SuperSquid {
            algorithm: helpers::database::Algorithm::Map(algo),
            instance: db_instance,
        }))
        .serve(addr)
        .await
        .unwrap();
}
