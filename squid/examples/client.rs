use serde::Deserialize;
use std::io::Read;

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
    let requester = zmq::Context::new().socket(zmq::REQ).unwrap();
    assert!(requester.connect("tcp://localhost:5555").is_ok());

    let mut msg = zmq::Message::new();

    println!("Sending...");

    // Calculate time taken.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    // Send request and wait until response.
    requester.send("Hello", 0).unwrap();
    requester.recv(&mut msg, 0).unwrap();

    let data: Result<Vec<_>, _> = msg.bytes().collect();
    let result: Leaderboard = rmp_serde::from_slice(&data.unwrap()).unwrap();

    println!(
        "Received {:?} in {}ms",
        result,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            - now
    );
}
