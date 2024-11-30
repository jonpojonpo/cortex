// src/claude/mod.rs
mod text_formatter;
use text_formatter::TextFormatter;

use anyhow::{Result, Context};
use reqwest::{Client as HttpClient, header};
use serde::Deserialize;
use colored::*;
use serde_json::Value;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1";

#[derive(Debug)]
pub struct ClaudeClient {
    http_client: HttpClient,
    text_formatter: TextFormatter,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ApiResponse {
    Success(SuccessResponse),
    Error(ErrorResponse),
}

#[derive(Debug, Deserialize)]
struct SuccessResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    model: String,
    content: Vec<Content>,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    #[serde(rename = "type")]
    error_type: String,
    error: ErrorDetails,
}

#[derive(Debug, Deserialize)]
struct ErrorDetails {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

impl ClaudeClient {
    pub fn new(api_key: String) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            header::HeaderValue::from_str(&api_key)?,
        );
        headers.insert(
            "anthropic-version",
            header::HeaderValue::from_static("2023-06-01"),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let http_client = HttpClient::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { 
            http_client,
            text_formatter: TextFormatter::new(),
        })
    }

    pub async fn send_message(&self, prompt: &str) -> Result<String> {
        self.send_message_with_history(prompt, &[]).await
    }

    pub async fn send_message_with_history(&self, prompt: &str, history: &[Value]) -> Result<String> {
        let mut messages = Vec::from(history);
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt,
        }));

        let request = serde_json::json!({
            "model": "claude-3-5-sonnet-20241022",
            "messages": messages,
            "max_tokens": 1024,
        });

        let response = self.http_client
            .post(&format!("{}/messages", ANTHROPIC_API_URL))
            .json(&request)
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        let response_text = response.text().await?;

        // Debug output
        eprintln!("{}", "Request:".yellow());
        eprintln!("{}", serde_json::to_string_pretty(&request).unwrap().yellow());
        eprintln!("{}", format!("Status: {}", status).cyan());
        eprintln!("{}", "Response:".magenta());
        eprintln!("{}", response_text.magenta());

        let api_response: ApiResponse = serde_json::from_str(&response_text)
            .context("Failed to parse API response")?;

        match api_response {
            ApiResponse::Success(success) => {
                // Get the raw text and process it
                let raw_text = success.content.get(0)
                    .map(|c| c.text.clone())
                    .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

                // Process the text with our formatter
                let formatted_text = self.text_formatter.format_text(&raw_text);

                eprintln!("{}", format!(
                    "Tokens used: {} input, {} output",
                    success.usage.input_tokens,
                    success.usage.output_tokens
                ).bright_black());

                Ok(formatted_text)
            },
            ApiResponse::Error(error) => {
                anyhow::bail!("{}: {}", error.error.error_type, error.error.message)
            }
        }
    }
}
