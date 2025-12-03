use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

pub struct ProjectScanner {
    root: PathBuf,
}

impl ProjectScanner {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// Scans the project directory, respecting .gitignore and other ignore files
    pub fn scan(&self) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();

        for result in WalkBuilder::new(&self.root)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .hidden(true) // Include hidden files
            .build()
        {
            let entry = result?;
            let metadata = entry.metadata()?;

            files.push(FileInfo {
                path: entry.path().to_path_buf(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
            });
        }

        Ok(files)
    }

    /// Find files matching a specific pattern
    pub fn find_files(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let files = self.scan()?;
        let mut matching = Vec::new();

        for file in files {
            if file.is_dir {
                continue;
            }

            if let Some(filename) = file.path.file_name() {
                if filename.to_string_lossy().contains(pattern) {
                    matching.push(file.path);
                }
            }
        }

        Ok(matching)
    }

    /// Get project statistics
    pub fn get_stats(&self) -> Result<ProjectStats> {
        let files = self.scan()?;

        let mut stats = ProjectStats {
            total_files: 0,
            total_dirs: 0,
            total_size: 0,
        };

        for file in files {
            if file.is_dir {
                stats.total_dirs += 1;
            } else {
                stats.total_files += 1;
                stats.total_size += file.size;
            }
        }

        Ok(stats)
    }
}

#[derive(Debug, Clone)]
pub struct ProjectStats {
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_size: u64,
}
