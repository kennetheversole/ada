use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::ToolError;

#[derive(Deserialize)]
pub struct WebFetchArgs {
    pub url: String,
}

#[derive(Deserialize, Serialize)]
pub struct WebFetch;

impl Tool for WebFetch {
    const NAME: &'static str = "webfetch";

    type Error = ToolError;
    type Args = WebFetchArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "webfetch".to_string(),
            description: "Fetch content from a URL (useful for reading documentation, APIs, etc.)"
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to fetch"
                    }
                },
                "required": ["url"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let client = reqwest::Client::builder()
            .user_agent("Ada/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ToolError(format!("Failed to create HTTP client: {}", e)))?;

        let response = client
            .get(&args.url)
            .send()
            .await
            .map_err(|e| ToolError(format!("Failed to fetch {}: {}", args.url, e)))?;

        if !response.status().is_success() {
            return Err(ToolError(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        let content = response
            .text()
            .await
            .map_err(|e| ToolError(format!("Failed to read response body: {}", e)))?;

        // Limit response size
        const MAX_SIZE: usize = 100_000; // 100KB limit
        if content.len() > MAX_SIZE {
            Ok(format!(
                "{}... (truncated, total size: {} bytes)",
                &content[..MAX_SIZE],
                content.len()
            ))
        } else {
            Ok(content)
        }
    }
}
