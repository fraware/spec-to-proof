use std::error::Error;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use tokio::time::sleep;

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
    usage: ClaudeUsage,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

pub struct ClaudeClient {
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
    http_client: Client,
    base_url: String,
}

impl ClaudeClient {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            max_tokens: 4000,
            temperature: 0.0,
            http_client: Client::new(),
            base_url: "https://api.anthropic.com/v1/messages".to_string(),
        }
    }

    pub async fn generate_response(
        &self,
        prompt: &str,
        max_retries: u32,
        retry_delay_ms: u64,
    ) -> Result<(String, u32, u32), Box<dyn Error>> {
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match self.make_request(prompt).await {
                Ok((response_text, input_tokens, output_tokens)) => {
                    tracing::info!(
                        "Claude API call successful: {} input tokens, {} output tokens",
                        input_tokens,
                        output_tokens
                    );
                    return Ok((response_text, input_tokens, output_tokens));
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        tracing::warn!(
                            "Claude API call failed (attempt {}/{}), retrying in {}ms: {}",
                            attempt + 1,
                            max_retries + 1,
                            retry_delay_ms,
                            last_error.as_ref().unwrap()
                        );
                        sleep(Duration::from_millis(retry_delay_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| "Unknown error".into()))
    }

    async fn make_request(&self, prompt: &str) -> Result<(String, u32, u32), Box<dyn Error>> {
        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            messages: vec![
                ClaudeMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
        };

        let response = self.http_client
            .post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Claude API error: {} - {}", response.status(), error_text).into());
        }

        let claude_response: ClaudeResponse = response.json().await?;
        
        if claude_response.content.is_empty() {
            return Err("Empty response from Claude API".into());
        }

        let response_text = claude_response.content[0].text.clone();
        let input_tokens = claude_response.usage.input_tokens;
        let output_tokens = claude_response.usage.output_tokens;

        Ok((response_text, input_tokens, output_tokens))
    }

    pub fn estimate_cost(&self, input_tokens: u32, output_tokens: u32, cost_per_1k_tokens: f64) -> f64 {
        let total_tokens = input_tokens + output_tokens;
        (total_tokens as f64 / 1000.0) * cost_per_1k_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_claude_client_creation() {
        let client = ClaudeClient::new("test_key", "claude-3-opus-20240229");
        assert_eq!(client.model, "claude-3-opus-20240229");
        assert_eq!(client.temperature, 0.0);
    }

    #[test]
    fn test_cost_estimation() {
        let client = ClaudeClient::new("test_key", "claude-3-opus-20240229");
        let cost = client.estimate_cost(1000, 500, 0.015);
        assert!((cost - 0.0225).abs() < 0.001); // (1500/1000) * 0.015
    }
} 