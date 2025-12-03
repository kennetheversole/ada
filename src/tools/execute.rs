use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::process::Command;

use super::ToolError;

#[derive(Deserialize)]
pub struct ExecuteArgs {
    pub command: String,
    pub working_dir: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Execute;

impl Tool for Execute {
    const NAME: &'static str = "execute";

    type Error = ToolError;
    type Args = ExecuteArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "execute".to_string(),
            description: "Execute a shell command and return its output".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    },
                    "working_dir": {
                        "type": "string",
                        "description": "Optional working directory for the command"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(&args.command);

        if let Some(working_dir) = args.working_dir {
            cmd.current_dir(working_dir);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| ToolError(format!("Failed to execute command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str("STDERR:\n");
            result.push_str(&stderr);
        }

        if !output.status.success() {
            result.push_str(&format!("\nExit code: {}", output.status));
        }

        if result.is_empty() {
            result = "Command executed successfully (no output)".to_string();
        }

        Ok(result)
    }
}
