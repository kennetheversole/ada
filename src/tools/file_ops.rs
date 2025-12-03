use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs;

use super::formatter::{create_diff, ToolOutput};
use super::ToolError;

#[derive(Deserialize)]
pub struct FileOpsArgs {
    pub operation: String,
    pub source: String,
    pub destination: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct FileOps;

impl Tool for FileOps {
    const NAME: &'static str = "file_ops";

    type Error = ToolError;
    type Args = FileOpsArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "file_ops".to_string(),
            description: "Perform file operations: delete, move, rename, copy".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "description": "Operation to perform: 'delete', 'move', 'copy'"
                    },
                    "source": {
                        "type": "string",
                        "description": "Source file or directory path"
                    },
                    "destination": {
                        "type": "string",
                        "description": "Destination path (required for move/copy operations)"
                    }
                },
                "required": ["operation", "source"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match args.operation.as_str() {
            "delete" => {
                let metadata = fs::metadata(&args.source)
                    .await
                    .map_err(|e| ToolError(format!("Failed to access {}: {}", args.source, e)))?;

                let item_type = if metadata.is_dir() { "directory" } else { "file" };

                if metadata.is_dir() {
                    fs::remove_dir_all(&args.source)
                        .await
                        .map_err(|e| ToolError(format!("Failed to delete directory: {}", e)))?;
                } else {
                    fs::remove_file(&args.source)
                        .await
                        .map_err(|e| ToolError(format!("Failed to delete file: {}", e)))?;
                }

                let output = ToolOutput::new("Delete", &args.source)
                    .with_details(format!("Deleted {} {}", item_type, args.source));
                Ok(output.format())
            }
            "move" => {
                let destination = args
                    .destination
                    .ok_or_else(|| ToolError("Destination required for move operation".to_string()))?;

                fs::rename(&args.source, &destination)
                    .await
                    .map_err(|e| ToolError(format!("Failed to move file: {}", e)))?;

                let output = ToolOutput::new("Move", &args.source)
                    .with_details(format!("Moved {} to {}", args.source, destination));
                Ok(output.format())
            }
            "copy" => {
                let destination = args
                    .destination
                    .ok_or_else(|| ToolError("Destination required for copy operation".to_string()))?;

                let metadata = fs::metadata(&args.source)
                    .await
                    .map_err(|e| ToolError(format!("Failed to access source: {}", e)))?;

                if metadata.is_dir() {
                    return Err(ToolError(
                        "Copying directories not yet supported".to_string(),
                    ));
                }

                // Read old destination content if it exists
                let old_content = fs::read_to_string(&destination).await.unwrap_or_default();
                let source_content = fs::read_to_string(&args.source)
                    .await
                    .map_err(|e| ToolError(format!("Failed to read source: {}", e)))?;

                fs::copy(&args.source, &destination)
                    .await
                    .map_err(|e| ToolError(format!("Failed to copy file: {}", e)))?;

                // Show diff if destination had content, otherwise just details
                if old_content.is_empty() {
                    let output = ToolOutput::new("Copy", &destination)
                        .with_details(format!("Copied {} to {}", args.source, destination));
                    Ok(output.format())
                } else {
                    let diff = create_diff(&destination, &old_content, &source_content, 2);
                    let output = ToolOutput::new("Copy", &destination).with_diff(diff);
                    Ok(output.format())
                }
            }
            _ => Err(ToolError(format!(
                "Unknown operation: {}. Use 'delete', 'move', or 'copy'",
                args.operation
            ))),
        }
    }
}
