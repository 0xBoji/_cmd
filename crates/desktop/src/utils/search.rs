use crate::transcript::{is_context_block_start, is_legacy_context_block_start};
use std::collections::BTreeSet;

pub struct TranscriptSearchMatch {
    pub block_index: Option<usize>,
    pub line_index: usize,
    pub range: std::ops::Range<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TranscriptBlock {
    pub start: usize,
    pub end: usize,
    pub has_error: bool,
}

pub fn terminal_transcript_blocks(lines: &[&str]) -> Vec<TranscriptBlock> {
    use crate::transcript::command_block_has_error;
    let mut blocks = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let line = lines[index];
        let has_context_line = is_context_block_start(lines, index);
        if has_context_line || line.starts_with("$ ") {
            let prompt_index = if has_context_line { index + 1 } else { index };
            let mut block_end = prompt_index + 1;
            while block_end < lines.len()
                && !lines[block_end].starts_with("$ ")
                && !is_context_block_start(lines, block_end)
                && !is_legacy_context_block_start(lines, block_end)
            {
                block_end += 1;
            }

            blocks.push(TranscriptBlock {
                start: index,
                end: block_end,
                has_error: command_block_has_error(lines, prompt_index),
            });
            index = block_end;
            continue;
        }

        index += 1;
    }

    blocks
}

pub fn find_match_ranges(line: &str, query: &str, case_sensitive: bool) -> Vec<std::ops::Range<usize>> {
    if query.is_empty() {
        return Vec::new();
    }

    if case_sensitive {
        return line
            .match_indices(query)
            .map(|(start, matched)| start..start + matched.len())
            .collect();
    }

    let lowercase_line = line.to_lowercase();
    let lowercase_query = query.to_lowercase();
    lowercase_line
        .match_indices(&lowercase_query)
        .map(|(start, _)| start..start + lowercase_query.len())
        .collect()
}

pub fn transcript_line_block_indexes(lines: &[&str]) -> Vec<Option<usize>> {
    let mut indexes = vec![None; lines.len()];
    let mut index = 0usize;
    let mut block_index = 0usize;
    while index < lines.len() {
        let line = lines[index];
        let has_context_line = is_context_block_start(lines, index);
        if has_context_line || line.starts_with("$ ") {
            let prompt_index = if has_context_line { index + 1 } else { index };
            let mut block_end = prompt_index + 1;
            while block_end < lines.len()
                && !lines[block_end].starts_with("$ ")
                && !is_context_block_start(lines, block_end)
                && !is_legacy_context_block_start(lines, block_end)
            {
                block_end += 1;
            }
            for slot in &mut indexes[index..block_end] {
                *slot = Some(block_index);
            }
            block_index += 1;
            index = block_end;
            while index < lines.len() && lines[index].trim().is_empty() {
                index += 1;
            }
            continue;
        }

        index += 1;
    }

    indexes
}

pub fn transcript_search_matches(
    lines: &[&str],
    selected_blocks: &BTreeSet<usize>,
    query: &str,
    case_sensitive: bool,
) -> Vec<TranscriptSearchMatch> {
    let block_indexes = transcript_line_block_indexes(lines);
    lines
        .iter()
        .enumerate()
        .filter(|(line_index, _)| {
            selected_blocks.is_empty()
                || block_indexes[*line_index]
                    .is_some_and(|block_index| selected_blocks.contains(&block_index))
        })
        .flat_map(|(line_index, line)| {
            let block_index = block_indexes[line_index];
            find_match_ranges(line, query, case_sensitive)
                .into_iter()
                .map(move |range| TranscriptSearchMatch {
                    block_index,
                    line_index,
                    range,
                })
        })
        .collect()
}

pub fn selected_search_scope(
    selected_blocks: &BTreeSet<usize>,
    selected_only: bool,
) -> BTreeSet<usize> {
    if selected_only {
        selected_blocks.clone()
    } else {
        BTreeSet::new()
    }
}

pub fn next_search_index(
    current_index: Option<usize>,
    result_count: usize,
    backwards: bool,
) -> Option<usize> {
    if result_count == 0 {
        return None;
    }

    Some(match (current_index, backwards) {
        (Some(0), true) | (None, true) => result_count - 1,
        (Some(index), true) => index.saturating_sub(1),
        (Some(index), false) => (index + 1) % result_count,
        (None, false) => 0,
    })
}
