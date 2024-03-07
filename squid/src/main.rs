#![forbid(unsafe_code)]

mod helpers;
mod models;

use squid::{
    squid_server::{Squid, SquidServer},
    {AddRequest, LeaderboardRequest, Ranking, Void, Word},
};
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::{transport::Server, Request, Response, Status};

pub mod squid {
    tonic::include_proto!("squid");
}

struct SuperSquid {
    algorithm: helpers::database::Algorithm,
}

#[tonic::async_trait]
impl Squid for SuperSquid {
    async fn leaderboard(
        &self,
        request: Request<LeaderboardRequest>,
    ) -> Result<Response<Ranking>, Status> {
        Ok(Response::new(Ranking {
            word: Some(Word {
                word: "test".to_string(),
                occurence: 0,
            }),
        }))
    }

    async fn add(
        &self,
        request: Request<AddRequest>,
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
    let mut db_instance: squid_db::Instance<models::database::Entity> =
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

    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();

    let rank = algo.rank(3);

    println!("Rank of the 3 most used words: {:?}", rank);

    println!(
        "Took {}μs",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros()
            - start
    );
    
    let addr = format!("0.0.0.0:{}", config.port.unwrap_or(50051))
        .parse()
        .unwrap();

    log::info!("Server started on {}", addr);

    Server::builder()
        .add_service(SquidServer::new(SuperSquid {
            algorithm: helpers::database::Algorithm::Map(algo),
        }))
        .serve(addr)
        .await
        .unwrap();
}
