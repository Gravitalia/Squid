use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Leaderboard {
    pub words: Vec<Word>,
}

#[derive(Debug, Deserialize)]
pub struct Word {
    pub word: String,
    pub occurrence: usize,
}

fn main() {
    // Calculate time taken.
    let now: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    println!(
        "Received in {}ms",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            - now
    );
}
