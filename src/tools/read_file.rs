use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs;

use super::ToolError;

#[derive(Deserialize)]
pub struct ReadFileArgs {
    pub file_path: String,
}

#[derive(Deserialize, Serialize)]
pub struct ReadFile;

impl Tool for ReadFile {
    const NAME: &'static str = "read_file";

    type Error = ToolError;
    type Args = ReadFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read the contents of a file from the filesystem".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The path to the file to read"
                    }
                },
                "required": ["file_path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let content = fs::read_to_string(&args.file_path)
            .await
            .map_err(|e| ToolError(format!("Failed to read {}: {}", args.file_path, e)))?;

        // Format with line numbers
        let numbered_content: String = content
            .lines()
            .enumerate()
            .map(|(i, line)| format!("{:6}→{}", i + 1, line))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(numbered_content)
    }
}
