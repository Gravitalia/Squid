mod helpers;

fn main() {
    // Set logger with Fern.
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                helpers::format::format_rfc3339(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs()
                ),
                record.level(),
                message
            ))
        })
        .level(if cfg!(debug_assertions) {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Info
        })
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let port = std::env::var("port").unwrap_or_else(|_| "5555".to_string());
    let responder = zmq::Context::new().socket(zmq::REP).unwrap();

    match responder.bind(&format!("tcp://*:{}", port)) {
        Ok(_) => log::info!("Started Squid server on port {}", port),
        Err(error) => log::error!("Failed starting Squid: {}", error),
    }

    let mut msg = zmq::Message::new();
    loop {
        responder.recv(&mut msg, 0).unwrap();
        println!("Received {}", msg.as_str().unwrap());
        responder.send("World", 0).unwrap();
    }
}
