use squid::squid_client::SquidClient;
use squid::AddRequest;

pub mod squid {
    tonic::include_proto!("squid");
}

const SENTENCES: [&str; 1] = [
    "Gravitalia is amazing",
    /*"Retour en images sur la table ronde organisée ce matin à la Cour pour la #JourneeDesDroitsDesFemmes.",
    "Gravitalia is cool!",
    "Retour en images sur la table ronde organisée ce matin à la Cour pour la #JourneeDesDroitsDesFemmes. L'occasion d'échanger sur les meilleures pratiques en termes de lutte contre les discriminations et de promotion de la #diversité au sein des organisations de travail.",
    "L'#IVG va être inscrit dans la Constitution. Mais le combat continue : la @FranceInsoumise déposera un projet de résolution pour que le droit à l'IVG soit inscrit dans la Charte européenne des droits fondamentaux. #8mars #JourneeDesDroitsDesFemmes",
    "À nos grands-mères, à nos mères, à nos femmes, à nos filles, à nos amies, à vous toutes. #JourneeDesDroitsDesFemmes #WomensDay",
    "🌍 En ce vendredi 8 mars, #JourneeDesDroitsDesFemmes, nous avons une tendre pensée pour Simone Veil.",
    "The last time Mbappé played the entire match against Reims ☺️",
    "Mbappé's reaction after Reims scored the equalizer.😂",
    "The fans reaction after Mbappé got subbed on.❤️",
    "dembele and mbappe are so good but a certain hag would rather bench them and start \"carlos soler\"",
    "🚨 American rapper Megan Thee Stallion is in the top tweet in France because of the racism that Aya Nakamura is currently experiencing. French racists are not capable of distinguishing between several black women. #SoutienAyaNakamura",
    "🚨De nombreux artistes tels que Dadju, Joe Dwet Filé et Nej expriment leur soutien à Aya Nakamura suite au lynchage raciste qu'elle subit depuis l'annonce de sa prestation aux JO.",
    "J'ai rencontré Aya Nakamura à la sortie d'un petit festival pour lequel elle était présente.",*/
];

#[tokio::main]
async fn main() {
    // Calculate time taken.
    let now: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    // Set words into the database.
    // This doesn't really send a response.
    //for _ in 1..6000 {
    for sentence in SENTENCES {
        let _ = SquidClient::connect("http://localhost:50051")
            .await
            .unwrap()
            .add(AddRequest {
                sentence: sentence.to_string(),
                lifetime: 10,
            })
            .await
            .unwrap()
            .into_inner();
    }
    //}

    println!(
        "Set in {}ms",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            - now
    );
}
