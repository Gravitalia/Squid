#![forbid(unsafe_code)]

mod helpers;
mod models;

use squid::{
    squid_server::{Squid, SquidServer},
    {AddRequest, LeaderboardRequest, Ranking, Void, Word},
};
use squid_tokenizer::tokenize;
use std::sync::{Arc, RwLock};
use tonic::{transport::Server, Request, Response, Status};

pub mod squid {
    tonic::include_proto!("squid");
}

struct SuperSquid {
    algorithm: helpers::database::Algorithm,
    instance: Arc<RwLock<squid_db::Instance<models::database::Entity>>>,
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
        request: Request<AddRequest>,
    ) -> Result<Response<Void>, Status> {
        let data = request.into_inner();

        helpers::database::set(
            Arc::clone(&self.instance),
            self.algorithm.clone(),
            models::database::Entity {
                id: String::default(),
                original_text: None,
                post_processing_text: tokenize(&data.sentence).map_err(
                    |error| {
                        log::error!(
                            "Failed to tokenize {:?}: {}",
                            data.sentence,
                            error
                        );
                        Status::invalid_argument("failed to tokenize sentence")
                    },
                )?,
                lang: "fr".to_string(),
                ttl: data.lifetime as usize,
                creation_date: 0,
            },
        )
        .unwrap();

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
            instance: Arc::new(RwLock::new(db_instance)),
        }))
        .serve(addr)
        .await
        .unwrap();
}
