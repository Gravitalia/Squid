use serde::Deserialize;

/// The data in the configuration file for setting up Squid.
#[derive(Deserialize, Debug)]
pub struct Config {
    pub port: Option<u16>,
    pub service: Service,
}

/// The algorithm used to rank the most frequently used words.
#[derive(Deserialize, Debug, Default)]
pub enum Algorithm {
    #[default]
    Hashmap,
}

/// Which words need to be selected to be classified.
#[derive(Deserialize, Debug, Default)]
pub enum MessageType {
    #[default]
    Anything,
    Word,
    Hashtag,
}

/// Definition of a service. A service is equal to a database.
#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct Service {
    /// Name of the database.
    name: String,
    /// The algorithm to be used.
    /// This affects RAM consumption and accuracy.
    #[serde(default)]
    pub algorithm: Algorithm,
    /// The maximum number of words returned for a query.
    max_words: Option<u8>,
    /// What data the algorithm needs to cache.
    #[serde(default)]
    pub message_type: MessageType,
    /// The language of words to be returned.
    lang: Option<String>,
    /// Words to exclude from the search.
    #[serde(default)]
    pub exclude: Vec<String>,
}
