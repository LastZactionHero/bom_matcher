use std::error::Error;

use anyhow::Result;
use anyhow::anyhow;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateRequest {
    contents: Vec<Contents>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Contents {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Part {
    text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    candidates: Vec<Candidate>,
    usage_metadata: UsageMetadata,
    model_version: String,
    response_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Candidate {
    content: Content,
    finish_reason: String,
    index: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Content {
    parts: Vec<Part>,
    role: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UsageMetadata {
    prompt_token_count: u32,
    candidates_token_count: u32,
    total_token_count: u32,
    prompt_tokens_details: Vec<PromptTokenDetails>,
    thoughts_token_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PromptTokenDetails {
    modality: String,
    token_count: u32,
}

pub async fn generate_content(prompt: String) -> Result<String> {
    dotenv().ok();

    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

    let generate_content_url =
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";

    let generate_request = GenerateRequest {
        contents: vec![Contents {
            parts: vec![Part { text: prompt }],
        }],
    };
    let client = reqwest::Client::new();
    let response = client
        .post(generate_content_url)
        .header("x-goog-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&generate_request)
        .send()
        .await?;
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Gemini generate content request failed: {}", error_text).into());
    }

    let generate_response: GeminiResponse = response.json::<GeminiResponse>().await?;
    Ok(generate_response.candidates[0].content.parts[0]
        .text
        .to_string())
}

pub fn extract_json(s: &str) -> Option<String> {
    let start_tag = "```json\n";
    let end_tag = "```";

    if let Some(start) = s.find(start_tag) {
        let after_start = &s[start + start_tag.len()..];
        if let Some(end) = after_start.rfind(end_tag) {
            return Some((&after_start[..end].trim()).to_string());
        }
    }
    None
}
