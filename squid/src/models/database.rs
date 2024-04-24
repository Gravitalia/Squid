use serde::{Deserialize, Serialize};
use squid_db::Attributes;

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
    pub meta: String,
}

impl Attributes for Entity {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn ttl(&self) -> Option<u64> {
        Some(
            self.meta.split("expire_at:").collect::<Vec<&str>>()[1]
                .parse()
                .unwrap_or_default(),
        )
    }
}
