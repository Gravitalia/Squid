use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Services {
    pub name: String,
    pub max_words: Option<u8>,
    pub message_type: Option<String>,
    pub lang: Option<String>,
    pub  exclude: Option<Vec<String>>
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub update_frequency_sec: u32,
    pub services: Vec<Services>,
}

pub fn read() -> Config {
    let config: Config = serde_yaml::from_reader(std::fs::File::open("config.yaml").expect("Could not find config.yaml file")).expect("Could not read values of config.yaml file");
    config
}