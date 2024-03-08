use crate::models::database::Entity;
use anyhow::Result;
use squid_algorithm::hashtable::MapAlgorithm;
use squid_db::Instance;

/// The algorithms managed by Squid.
#[derive(Debug)]
pub enum Algorithm {
    Map(MapAlgorithm),
}

impl From<MapAlgorithm> for Algorithm {
    /// Implements conversion from a MapAlgorithm to Algorithm.
    fn from(map: MapAlgorithm) -> Self {
        Algorithm::Map(map)
    }
}

/// Adds a value to the database and the algorithm.
pub fn set<A: Into<Algorithm>>(
    instance: &mut Instance<Entity>,
    algorithm: A,
    value: Entity,
) -> Result<()> {
    instance.set(value.clone())?;

    match algorithm.into() {
        Algorithm::Map(mut implementation) => {
            for str in value.post_processing_text.split_whitespace() {
                implementation.set(str);
            }
        },
    }

    Ok(())
}

/// Rank the most used words.
pub fn rank<A: Into<Algorithm>>(
    instance: &mut Instance<Entity>,
    algorithm: A,
    length: usize,
) -> Result<Vec<String>> {
    match algorithm.into() {
        Algorithm::Map(implementation) => Ok(implementation.rank(length)),
    }
}
