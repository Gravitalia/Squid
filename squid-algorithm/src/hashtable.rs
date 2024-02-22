use std::collections::HashMap;

/// Structure containing the data required by the HashMap algorithm.
#[derive(Debug, Default, Clone)]
pub struct MapAlgorithm {
    /// Data from the HashMap.
    data: HashMap<String, usize>,
}

impl MapAlgorithm {
    /// Adds data to the data contained in the HashMap.
    pub fn set<T>(&mut self, key: T)
    where
        T: ToString,
    {
        if let Some(counter) = self.data.get_mut(&key.to_string()) {
            *counter += 1;
        } else {
            self.data.insert(key.to_string(), 1);
        }
    }

    /// Classify the most frequently used words.
    pub fn rank(&self, length: usize) -> Vec<String> {
        let mut sorted_word_counts: Vec<_> =
            self.data.clone().into_iter().collect();
        sorted_word_counts.sort_by(|a, b| b.1.cmp(&a.1));

        let most_used_words: Vec<_> = sorted_word_counts
            .iter()
            .take(length)
            .map(|(word, _count)| word.clone())
            .collect();

        most_used_words
    }
}
