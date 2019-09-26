use reqwest;
use reqwest::Client;

const BASE_DEEZER_API: &str = "https://api.deezer.com/";

pub struct DeezerClient {
    client: Client,
}

impl DeezerClient {
    pub fn new() -> Self {
        DeezerClient {
            client: Client::new(),
        }
    }

    pub fn search(&self) -> String {
        self.client
            .get(&[BASE_DEEZER_API, "search/track"].concat())
            .query(&[("q", "owl city")])
            .send()
            .expect("Panic!!!")
            .text()
            .expect("More Panic!!!")
    }

    pub fn track(&self) -> String {
        self.client
            .get(&[BASE_DEEZER_API, "track/4188437"].concat())
            .send()
            .expect("Panic!!!")
            .text()
            .expect("More Panic!!!")
    }
}
