//! # Embeddings

use std::env;

use anyhow::{anyhow, Ok, Result};
use orgize::Org;
use reqwest::blocking::{Client, ClientBuilder};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct EmbeddingsProvider {
    url: String,
    model: String,
}

#[derive(Deserialize, Debug, Clone)]
struct OllamaResponse {
    model: String,
    embeddings: Vec<Vec<f32>>,
    total_duration: u64,
    load_duration: u64,
    prompt_eval_count: u64,
}

impl EmbeddingsProvider {
    pub fn new() -> Self {
        let ollama = env::var("ROAMER_OLLAMA_HOST").unwrap_or(String::from("localhost:11434"));
        let model = env::var("ROAMER_OLLAMA_MODEL").unwrap_or(String::from("mxbai-embed-large"));
        let url = format!("http://{ollama}/api/embed");
        Self { url, model }
    }

    pub fn generate(&self, content: &Org) -> Result<Vec<f32>> {
        // TODO: Make nicer text extraction without as much formatting
        let content = content.to_org();

        let body = json!({
            "model": self.model,
            "input": content,
        });

        let client = Client::new();
        let req = client.post(self.url.clone()).json(&body);

        let resp = req.send()?;
        let embed: OllamaResponse = resp.json()?;
        let embeds = embed
            .embeddings
            .get(0)
            .ok_or(anyhow!("no embeds returned by ollama"))?;
        let owned = embeds.to_owned();

        Ok(owned)
    }
}
