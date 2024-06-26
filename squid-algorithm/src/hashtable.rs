use ahash::RandomState;
use rayon::prelude::*;
use std::collections::HashMap;

/// Structure containing the data required by the HashMap algorithm.
#[derive(Debug, Default, Clone)]
pub struct MapAlgorithm {
    /// Data from the HashMap.
    data: HashMap<String, usize, RandomState>,
}

impl MapAlgorithm {
    /// Adds data to the data contained in the HashMap.
    pub fn set<T>(&mut self, key: T)
    where
        T: ToString,
    {
        self.data
            .entry(key.to_string())
            .and_modify(|d| *d += 1)
            .or_insert(1);
    }

    /// Removes data from the data contained in the HashMap.
    pub fn remove<T>(&mut self, key: T)
    where
        T: ToString,
    {
        if let Some(count) = self.data.get_mut(&key.to_string()) {
            if *count > 1 {
                *count -= 1;
            } else {
                self.data.remove(&key.to_string());
            }
        }
    }

    /// Classify the most frequently used words.
    pub fn rank(&self, length: usize) -> Vec<(String, usize)> {
        let mut sorted_word_counts: Vec<_> =
            self.data.clone().into_iter().collect();
        sorted_word_counts.sort_by(|a, b| b.1.cmp(&a.1));

        let most_used_words: Vec<_> = sorted_word_counts
            .par_iter()
            .take(length)
            .map(|(word, count)| (word.clone(), *count))
            .collect();

        most_used_words
    }
}
