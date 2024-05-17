use regex_lite::Regex;
use serde::{Deserialize, Serialize};
use squid_db::Attributes;

lazy_static! {
    static ref EXPIRE_AT: Regex = Regex::new(r"expire_at:(\d+)").unwrap();
}

/// Text representation in the database.
#[derive(Serialize, Deserialize, PartialEq, Default, Debug, Clone)]
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
    /// Additional data associated with the entity.
    ///
    /// Accepted metatag:
    /// - `expire_at:<u64>` as TTL. 0 means infinite.
    /// - `tag:<String>` to specify a field for the sentence.
    ///
    /// # Examples
    /// `expire_at:0,tag:politic`,
    /// `expire_at:1714240000,tag:sport`
    pub meta: String,
}

impl Attributes for Entity {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn ttl(&self) -> Option<u64> {
        EXPIRE_AT
            .captures(&self.meta)
            .and_then(|capture| capture.get(1))
            .map(|expire| expire.as_str().parse().unwrap_or_default())
    }
}
