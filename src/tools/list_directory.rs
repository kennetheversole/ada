use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs;

use super::ToolError;

#[derive(Deserialize)]
pub struct ListDirectoryArgs {
    pub path: Option<String>,
    pub show_hidden: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct ListDirectory;

impl Tool for ListDirectory {
    const NAME: &'static str = "list_directory";

    type Error = ToolError;
    type Args = ListDirectoryArgs;
    type Output = Vec<String>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "list_directory".to_string(),
            description: "List files and directories in a given path".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path to list (default: current directory)"
                    },
                    "show_hidden": {
                        "type": "boolean",
                        "description": "Show hidden files (default: false)"
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = args.path.as_deref().unwrap_or(".");
        let show_hidden = args.show_hidden.unwrap_or(false);

        let mut entries = fs::read_dir(path)
            .await
            .map_err(|e| ToolError(format!("Failed to read directory {}: {}", path, e)))?;

        let mut results = Vec::new();

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| ToolError(format!("Failed to read directory entry: {}", e)))?
        {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Skip hidden files if not requested
            if !show_hidden && file_name_str.starts_with('.') {
                continue;
            }

            let metadata = entry
                .metadata()
                .await
                .map_err(|e| ToolError(format!("Failed to read metadata: {}", e)))?;

            let entry_type = if metadata.is_dir() {
                "DIR "
            } else if metadata.is_symlink() {
                "LINK"
            } else {
                "FILE"
            };

            results.push(format!("{} {}", entry_type, file_name_str));
        }

        results.sort();

        if results.is_empty() {
            Ok(vec!["Empty directory".to_string()])
        } else {
            Ok(results)
        }
    }
}
