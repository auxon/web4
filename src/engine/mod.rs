use serde::{Serialize, Deserialize};
use crate::llm::LLMService;
use crate::network::NetworkService;
use anyhow::{Result};

#[derive(Serialize, Deserialize, Debug)]
pub struct BrowserResponse {
    pub content: String,
    pub summary: String,
    pub analysis: String,
    pub url: String,
}

pub struct BrowserEngine {
    llm_service: LLMService,
    network_service: NetworkService,
}

impl BrowserEngine {
    pub fn new() -> Self {
        BrowserEngine {
            llm_service: LLMService::new(),
            network_service: NetworkService::new(),
        }
    }

    pub async fn load_url(&self, url: &str) -> Result<BrowserResponse> {
        println!("BrowserEngine: Loading URL: {}", url);
        let content = self.network_service.fetch(url).await?;
        println!("BrowserEngine: Fetched content length: {}", content.len());
        let summary = self.llm_service.summarize(&content).await?;
        println!("BrowserEngine: Generated summary length: {}", summary.len());
        let analysis = self.llm_service.analyze_content(&content).await?;
        println!("BrowserEngine: Generated analysis length: {}", analysis.len());
        
        Ok(BrowserResponse {
            content,
            summary,
            analysis,
            url: url.to_string(),
        })
    }
}