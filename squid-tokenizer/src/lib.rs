pub mod stopwords;

use std::{collections::HashSet, convert::Infallible, path::Path};

/// Lowercase words, remove punctuation, separate words into tokens and convert them into numbers.
pub fn tokenize<T: ToString>(text: T) -> Result<String, Infallible> {
    stopwords::init(Path::new("./stopwords").to_path_buf());

    let punctuation: HashSet<char> = ['!', ',', '.', ':', ';', '?', '-', '\"', '(', ')']
        .iter()
        .cloned()
        .collect();

    let result_string: String = stopwords::remove_words_from_sentence(
        text.to_string()
            .replace('\'', " ")
            .to_lowercase()
            .chars()
            .filter(|c| !punctuation.contains(c))
            .collect::<String>()
            .split_ascii_whitespace()
            .filter(|c| *c != " " && c.len() > 1)
            .map(|c| format!("{} ", c))
            .collect(),
    );

    let normalize = result_string
        .chars()
        .map(|c| {
            if c.len_utf8() > 1 {
                c.escape_unicode().to_string()
            } else {
                c.to_string()
            }
        })
        .collect::<String>();

    Ok(normalize.trim_end().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let plaintext = "I really like apples! But I prefer Gravitalia, sometimes... yeah?";

        assert_eq!(
            tokenize(plaintext).unwrap(),
            "really like apples but prefer gravitalia sometimes yeah"
        )
    }
}
