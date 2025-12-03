use ignore::WalkBuilder;
use regex::Regex;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs;

use super::ToolError;

#[derive(Deserialize)]
pub struct GrepArgs {
    pub pattern: String,
    pub path: Option<String>,
    pub case_insensitive: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct Grep;

impl Tool for Grep {
    const NAME: &'static str = "grep";

    type Error = ToolError;
    type Args = GrepArgs;
    type Output = Vec<String>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "grep".to_string(),
            description: "Search for a pattern in files (regex supported)".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "The regex pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "File or directory to search in (default: current directory)"
                    },
                    "case_insensitive": {
                        "type": "boolean",
                        "description": "Case insensitive search (default: false)"
                    }
                },
                "required": ["pattern"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let search_path = args.path.as_deref().unwrap_or(".");
        let case_insensitive = args.case_insensitive.unwrap_or(false);

        let regex_pattern = if case_insensitive {
            format!("(?i){}", args.pattern)
        } else {
            args.pattern.clone()
        };

        let re = Regex::new(&regex_pattern)
            .map_err(|e| ToolError(format!("Invalid regex pattern: {}", e)))?;

        let mut results = Vec::new();

        // Check if path is a file or directory
        let path = std::path::Path::new(search_path);
        if path.is_file() {
            // Search single file
            let content = fs::read_to_string(path)
                .await
                .map_err(|e| ToolError(format!("Failed to read file: {}", e)))?;

            for (line_num, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    results.push(format!("{}:{}: {}", search_path, line_num + 1, line));
                }
            }
        } else {
            // Search directory
            for entry_result in WalkBuilder::new(search_path)
                .git_ignore(true)
                .git_global(true)
                .git_exclude(true)
                .hidden(false)
                .build()
            {
                let entry = entry_result.map_err(|e| ToolError(format!("Walk error: {}", e)))?;
                let entry_path = entry.path();

                if entry_path.is_file() {
                    if let Ok(content) = fs::read_to_string(entry_path).await {
                        for (line_num, line) in content.lines().enumerate() {
                            if re.is_match(line) {
                                results.push(format!(
                                    "{}:{}: {}",
                                    entry_path.display(),
                                    line_num + 1,
                                    line
                                ));
                            }
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            Ok(vec!["No matches found".to_string()])
        } else {
            Ok(results)
        }
    }
}
