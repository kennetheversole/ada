use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use tokio::fs;

use super::formatter::{create_diff, ToolOutput};
use super::ToolError;

#[derive(Deserialize)]
pub struct FileToWrite {
    pub path: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct WriteFilesArgs {
    pub files: Vec<FileToWrite>,
}

#[derive(Deserialize, Serialize)]
pub struct WriteFiles;

impl Tool for WriteFiles {
    const NAME: &'static str = "write_files";

    type Error = ToolError;
    type Args = WriteFilesArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "write_files".to_string(),
            description: "Write content to multiple files at once".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "files": {
                        "type": "array",
                        "description": "Array of files to write",
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": {
                                    "type": "string",
                                    "description": "File path"
                                },
                                "content": {
                                    "type": "string",
                                    "description": "File content"
                                }
                            },
                            "required": ["path", "content"]
                        }
                    }
                },
                "required": ["files"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut outputs = Vec::new();

        for file in args.files {
            let path = Path::new(&file.path);

            // Create parent directories if they don't exist
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| ToolError(format!("Failed to create directory: {}", e)))?;
            }

            // Read old content if file exists
            let old_content = fs::read_to_string(&file.path).await.unwrap_or_default();

            // Write new content
            fs::write(&file.path, &file.content)
                .await
                .map_err(|e| ToolError(format!("Failed to write {}: {}", file.path, e)))?;

            // Create diff
            let diff = create_diff(&file.path, &old_content, &file.content, 2);
            let output = ToolOutput::new("WriteFile", &file.path).with_diff(diff);
            outputs.push(output.format());
        }

        Ok(outputs.join("\n"))
    }
}
