use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs;

use super::formatter::{create_diff, ToolOutput};
use super::ToolError;

#[derive(Deserialize)]
pub struct EditArgs {
    pub file_path: String,
    pub old_string: String,
    pub new_string: String,
    pub replace_all: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct Edit;

impl Tool for Edit {
    const NAME: &'static str = "edit";

    type Error = ToolError;
    type Args = EditArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "edit".to_string(),
            description: "Replace text in a file by finding and replacing exact strings"
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The path to the file to edit"
                    },
                    "old_string": {
                        "type": "string",
                        "description": "The exact string to find and replace"
                    },
                    "new_string": {
                        "type": "string",
                        "description": "The new string to replace with"
                    },
                    "replace_all": {
                        "type": "boolean",
                        "description": "If true, replace all occurrences. If false, only replace first occurrence. Default: false"
                    }
                },
                "required": ["file_path", "old_string", "new_string"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let old_content = fs::read_to_string(&args.file_path)
            .await
            .map_err(|e| ToolError(format!("Failed to read {}: {}", args.file_path, e)))?;

        let replace_all = args.replace_all.unwrap_or(false);

        // Check if old_string exists
        if !old_content.contains(&args.old_string) {
            return Err(ToolError(format!(
                "String not found in file: '{}'",
                args.old_string
            )));
        }

        let new_content = if replace_all {
            old_content.replace(&args.old_string, &args.new_string)
        } else {
            old_content.replacen(&args.old_string, &args.new_string, 1)
        };

        fs::write(&args.file_path, &new_content)
            .await
            .map_err(|e| ToolError(format!("Failed to write {}: {}", args.file_path, e)))?;

        // Create diff
        let diff = create_diff(&args.file_path, &old_content, &new_content, 2);

        let output = ToolOutput::new("Edit", &args.file_path).with_diff(diff);

        Ok(output.format())
    }
}
