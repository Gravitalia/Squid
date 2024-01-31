use anyhow::Result;
use std::collections::HashSet;

/// Lowercase words, remove punctuation, separate words into tokens and convert them into numbers.
pub fn tokenize<T: ToString>(text: T) -> Result<String> {
    let punctuation: HashSet<char> =
        ['!', ',', '.', ':', ';', '?', '-', '\'', '\"', '(', ')']
            .iter()
            .cloned()
            .collect();

    let result_string: String = text
        .to_string()
        .to_lowercase()
        .chars()
        .filter(|c| !punctuation.contains(c))
        .collect();

    Ok(result_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let plaintext =
            "I really like apples! But I prefer Gravitalia, sometimes... yeah?";

        assert_eq!(
            tokenize(plaintext).unwrap(),
            "i really like apples but i prefer gravitalia sometimes yeah"
        )
    }
}
