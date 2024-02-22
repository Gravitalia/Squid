use serde::{Deserialize, Serialize};

/// Text representation in the database.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Entity {
    /// Unique identifier of the text.
    pub id: String,
    /// Original text without tokenizer processing.
    /// If set to null, it will not be possible to modify the processing configuration.
    pub original_text: Option<String>,
    /// Text after tokenization, lemmatization and processing.
    pub post_processing_text: String,
    /// The language in which the text is written.
    pub lang: String,
    /// Text lifetime in seconds.
    /// After this time it will no longer be taken into account in calculations
    /// and will be deleted.
    ///
    /// Set to 0 for infinite.
    pub ttl: usize,
    /// Timestamp in seconds since the 1st January, 1970.
    pub creation_date: usize,
}
