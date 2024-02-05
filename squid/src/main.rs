#![forbid(unsafe_code)]

mod helpers;
mod models;

#[macro_use]
extern crate serde_derive;
use rmp_serde::Serializer;
use serde::Serialize;
use smallvec::smallvec;

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
        let mut buf = Vec::new();
        let val = models::query::Leaderboard {
            words: smallvec![
                models::query::Word {
                    word: "hinome",
                    occurrence: 0,
                },
                models::query::Word {
                    word: "#gravitalia",
                    occurrence: 0,
                },
            ],
        };
        val.serialize(&mut Serializer::new(&mut buf)).unwrap();

        println!("{:?}", buf);

        responder.recv(&mut msg, 0).unwrap();
        println!("Received {}", msg.as_str().unwrap());
        responder.send(buf, 0).unwrap();
    }
}
