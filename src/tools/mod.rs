// Tool modules
pub mod formatter;
pub mod read_file;
pub mod search_directory;
pub mod edit;
pub mod grep;
pub mod glob;
pub mod git;
pub mod webfetch;
pub mod execute;
pub mod list_directory;
pub mod write_files;
pub mod file_ops;
pub mod tree;

// Re-export tools for easy access
pub use read_file::ReadFile;
pub use search_directory::SearchDirectory;
pub use edit::Edit;
pub use grep::Grep;
pub use glob::Glob;
pub use git::Git;
pub use webfetch::WebFetch;
pub use execute::Execute;
pub use list_directory::ListDirectory;
pub use write_files::WriteFiles;
pub use file_ops::FileOps;
pub use tree::Tree;

// Common error type for all tools
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ToolError(pub String);
