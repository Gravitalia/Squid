use anyhow::Result;
use std::collections::HashSet;

/// Lowercase words, remove punctuation, separate words into tokens and convert them into numbers.
pub fn tokenize<T: ToString>(text: T) -> Result<String> {
    let punctuation: HashSet<char> =
        ['!', ',', '.', ':', ';', '?', '-', '\"', '(', ')']
            .iter()
            .cloned()
            .collect();

    let result_string: String = text
        .to_string()
        .replace('\'', " ")
        .to_lowercase()
        .chars()
        .filter(|c| !punctuation.contains(c))
        .collect::<String>()
        .split_ascii_whitespace()
        .filter(|c| *c != " " && c.len() > 1)
        .map(|c| format!("{} ", c))
        .collect();

    Ok(result_string.trim_end().to_string())
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
            "really like apples but prefer gravitalia sometimes yeah"
        )
    }
}
