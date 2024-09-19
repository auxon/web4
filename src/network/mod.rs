use reqwest::Client;
use anyhow::Result;

pub struct NetworkService {
    client: Client,
}

impl NetworkService {
    pub fn new() -> Self {
        NetworkService {
            client: Client::new(),
        }
    }

    pub async fn fetch(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;
        let content = response.text().await?;
        Ok(content)
    }
}