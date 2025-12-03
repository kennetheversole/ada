# Ada - A Minimal Claude Code Clone

A small Rust implementation of Claude Code with a TUI interface built using Tokio and Ratatui.

## Features

- **Terminal User Interface (TUI)**: Interactive chat interface built with Ratatui
- **Tool System**: Extensible tool system with async trait support
- **Built-in Tools**:
  - `read_file`: Read files from the filesystem
  - `write_file`: Write content to files
  - `bash`: Execute shell commands
- **Project Scanner**: Gitignore-aware directory traversal using the `ignore` crate
- **Async Runtime**: Built on Tokio for async I/O operations

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

### Commands

- `/help` - Show available commands
- `/read <file_path>` - Read a file
- `/write <file_path> <content>` - Write content to a file
- `/exec <command>` - Execute a bash command
- `/scan` - Scan the current project directory

### Controls

- Type your message or command and press `Enter` to submit
- Press `Ctrl+C` to quit the application

## Architecture

### Project Structure

```
ada/
├── src/
│   ├── main.rs           # Main application and event loop
│   ├── ui.rs             # TUI interface with Ratatui
│   ├── scanner.rs        # Project directory scanner
│   └── tools/
│       ├── mod.rs        # Tool trait definition
│       ├── read_file.rs  # File reading tool
│       ├── write_file.rs # File writing tool
│       └── bash.rs       # Command execution tool
└── Cargo.toml
```

### Tool System

Tools implement the `Tool` trait:

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn execute(&self, parameters: serde_json::Value) -> Result<ToolResult>;
}
```

## Future Enhancements

- [ ] Add AI API integration (currently commented out)
- [ ] Implement more tools (grep, edit, glob, etc.)
- [ ] Add conversation history
- [ ] Implement file watching with `notify`
- [ ] Add syntax highlighting with `tree-sitter`
- [ ] Support for streaming responses

## Dependencies

- `tokio` - Async runtime
- `ratatui` - Terminal UI framework
- `crossterm` - Terminal manipulation
- `serde` & `serde_json` - Serialization
- `anyhow` - Error handling
- `ignore` - Gitignore-aware file walking
- `git2` - Git repository operations
- `notify` - File system notifications
- `tree-sitter` - Code parsing
- `reqwest` - HTTP client (for AI API calls)

## License

MIT
