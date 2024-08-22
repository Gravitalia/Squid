#![forbid(unsafe_code)]

mod helpers;
mod models;

#[macro_use]
extern crate lazy_static;

use crate::models::database::Entity;
use squid::{
    squid_server::{Squid, SquidServer},
    {AddRequest, LeaderboardRequest, Ranking, Void, Word},
};
use squid_tokenizer::tokenize;
use std::{
    ops::Add,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::signal;
use tokio::sync::{mpsc, RwLock};
use tonic::{transport::Server, Request, Response, Status};
use tracing::{error, info, Level};
use tracing_subscriber::fmt;

pub mod squid {
    tonic::include_proto!("squid");
}
struct SuperSquid {
    algorithm: helpers::database::Algorithm,
    config: models::config::Config,
    instance: Arc<RwLock<squid_db::Instance<models::database::Entity>>>,
}

const FLUSHTABLE_FLUSH_SIZE_KB: usize = 100; // wait 100kb on memtable before save it on disk.

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
            .await
            .iter()
            .map(|(word, occurence)| Word {
                word: word.to_string().replace("%20", " "),
                occurence: (*occurence).try_into().unwrap_or_default(),
            })
            .collect::<Vec<_>>(),
        }))
    }

    async fn add(&self, request: Request<AddRequest>) -> Result<Response<Void>, Status> {
        let data = request.into_inner();

        helpers::database::set(
            &self.config,
            Arc::clone(&self.instance),
            self.algorithm.clone(),
            models::database::Entity {
                id: uuid::Uuid::new_v4().to_string(),
                original_text: None,
                post_processing_text: tokenize(&data.sentence).map_err(|error| {
                    error!("Failed to tokenize {:?}: {}", data.sentence, error);
                    Status::invalid_argument("failed to tokenize sentence")
                })?,
                lang: "fr".to_string(),
                meta: if data.lifetime == 0 {
                    String::default()
                } else {
                    format!(
                        "expire_at:{}",
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .add(Duration::from_secs(data.lifetime))
                            .as_secs()
                    )
                },
            },
        )
        .await
        .unwrap();

        Ok(Response::new(Void {}))
    }
}

#[tokio::main]
async fn main() {
    #[cfg(not(debug_assertions))]
    fmt()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_max_level(Level::INFO)
        .init();

    #[cfg(debug_assertions)]
    fmt()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_max_level(Level::TRACE)
        .init();

    let config = helpers::config::read();

    // Set producer channel to receive expired sentences.
    let (tx, mut rx) = mpsc::channel::<Entity>(2305843009213693951);

    // Start database.
    let instance = squid_db::Builder::default()
        .memtable_flush_size(FLUSHTABLE_FLUSH_SIZE_KB)
        .mpsc_sender(tx)
        .with_ttl()
        .build()
        .await
        .unwrap();
    info!(
        "Loaded instance with {} entities.",
        instance.read().await.entries.len()
    );

    // Chose algorithm.
    let algo = Arc::new(RwLock::new(match config.service.algorithm {
        models::config::Algorithm::Hashmap => squid_algorithm::hashtable::MapAlgorithm::default(),
    }));

    // Init MPSC consumer.
    let ttl_algo = Arc::clone(&algo);
    tokio::task::spawn(async move {
        while let Some(data) = rx.recv().await {
            for word in data.post_processing_text.split_ascii_whitespace() {
                ttl_algo.write().await.remove(word);
            }
        }
    });

    // Add each words to algorithm.
    for data in &instance.read().await.entries {
        for str in data.post_processing_text.split_whitespace() {
            if !config.service.exclude.contains(&str.to_string()) {
                match config.service.message_type {
                    models::config::MessageType::Hashtag => {
                        if str.starts_with('#') {
                            algo.write().await.set(str)
                        }
                    }
                    models::config::MessageType::Word => {
                        if !str.starts_with('#') {
                            algo.write().await.set(str)
                        }
                    }
                    _ => algo.write().await.set(str),
                }
            }
        }
    }

    // Waiting for CTRL+C to save memtable.
    let ctrlc_instance = Arc::clone(&instance);
    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl+c event");
        if FLUSHTABLE_FLUSH_SIZE_KB > 0 {
            info!("Flushing memtable...");
            if let Err(err) = ctrlc_instance.write().await.flush() {
                error!("Some data haven't been flushed from memtable: {}", err);
            }
        }
        info!("Closing Squid server...");
        std::process::exit(0);
    });

    let addr = format!("0.0.0.0:{}", config.port.unwrap_or(50051))
        .parse()
        .unwrap();

    info!("Server started on {}", addr);

    // Remove entires to reduce ram usage.
    instance.write().await.entries.clear();

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
