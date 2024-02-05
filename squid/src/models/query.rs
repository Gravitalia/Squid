use smallvec::SmallVec;

/// Answer with a ranking of the most frequently used words.
#[derive(Debug, Serialize, Clone)]
pub struct Leaderboard {
    /// Lists of the most frequently found words.
    pub words: SmallVec<[Word; 5]>,
}

/// The word data to be returned.
#[derive(Debug, Serialize, Clone, Copy)]
pub struct Word {
    /// The word in letters.
    pub word: &'static str,
    /// Number of times the word appears in the recorded texts.
    pub occurrence: usize,
}
