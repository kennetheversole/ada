use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::process::Command;

use super::ToolError;

#[derive(Deserialize)]
pub struct GitArgs {
    pub operation: String,
    pub args: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize)]
pub struct Git;

impl Tool for Git {
    const NAME: &'static str = "git";

    type Error = ToolError;
    type Args = GitArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "git".to_string(),
            description: "Execute git operations (status, diff, log, add, commit, etc.)".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "description": "Git operation to perform (status, diff, log, add, commit, etc.)"
                    },
                    "args": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Additional arguments for the git command"
                    }
                },
                "required": ["operation"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut cmd = Command::new("git");
        cmd.arg(&args.operation);

        if let Some(extra_args) = args.args {
            cmd.args(&extra_args);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| ToolError(format!("Failed to execute git: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(ToolError(format!(
                "Git command failed:\n{}{}",
                stdout, stderr
            )));
        }

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&stderr);
        }

        if result.is_empty() {
            result = "Command completed successfully".to_string();
        }

        Ok(result)
    }
}
