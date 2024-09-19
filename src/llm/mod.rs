use reqwest;
use serde_json::json;
use anyhow::Result;

pub struct LLMService {
    client: reqwest::Client,
    api_key: String,
}

impl LLMService {
    pub fn new() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        LLMService {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    pub async fn summarize(&self, content: &str) -> Result<String> {
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": "gpt-4",
                "messages": [
                    {"role": "system", "content": "You are a helpful assistant that summarizes content."},
                    {"role": "user", "content": format!("Please summarize the following content:\n\n{}", content)}
                ],
                "max_tokens": 150,
                "n": 1,
                "temperature": 0.5,
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(response["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string())
    }

    pub async fn analyze_content(&self, content: &str) -> Result<String> {
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": "gpt-4",
                "messages": [
                    {"role": "system", "content": "You are a helpful assistant that analyzes content and provides key insights."},
                    {"role": "user", "content": format!("Please analyze the following content and provide key insights:\n\n{}", content)}
                ],
                "max_tokens": 200,
                "n": 1,
                "temperature": 0.7,
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(response["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string())
    }
}