mod tools;
mod ui;
mod config;

use anyhow::Result;
use config::Config;
use rig::agent::Agent;
use rig::completion::Prompt;
use rig::prelude::*;
use rig::providers::openai;
use rig::providers::openai::responses_api::ResponsesCompletionModel;
use std::collections::HashSet;
use std::sync::Arc;
use tools::*;
use ui::{App, MessageRole, UI};

struct Ada {
    config: Config,
    intent_classifier: Agent<ResponsesCompletionModel>,
    code_agent: Agent<ResponsesCompletionModel>,
    file_agent: Agent<ResponsesCompletionModel>,
    git_agent: Agent<ResponsesCompletionModel>,
    execute_agent: Agent<ResponsesCompletionModel>,
    web_agent: Agent<ResponsesCompletionModel>,
    general_agent: Agent<ResponsesCompletionModel>,
    available_commands: HashSet<String>,
}

impl Ada {
    fn new() -> Self {
        // Load configuration from ~/.ada/config
        let config = Config::load().expect("Failed to load configuration");

        // Load all available commands from $PATH at startup
        let available_commands = Self::load_path_commands();
        eprintln!("Loaded {} commands from PATH", available_commands.len());

        let client = openai::Client::from_env();

        // Intent classifier - determines which specialized agent to use
        let intent_classifier = client
            .agent(openai::GPT_4)
            .preamble("You are an intent classifier. Analyze the user's request and classify it into ONE of these categories:
- code_search: searching code, finding functions/classes, grepping content, using regex
- file_ops: reading, editing, writing, moving, copying, deleting files, listing directories, showing file trees
- git: git operations like status, diff, log, commit, branch operations
- execution: running shell commands, executing scripts
- web: fetching web content, downloading from URLs
- general: general questions, help, or requests that don't fit above categories

Respond with ONLY the category name, nothing else.")
            .build();

        // Code search specialist
        let code_agent = client
            .agent(openai::GPT_4)
            .preamble("You are a code search specialist. Help users find and analyze code using grep, glob patterns, and search tools. When tools return formatted output, preserve it exactly.")
            .tool(Grep)
            .tool(Glob)
            .tool(SearchDirectory)
            .tool(ReadFile)
            .build();

        // File operations specialist
        let file_agent = client
            .agent(openai::GPT_4)
            .preamble("You are a file operations specialist. Help users read, edit, write, and manage files. When tools return formatted output (especially diffs with ⏺ symbols), ALWAYS include the complete tool output in your response without summarizing. Preserve all formatting, line numbers, and diff markers exactly as returned.")
            .tool(ReadFile)
            .tool(Edit)
            .tool(WriteFiles)
            .tool(FileOps)
            .tool(ListDirectory)
            .tool(Tree)
            .build();

        // Git operations specialist
        let git_agent = client
            .agent(openai::GPT_4)
            .preamble("You are a git operations specialist. Help users with git commands and repository management. When tools return formatted output, preserve it exactly.")
            .tool(Git)
            .tool(ReadFile)
            .build();

        // Shell execution specialist
        let execute_agent = client
            .agent(openai::GPT_4)
            .preamble("You are a shell command specialist. Help users execute commands safely. When tools return formatted output, preserve it exactly.")
            .tool(Execute)
            .build();

        // Web fetching specialist
        let web_agent = client
            .agent(openai::GPT_4)
            .preamble("You are a web fetching specialist. Help users retrieve content from URLs. When tools return formatted output, preserve it exactly.")
            .tool(WebFetch)
            .build();

        // General assistant for everything else
        let general_agent = client
            .agent(openai::GPT_4)
            .preamble("You are Ada, a helpful AI assistant. Answer questions and provide assistance.")
            .build();

        Self {
            config,
            intent_classifier,
            code_agent,
            file_agent,
            git_agent,
            execute_agent,
            web_agent,
            general_agent,
            available_commands,
        }
    }

    fn load_path_commands() -> HashSet<String> {
        use std::env;
        use std::fs;
        use std::path::Path;

        let mut commands = HashSet::new();

        // Get PATH environment variable
        let path_var = match env::var("PATH") {
            Ok(path) => path,
            Err(_) => return commands,
        };

        // Split PATH by ':' (Unix) or ';' (Windows)
        let separator = if cfg!(windows) { ';' } else { ':' };

        for dir in path_var.split(separator) {
            let dir_path = Path::new(dir);

            // Read directory entries
            if let Ok(entries) = fs::read_dir(dir_path) {
                for entry in entries.flatten() {
                    // Check if it's a file and executable
                    if let Ok(metadata) = entry.metadata() {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            // Check if executable (has execute permission)
                            if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                                if let Some(name) = entry.file_name().to_str() {
                                    commands.insert(name.to_string());
                                }
                            }
                        }

                        #[cfg(not(unix))]
                        {
                            // On non-Unix systems, just add all files
                            if metadata.is_file() {
                                if let Some(name) = entry.file_name().to_str() {
                                    commands.insert(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        commands
    }

    async fn process_command(&self, input: &str) -> String {
        // Check for special commands
        if input == "/help" {
            return self.show_help();
        }

        // Check if input is a direct shell command (if enabled)
        if self.config.enable_direct_commands {
            if let Some(direct_output) = self.try_direct_command(input).await {
                return direct_output;
            }
        }

        // First, classify the intent
        let intent = match self.intent_classifier.prompt(input).await {
            Ok(classification) => classification.trim().to_lowercase(),
            Err(e) => {
                return format!("Error classifying intent: {}", e);
            }
        };

        // Map intent to agent name for display
        let agent_name = match intent.as_str() {
            "code_search" => "Code Search",
            "file_ops" => "File Operations",
            "git" => "Git Operations",
            "execution" => "Shell Execution",
            "web" => "Web Fetching",
            _ => "General Assistant",
        };

        // Route to appropriate specialist agent using configured multi-turn depth
        let depth = self.config.multi_turn_depth;
        let result = match intent.as_str() {
            "code_search" => self.code_agent.prompt(input).multi_turn(depth).await,
            "file_ops" => self.file_agent.prompt(input).multi_turn(depth).await,
            "git" => self.git_agent.prompt(input).multi_turn(depth).await,
            "execution" => self.execute_agent.prompt(input).multi_turn(depth).await,
            "web" => self.web_agent.prompt(input).multi_turn(depth).await,
            _ => self.general_agent.prompt(input).multi_turn(depth / 2).await,
        };

        match result {
            Ok(response) => {
                if self.config.show_intent {
                    format!("Intent: {} → [{}]\n\n{}", intent, agent_name, response)
                } else {
                    format!("[{}]\n\n{}", agent_name, response)
                }
            }
            Err(e) => format!("Error calling AI agent: {}", e),
        }
    }

    async fn try_direct_command(&self, input: &str) -> Option<String> {
        let input = input.trim();
        let mut tokens = input.split_whitespace();

        // Get first token (command)
        let first_word = tokens.next()?.to_lowercase();

        // Skip if input looks like a natural language question
        let question_words = ["what", "how", "why", "when", "where", "who", "which", "can", "could", "would", "should", "is", "are", "do", "does"];
        if question_words.contains(&first_word.as_str()) {
            return None;
        }

        // Check if command exists in our pre-loaded PATH commands
        if !self.available_commands.contains(&first_word) {
            return None;
        }

        // Get second token if it exists and check if it looks like natural language
        if let Some(second_token) = tokens.next() {
            let second_lower = second_token.to_lowercase();

            // Natural language indicators - common words that suggest this is a question/request
            let natural_language_words = [
                "all", "the", "my", "this", "that", "these", "those",
                "every", "each", "some", "any", "many", "few",
                "a", "an", "me", "you", "it", "them", "us",
            ];

            // If second token is a natural language word (not a flag or path), skip direct execution
            if natural_language_words.contains(&second_lower.as_str()) {
                return None;
            }
        }

        // Execute the command directly
        use rig::tool::Tool;
        let result = Execute
            .call(tools::execute::ExecuteArgs {
                command: input.to_string(),
                working_dir: None,
            })
            .await;

        match result {
            Ok(output) => Some(format!("Direct Command: {}\n\n{}", first_word, output)),
            Err(e) => Some(format!("Command failed: {}", e)),
        }
    }

    fn show_help(&self) -> String {
        let mut help = String::from("Ada - AI Assistant with Intent Routing\n\n");

        // Show config info
        if let Ok(config_path) = Config::config_file_path() {
            help.push_str(&format!("Config: {}\n", config_path.display()));
        }
        help.push_str(&format!("Model: {} | Multi-turn depth: {} | Direct commands: {}\n\n",
            self.config.model,
            self.config.multi_turn_depth,
            if self.config.enable_direct_commands { "enabled" } else { "disabled" }
        ));

        if self.config.enable_direct_commands {
            help.push_str(&format!("Direct Commands: {} commands available from PATH\n", self.available_commands.len()));
            help.push_str("Type any system command (ls, git, cargo, etc.) to execute directly!\n\n");
        }

        help.push_str("I automatically route other requests to specialized agents:\n\n");

        help.push_str("Code Search Agent:\n");
        help.push_str("  - grep - Search file contents with regex\n");
        help.push_str("  - glob - Find files by pattern (*.rs, **/*.toml)\n");
        help.push_str("  - search_directory - Search directories\n");
        help.push_str("  - read_file - Read files\n\n");

        help.push_str("File Operations Agent:\n");
        help.push_str("  - read_file - Read file contents with line numbers\n");
        help.push_str("  - edit - Replace text in files (shows diffs)\n");
        help.push_str("  - write_files - Write multiple files at once\n");
        help.push_str("  - file_ops - Delete, move, copy files\n");
        help.push_str("  - list_directory - List files and folders\n");
        help.push_str("  - tree - Visual directory structure\n\n");

        help.push_str("Git Operations Agent:\n");
        help.push_str("  - git - Git operations (status, diff, log, commit)\n\n");

        help.push_str("Shell Execution Agent:\n");
        help.push_str("  - execute - Run shell commands\n\n");

        help.push_str("Web Fetching Agent:\n");
        help.push_str("  - webfetch - Fetch content from URLs\n\n");

        help.push_str("General Agent:\n");
        help.push_str("  - Answers general questions and provides assistance\n\n");

        help.push_str("Commands:\n");
        help.push_str("  /help - Show this help message\n\n");

        help.push_str("Examples:\n");
        help.push_str("  - \"find all TODO comments in src\"\n");
        help.push_str("  - \"edit Cargo.toml and add serde dependency\"\n");
        help.push_str("  - \"what's the git status?\"\n");
        help.push_str("  - \"run cargo build\"\n");
        help.push_str("  - \"what is Rust?\"\n");

        help
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Don't initialize tracing to avoid interfering with TUI
    // tracing_subscriber::fmt::init();

    let ada = Arc::new(Ada::new());
    let mut app = App::new();
    let mut ui = UI::new()?;

    // Main event loop
    loop {
        let message_count_before = app.messages.len();
        ui.handle_events(&mut app)?;

        if app.should_quit {
            break;
        }

        // Check if a new message was added
        if app.messages.len() > message_count_before {
            let last_message = app.messages.last().unwrap();
            if matches!(last_message.role, MessageRole::User) {
                let input = last_message.content.clone();
                let ada = Arc::clone(&ada);

                // Set processing state and redraw
                app.is_processing = true;
                ui.draw(&app)?;

                // Process the command
                let response = ada.process_command(&input).await;

                // Clear processing state
                app.is_processing = false;

                app.add_message(MessageRole::Assistant, response);
            }
        }

        // Only redraw when needed
        ui.draw(&app)?;
    }

    Ok(())
}
