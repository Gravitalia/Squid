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

    println!(
        "Received {} in {}ms",
        msg.as_str().unwrap(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            - now
    );
}
