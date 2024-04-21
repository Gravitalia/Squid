#![forbid(unsafe_code)]

mod helpers;
mod models;

use squid::{
    squid_server::{Squid, SquidServer},
    {AddRequest, LeaderboardRequest, Ranking, Void, Word},
};
use squid_tokenizer::tokenize;
use std::{
    ops::Add,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tonic::{transport::Server, Request, Response, Status};

pub mod squid {
    tonic::include_proto!("squid");
}
struct SuperSquid {
    algorithm: helpers::database::Algorithm,
    config: models::config::Config,
    instance: Arc<RwLock<squid_db::Instance<models::database::Entity>>>,
}

const FLUSHTABLE_FLUSH_SIZE_KB: usize = 100; // 100kB.

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
            &self.config,
            Arc::clone(&self.instance),
            self.algorithm.clone(),
            models::database::Entity {
                id: uuid::Uuid::new_v4().to_string(),
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
                meta: format!(
                    "expire_at:{}",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .add(Duration::from_secs(data.lifetime))
                        .as_secs()
                ),
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
        squid_db::Instance::new(FLUSHTABLE_FLUSH_SIZE_KB).unwrap();
    log::info!(
        "Loaded instance with {} entities.",
        db_instance.entries.len()
    );

    // Chose algorithm.
    let mut algo = match config.service.algorithm {
        models::config::Algorithm::Hashmap => {
            squid_algorithm::hashtable::MapAlgorithm::default()
        },
    };

    for data in &db_instance.entries {
        for str in data.post_processing_text.split_whitespace() {
            if !config.service.exclude.contains(&str.to_string()) {
                match config.service.message_type {
                    models::config::MessageType::Hashtag => {
                        if str.starts_with('#') {
                            algo.set(str)
                        }
                    },
                    models::config::MessageType::Word => {
                        if !str.starts_with('#') {
                            algo.set(str)
                        }
                    },
                    _ => algo.set(str),
                }
            }
        }
    }

    // Init TTL.
    let instance = db_instance.start_ttl();

    // Remove entires to reduce ram usage.
    instance.write().unwrap().entries.clear();

    let ctrlc_instance = Arc::clone(&instance);
    ctrlc::set_handler(move || {
        if FLUSHTABLE_FLUSH_SIZE_KB > 0 {
            log::info!("Flush memtable...");
            if let Err(err) = ctrlc_instance.write().unwrap().flush() {
                log::error!(
                    "Some data haven't been flushed from memtable: {}",
                    err
                );
            }
        }

        std::process::exit(0);
    })
    .expect("Failed to set Ctrl+C handler");

    let addr = format!("0.0.0.0:{}", config.port.unwrap_or(50051))
        .parse()
        .unwrap();

    log::info!("Server started on {}", addr);

    Server::builder()
        .add_service(SquidServer::new(SuperSquid {
            algorithm: helpers::database::Algorithm::Map(algo),
            config,
            instance,
        }))
        .serve(addr)
        .await
        .unwrap();
}
