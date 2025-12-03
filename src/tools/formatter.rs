use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub tool_name: String,
    pub summary: String,
    pub details: Option<String>,
    pub diff: Option<FileDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub file_path: String,
    pub additions: usize,
    pub removals: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_number: usize,
    pub change_type: ChangeType,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Context,
    Addition,
    Removal,
}

impl ToolOutput {
    pub fn new(tool_name: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            summary: summary.into(),
            details: None,
            diff: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn with_diff(mut self, diff: FileDiff) -> Self {
        self.diff = Some(diff);
        self
    }

    pub fn format(&self) -> String {
        let mut output = String::new();

        // Tool header
        output.push_str(&format!("⏺ {}({})\n", self.tool_name, self.summary));

        // Diff if present
        if let Some(diff) = &self.diff {
            output.push_str(&format!(
                "  ⎿  Updated {} with {} addition{} and {} removal{}\n",
                diff.file_path,
                diff.additions,
                if diff.additions == 1 { "" } else { "s" },
                diff.removals,
                if diff.removals == 1 { "" } else { "s" }
            ));

            // Show diff lines
            for line in &diff.lines {
                let (prefix, content) = match line.change_type {
                    ChangeType::Context => ("     ", &line.content),
                    ChangeType::Addition => ("    +", &line.content),
                    ChangeType::Removal => ("    -", &line.content),
                };

                output.push_str(&format!("    {:4}{} {}\n", line.line_number, prefix, content));
            }
        } else if let Some(details) = &self.details {
            output.push_str(&format!("  ⎿  {}\n", details));
        }

        output
    }
}

pub fn create_diff(
    file_path: impl Into<String>,
    old_content: &str,
    new_content: &str,
    context_lines: usize,
) -> FileDiff {
    use similar::{ChangeTag, TextDiff};

    let file_path = file_path.into();
    let diff = TextDiff::from_lines(old_content, new_content);

    let mut additions = 0;
    let mut removals = 0;
    let mut lines = Vec::new();

    let mut current_line = 1;

    for change in diff.iter_all_changes() {
        let change_type = match change.tag() {
            ChangeTag::Delete => {
                removals += 1;
                ChangeType::Removal
            }
            ChangeTag::Insert => {
                additions += 1;
                ChangeType::Addition
            }
            ChangeTag::Equal => ChangeType::Context,
        };

        // Remove trailing newline from content
        let content = change.value().trim_end_matches('\n').to_string();

        lines.push(DiffLine {
            line_number: current_line,
            change_type,
            content,
        });

        if !matches!(change.tag(), ChangeTag::Delete) {
            current_line += 1;
        }
    }

    // Only keep context lines around changes
    let filtered_lines = filter_context_lines(lines, context_lines);

    FileDiff {
        file_path,
        additions,
        removals,
        lines: filtered_lines,
    }
}

fn filter_context_lines(lines: Vec<DiffLine>, context: usize) -> Vec<DiffLine> {
    if lines.is_empty() {
        return lines;
    }

    let mut result: Vec<DiffLine> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if !matches!(lines[i].change_type, ChangeType::Context) {
            // Found a change, include context before
            let start = i.saturating_sub(context);

            // Add context before if not already added
            for j in start..i {
                if result.is_empty() || result.last().unwrap().line_number != lines[j].line_number {
                    result.push(lines[j].clone());
                }
            }

            // Add the change
            result.push(lines[i].clone());

            // Find end of changes
            let mut end = i + 1;
            while end < lines.len() && !matches!(lines[end].change_type, ChangeType::Context) {
                result.push(lines[end].clone());
                end += 1;
            }

            // Add context after
            let context_end = (end + context).min(lines.len());
            for j in end..context_end {
                result.push(lines[j].clone());
            }

            i = end;
        } else {
            i += 1;
        }
    }

    result
}
