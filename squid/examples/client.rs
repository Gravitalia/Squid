use squid::squid_client::SquidClient;
use squid::LeaderboardRequest;

pub mod squid {
    tonic::include_proto!("squid");
}

#[tokio::main]
async fn main() {
    // Calculate time taken.
    let now: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    // Get leaderboard of 10 most used words.
    let response = SquidClient::connect("http://localhost:50051")
        .await
        .unwrap()
        .leaderboard(LeaderboardRequest { length: 10 })
        .await
        .unwrap()
        .into_inner();

    println!(
        "Received {:?} in {}ms",
        response.word,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            - now
    );
}
