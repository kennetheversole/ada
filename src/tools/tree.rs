use ignore::WalkBuilder;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;

use super::ToolError;

#[derive(Deserialize)]
pub struct TreeArgs {
    pub path: Option<String>,
    pub max_depth: Option<usize>,
}

#[derive(Deserialize, Serialize)]
pub struct Tree;

impl Tool for Tree {
    const NAME: &'static str = "tree";

    type Error = ToolError;
    type Args = TreeArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "tree".to_string(),
            description: "Display directory structure as a tree".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path to display (default: current directory)"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum depth to traverse (default: 3)"
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let root_path = args.path.as_deref().unwrap_or(".");
        let max_depth = args.max_depth.unwrap_or(3);

        let root = Path::new(root_path);
        let mut tree = String::new();
        tree.push_str(&format!("{}\n", root.display()));

        let mut entries: Vec<(String, usize, bool)> = Vec::new();

        for entry_result in WalkBuilder::new(root_path)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .hidden(false)
            .max_depth(Some(max_depth))
            .build()
        {
            let entry = entry_result.map_err(|e| ToolError(format!("Walk error: {}", e)))?;
            let path = entry.path();

            if path == root {
                continue;
            }

            let depth = path
                .strip_prefix(root)
                .map(|p| p.components().count())
                .unwrap_or(0);

            if depth > max_depth {
                continue;
            }

            let is_dir = path.is_dir();
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            entries.push((name, depth, is_dir));
        }

        // Sort entries
        entries.sort_by(|a, b| {
            let depth_cmp = a.1.cmp(&b.1);
            if depth_cmp == std::cmp::Ordering::Equal {
                a.0.cmp(&b.0)
            } else {
                depth_cmp
            }
        });

        for (name, depth, is_dir) in entries {
            let indent = "  ".repeat(depth.saturating_sub(1));
            let prefix = if is_dir { "📁 " } else { "📄 " };
            tree.push_str(&format!("{}├─ {}{}\n", indent, prefix, name));
        }

        Ok(tree)
    }
}
