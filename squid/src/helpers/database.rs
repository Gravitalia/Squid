use crate::models::database::Entity;
use squid_algorithm::hashtable::MapAlgorithm;
use squid_db::Instance;
use squid_error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

/// The algorithms managed by Squid.
#[derive(Debug, Clone)]
pub enum Algorithm {
    Map(Arc<RwLock<MapAlgorithm>>),
}

impl From<MapAlgorithm> for Algorithm {
    /// Implements conversion from a MapAlgorithm to Algorithm.
    fn from(map: MapAlgorithm) -> Self {
        Algorithm::Map(Arc::new(RwLock::new(map)))
    }
}

/// Adds a value to the database and the algorithm.
pub async fn set<A: Into<Algorithm>>(
    config: &crate::models::config::Config,
    instance: Arc<RwLock<Instance<Entity>>>,
    algorithm: A,
    value: Entity,
) -> Result<(), Error> {
    instance.write().await.set(value.clone())?;

    match algorithm.into() {
        Algorithm::Map(implementation) => {
            for str in value.post_processing_text.split_whitespace() {
                if !config.service.exclude.contains(&str.to_string()) {
                    match config.service.message_type {
                        crate::models::config::MessageType::Hashtag => {
                            if str.starts_with('#') {
                                implementation.write().await.set(str)
                            }
                        },
                        crate::models::config::MessageType::Word => {
                            if !str.starts_with('#') {
                                implementation.write().await.set(str)
                            }
                        },
                        _ => implementation.write().await.set(str),
                    }
                }
            }
        },
    }

    Ok(())
}

/// Removes a value to the algorithm.
pub async fn _remove<A: Into<Algorithm>>(
    algorithm: A,
    key: String,
) -> Result<(), Error> {
    match algorithm.into() {
        Algorithm::Map(implementation) => {
            implementation.write().await.remove(key)
        },
    }

    Ok(())
}

/// Rank the most used words.
pub async fn rank<A: Into<Algorithm>>(
    algorithm: A,
    length: usize,
) -> Vec<(String, usize)> {
    match algorithm.into() {
        Algorithm::Map(implementation) => {
            implementation.read().await.rank(length)
        },
    }
}
