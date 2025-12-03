use ignore::WalkBuilder;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::ToolError;

#[derive(Deserialize)]
pub struct SearchDirectoryArgs {
    pub directory: String,
    pub pattern: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct SearchDirectory;

impl Tool for SearchDirectory {
    const NAME: &'static str = "search_directory";

    type Error = ToolError;
    type Args = SearchDirectoryArgs;
    type Output = Vec<String>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "search_directory".to_string(),
            description: "Search for files in a directory, optionally filtering by pattern"
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "The directory to search in"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Optional pattern to filter files (e.g., '.rs', 'cargo')"
                    }
                },
                "required": ["directory"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut results = Vec::new();

        for result in WalkBuilder::new(&args.directory)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .hidden(true)
            .build()
        {
            let entry = result.map_err(|e| ToolError(format!("Walk error: {}", e)))?;
            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Apply pattern filter if provided
            if let Some(ref pattern) = args.pattern {
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if !filename_str.contains(pattern.as_str()) {
                        continue;
                    }
                }
            }

            results.push(path.display().to_string());
        }

        Ok(results)
    }
}
