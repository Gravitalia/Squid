use std::collections::HashMap;

/// Structure containing the data required by the HashMap algorithm.
#[derive(Debug)]
pub struct MapAlgorithm {
    /// Data from the HashMap.
    data: HashMap<String, usize>,
}

impl MapAlgorithm {
    pub fn new<T>(&mut self, key: T, value: usize)
    where
        T: ToString,
    {
        self.data.insert(key.to_string(), value);
    }
}
