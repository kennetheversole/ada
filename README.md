# Ada - AI Assistant with Intent Routing

A Rust-based AI assistant featuring intent-based routing to specialized agents, direct command execution, and a clean TUI interface. Built with Tokio, Ratatui, and OpenAI's GPT-4 via rig-core.

## Features

- **Intent Routing**: Automatically classifies requests and routes to specialized agents
- **Direct Command Execution**: Bypass AI for whitelisted shell commands (ls, git, cargo, etc.)
- **Terminal UI**: Clean, responsive chat interface with Ratatui
- **12 Powerful Tools**: File ops, code search, git, shell execution, web fetching, and more
- **Async Runtime**: Built on Tokio for high-performance async I/O
- **Formatted Diffs**: Claude Code-style diff output for file operations

## Getting Started

### Prerequisites

- Rust 1.89.0 or later
- Cargo

### Installation

```bash
cd ada
cargo build --release
```

### Running

Set your OpenAI API key first:
```bash
export OPENAI_API_KEY=sk-your-key-here
```

Then run Ada (use release mode for better performance):
```bash
cargo run --release
```

Or use the helper script:
```bash
./run.sh
```

Press `Ctrl+C` to quit.

## Usage

### Direct Commands

Type any whitelisted command directly:
```
> ls
> cat Cargo.toml
> git status
> cargo build
```

### Natural Language

Ask questions or give instructions in natural language:
```
> what files are in src?
> edit Cargo.toml and add tokio dependency
> find all TODO comments in the codebase
> what is Rust?
```

### Special Commands

- `/help` - Show available tools and agents

### Controls

- Type your message and press `Enter` to submit
- Press `Ctrl+C` to quit the application
- Text selection/copying works in the terminal

## Architecture

### Intent Routing System

Ada uses a multi-agent architecture with intent-based routing:

1. **Intent Classifier** - Analyzes user input and classifies into categories
2. **Specialized Agents** - Each agent focuses on specific tasks with relevant tools:
   - **Code Search Agent**: grep, glob, search_directory, read_file
   - **File Operations Agent**: read_file, edit, write_files, file_ops, list_directory, tree
   - **Git Operations Agent**: git, read_file
   - **Shell Execution Agent**: execute
   - **Web Fetching Agent**: webfetch
   - **General Agent**: Answers general questions

### Project Structure

```
ada/
├── src/
│   ├── main.rs           # Intent routing and agent orchestration
│   ├── ui.rs             # TUI interface with Ratatui
│   ├── scanner.rs        # Project directory scanner
│   └── tools/
│       ├── mod.rs        # Tool exports and common types
│       ├── formatter.rs  # Diff formatting for file operations
│       ├── read_file.rs  # Read files with line numbers
│       ├── edit.rs       # String replacement with diffs
│       ├── write_files.rs # Write multiple files at once
│       ├── file_ops.rs   # Delete, move, copy operations
│       ├── list_directory.rs # List directory contents
│       ├── tree.rs       # Visual directory structure
│       ├── grep.rs       # Regex search in files
│       ├── glob.rs       # File pattern matching
│       ├── search_directory.rs # Directory search
│       ├── git.rs        # Git operations
│       ├── execute.rs    # Shell command execution
│       └── webfetch.rs   # HTTP fetching
└── Cargo.toml
```

### Tool System

Tools implement the `rig::tool::Tool` trait:

```rust
impl Tool for Edit {
    const NAME: &'static str = "edit";
    type Error = ToolError;
    type Args = EditArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition { ... }
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> { ... }
}
```

## Tools

### Code Search
- **grep**: Search file contents with regex patterns
- **glob**: Find files by pattern (*.rs, **/*.toml)
- **search_directory**: Search directories with filters

### File Operations
- **read_file**: Read files with line numbers
- **edit**: Replace text with diff output
- **write_files**: Write multiple files at once
- **file_ops**: Delete, move, copy files and directories
- **list_directory**: List directory contents
- **tree**: Show visual directory tree

### Development
- **git**: Git operations (status, diff, log, commit, etc.)
- **execute**: Run shell commands

### Web
- **webfetch**: Fetch content from URLs

## Dependencies

- `tokio` - Async runtime
- `ratatui` - Terminal UI framework
- `crossterm` - Terminal manipulation
- `serde` & `serde_json` - Serialization
- `anyhow` & `thiserror` - Error handling
- `rig-core` - OpenAI agent framework (from GitHub)
- `ignore` - Gitignore-aware file walking
- `regex` - Regular expressions
- `globset` - Glob pattern matching
- `similar` - Diff generation
- `reqwest` - HTTP client

## License

MIT
