//! filters unnecessary words and removes it from sentences.

use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::OnceLock,
};

static STOP_WORDS: OnceLock<Vec<String>> = OnceLock::new();

/// Inits `STOP_WORDS` by adding every lines from a text file
/// to the cache.
pub(crate) fn init(path: PathBuf) {
    STOP_WORDS.get_or_init(|| {
        if let Ok(file) = OpenOptions::new().read(true).open(path) {
            let reader = BufReader::new(&file);

            let mut words: Vec<String> = vec![];
            for word in reader.lines().map_while(Result::ok) {
                words.push(word)
            }

            words
        } else {
            Vec::default()
        }
    });
}

/// Removes every stop words from a sentence.
///
/// # Example
/// ```rust
/// use std::{fs::File, io::prelude::*};
/// use squid_tokenizer::stopwords::remove_words_from_sentence;
///
/// let mut buffer: Vec<u8> = vec![];
/// buffer.extend_from_slice(b"ich");
/// buffer.extend_from_slice(b"\n");
///
/// buffer.extend_from_slice(b"bin");
/// buffer.extend_from_slice(b"\n");
///
/// let mut file = File::create("./stopwords.txt").unwrap();
/// file.write_all(&buffer).unwrap();
///
/// let sentence = "ich bin Hans".to_string();
/// assert_eq!(remove_words_from_sentence(sentence), "Hans".to_string());
/// ```
pub fn remove_words_from_sentence(sentence: String) -> String {
    let stop_words = STOP_WORDS.get_or_init(Vec::default);

    sentence
        .split_whitespace()
        .filter(|word| !stop_words.contains(&word.to_lowercase()))
        .collect::<Vec<&str>>()
        .join(" ")
}
