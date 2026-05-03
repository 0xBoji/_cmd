//! Terminal transcript rendering for VIEW Desktop.
//!
//! Handles the scrollable output area: command blocks, error highlighting,
//! context lines (cwd + git), and block separators. Kept separate from
//! input handling and shell plumbing so each can evolve independently.

// Local color constants (removed as they are no longer used by the inlined render logic)

// ── Helpers re-exported for tests ─────────────────────────────────────────────

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRow {
    pub logical_line_index: usize,
    pub start_char: usize,
    pub end_char: usize,
}

#[cfg(test)]
fn char_range_slice(line: &str, start_char: usize, end_char: usize) -> String {
    line.chars()
        .skip(start_char)
        .take(end_char.saturating_sub(start_char))
        .collect()
}

pub fn wrap_columns_for_width(width: f32, char_width: f32) -> usize {
    ((width / char_width).floor() as usize).max(1)
}

pub fn wrapped_row_count(line: &str, cols: usize) -> usize {
    let char_count = line.chars().count();
    char_count.max(1).div_ceil(cols.max(1))
}

#[cfg(test)]
pub fn build_physical_rows(lines: &[&str], cols: usize) -> Vec<PhysicalRow> {
    let cols = cols.max(1);
    let mut rows = Vec::new();

    for (logical_line_index, line) in lines.iter().enumerate() {
        if line.is_empty() {
            rows.push(PhysicalRow {
                logical_line_index,
                start_char: 0,
                end_char: 0,
            });
            continue;
        }

        let mut start_char = 0usize;
        let char_count = line.chars().count();
        while start_char < char_count {
            let end_char = (start_char + cols).min(char_count);
            rows.push(PhysicalRow {
                logical_line_index,
                start_char,
                end_char,
            });
            start_char = end_char;
        }
    }

    rows
}

#[cfg(test)]
pub fn selection_text_from_physical_rows(
    lines: &[&str],
    rows: &[PhysicalRow],
    selected_rows: std::ops::RangeInclusive<usize>,
) -> String {
    let mut output = String::new();
    let mut previous_logical_line = None;

    for row_index in selected_rows {
        let Some(row) = rows.get(row_index) else {
            continue;
        };
        let Some(line) = lines.get(row.logical_line_index) else {
            continue;
        };

        if let Some(previous) = previous_logical_line {
            if previous != row.logical_line_index {
                output.push('\n');
            }
        }

        output.push_str(&char_range_slice(line, row.start_char, row.end_char));
        previous_logical_line = Some(row.logical_line_index);
    }

    output
}

pub fn command_clears_transcript(command: &str) -> bool {
    matches!(command.trim(), "clear" | "cls")
}

pub fn is_error_output_line(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("command not found")
        || lower.contains("no such file or directory")
        || lower.contains("error:")
        || lower.contains("permission denied")
        || lower.starts_with("zsh: ")
}

pub fn command_block_has_error(lines: &[&str], prompt_index: usize) -> bool {
    lines
        .iter()
        .skip(prompt_index + 1)
        .take_while(|line| !line.starts_with("$ "))
        .any(|line| is_error_output_line(line))
}

pub fn should_render_block_separator(
    _previous_block_had_error: bool,
    _current_block_has_error: bool,
) -> bool {
    true
}

pub fn should_extend_error_block_to_bottom(has_error: bool, is_last_block: bool) -> bool {
    has_error && is_last_block
}

pub fn is_command_context_line(line: &str) -> bool {
    line.starts_with('/')
}

pub fn is_context_block_start(lines: &[&str], index: usize) -> bool {
    lines
        .get(index)
        .is_some_and(|line| is_command_context_line(line))
        && lines
            .get(index + 1)
            .is_some_and(|next| next.starts_with("$ "))
}

pub fn is_legacy_context_block_start(lines: &[&str], index: usize) -> bool {
    lines.get(index).is_some_and(|line| line.trim().is_empty())
        && lines
            .get(index + 1)
            .is_some_and(|line| is_command_context_line(line))
        && lines
            .get(index + 2)
            .is_some_and(|next| next.starts_with("$ "))
}
