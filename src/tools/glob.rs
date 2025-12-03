use globset::{Glob as GlobPattern, GlobSetBuilder};
use ignore::WalkBuilder;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::ToolError;

#[derive(Deserialize)]
pub struct GlobArgs {
    pub pattern: String,
    pub path: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Glob;

impl Tool for Glob {
    const NAME: &'static str = "glob";

    type Error = ToolError;
    type Args = GlobArgs;
    type Output = Vec<String>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "glob".to_string(),
            description: "Find files matching a glob pattern (e.g., '*.rs', '**/*.toml', 'src/**/*.rs')".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "The glob pattern to match (e.g., '*.rs', '**/*.toml')"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory to search in (default: current directory)"
                    }
                },
                "required": ["pattern"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let search_path = args.path.as_deref().unwrap_or(".");

        let glob = GlobPattern::new(&args.pattern)
            .map_err(|e| ToolError(format!("Invalid glob pattern: {}", e)))?;

        let mut builder = GlobSetBuilder::new();
        builder.add(glob);
        let glob_set = builder
            .build()
            .map_err(|e| ToolError(format!("Failed to build glob set: {}", e)))?;

        let mut results = Vec::new();

        for entry_result in WalkBuilder::new(search_path)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .hidden(false)
            .build()
        {
            let entry = entry_result.map_err(|e| ToolError(format!("Walk error: {}", e)))?;
            let path = entry.path();

            if path.is_file() && glob_set.is_match(path) {
                results.push(path.display().to_string());
            }
        }

        if results.is_empty() {
            Ok(vec!["No files matched the pattern".to_string()])
        } else {
            Ok(results)
        }
    }
}
