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
}
