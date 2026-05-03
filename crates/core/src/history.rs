use crate::app::terminals::{DIRECTORY_HISTORY_LIMIT, HISTORY_LIMIT};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const HISTORY_DIR: &str = ".view";
const HISTORY_FILE: &str = "shell-history.jsonl";
const RAW_HISTORY_LIMIT: usize = 2_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub command: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub timestamp_unix_ms: Option<u64>,
}

pub fn load_history() -> io::Result<VecDeque<String>> {
    match history_path() {
        Some(path) => load_history_from_path(&path),
        None => Ok(VecDeque::new()),
    }
}

pub fn load_history_from_path(path: &Path) -> io::Result<VecDeque<String>> {
    let entries = load_entries_from_path(path)?;
    let mut history = VecDeque::new();
    for entry in entries {
        push_recent_unique(&mut history, entry.command);
    }

    Ok(history)
}

pub fn append_history_entry(command: &str) -> io::Result<()> {
    append_history_entry_with_cwd(command, None)
}

pub fn append_history_entry_with_cwd(command: &str, cwd: Option<&str>) -> io::Result<()> {
    match history_path() {
        Some(path) => append_history_entry_to_path(&path, command, cwd),
        None => Ok(()),
    }
}

pub fn append_history_entry_to_path(
    path: &Path,
    command: &str,
    cwd: Option<&str>,
) -> io::Result<()> {
    let mut entries = load_entries_from_path(path)?;
    entries.push(HistoryEntry {
        command: command.to_string(),
        cwd: cwd.map(ToOwned::to_owned),
        timestamp_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_millis() as u64),
    });
    if entries.len() > RAW_HISTORY_LIMIT {
        let start = entries.len() - RAW_HISTORY_LIMIT;
        entries.drain(..start);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut lines = String::new();
    for entry in entries {
        let line = serde_json::to_string(&entry)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        lines.push_str(&line);
        lines.push('\n');
    }

    fs::write(path, lines)
}

fn history_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("VIEW_HISTORY_FILE") {
        return Some(PathBuf::from(path));
    }

    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(HISTORY_DIR).join(HISTORY_FILE))
}

pub fn load_entries() -> io::Result<Vec<HistoryEntry>> {
    match history_path() {
        Some(path) => load_entries_from_path(&path),
        None => Ok(Vec::new()),
    }
}

pub fn load_entries_from_path(path: &Path) -> io::Result<Vec<HistoryEntry>> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err),
    };

    let mut entries = Vec::new();
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(entry) = serde_json::from_str::<HistoryEntry>(line) else {
            continue;
        };
        if !entry.command.trim().is_empty() {
            entries.push(entry);
        }
    }

    Ok(entries)
}

pub fn directory_jump_history_from_entries(entries: &[HistoryEntry]) -> VecDeque<String> {
    let mut jumps = VecDeque::new();
    for entry in entries {
        if let Some(path) = resolved_directory_jump(entry) {
            if jumps.len() >= DIRECTORY_HISTORY_LIMIT {
                jumps.pop_front();
            }
            jumps.push_back(path);
        }
    }
    jumps
}

pub fn best_directory_jump_match(
    directory_history: &VecDeque<String>,
    query: &str,
) -> Option<String> {
    let query = query.trim();
    if query.is_empty() {
        return None;
    }

    #[derive(Clone, Copy)]
    struct Score {
        frequency: usize,
        last_seen_index: usize,
        basename_prefix: bool,
        basename_contains: bool,
        path_contains: bool,
    }

    let mut scores: HashMap<&str, Score> = HashMap::new();
    for (index, path) in directory_history.iter().enumerate() {
        let basename = Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let basename_prefix = basename.starts_with(query);
        let basename_contains = basename.contains(query);
        let path_contains = path.contains(query);
        if !(basename_prefix || basename_contains || path_contains) {
            continue;
        }

        let entry = scores.entry(path.as_str()).or_insert(Score {
            frequency: 0,
            last_seen_index: index,
            basename_prefix,
            basename_contains,
            path_contains,
        });
        entry.frequency += 1;
        entry.last_seen_index = index;
        entry.basename_prefix |= basename_prefix;
        entry.basename_contains |= basename_contains;
        entry.path_contains |= path_contains;
    }

    scores
        .into_iter()
        .max_by(|(left_path, left), (right_path, right)| {
            left.basename_prefix
                .cmp(&right.basename_prefix)
                .then(left.basename_contains.cmp(&right.basename_contains))
                .then(left.path_contains.cmp(&right.path_contains))
                .then(left.frequency.cmp(&right.frequency))
                .then(left.last_seen_index.cmp(&right.last_seen_index))
                .then_with(|| left_path.cmp(right_path))
        })
        .map(|(path, _)| path.to_string())
}

fn push_recent_unique(history: &mut VecDeque<String>, command: String) {
    if command.trim().is_empty() {
        return;
    }

    history.retain(|existing| existing != &command);
    if history.len() >= HISTORY_LIMIT {
        history.pop_front();
    }
    history.push_back(command);
}

fn resolved_directory_jump(entry: &HistoryEntry) -> Option<String> {
    let target = cd_target_from_command(&entry.command)?;
    resolve_cd_target(entry.cwd.as_deref(), target)
}

fn cd_target_from_command(command: &str) -> Option<&str> {
    let target = command.strip_prefix("cd ")?;
    let trimmed = target.trim();
    if trimmed.is_empty() || trimmed == "-" {
        None
    } else {
        Some(trimmed)
    }
}

fn resolve_cd_target(cwd: Option<&str>, target: &str) -> Option<String> {
    let target = target.strip_prefix("builtin ").unwrap_or(target);
    let target = strip_matching_quotes(target.trim());
    if target.is_empty() {
        return None;
    }

    if let Some(home_relative) = target.strip_prefix("~/") {
        let home = std::env::var("HOME").ok()?;
        return Some(normalize_path(Path::new(&home).join(home_relative)));
    }

    if target == "~" {
        let home = std::env::var("HOME").ok()?;
        return Some(normalize_path(PathBuf::from(home)));
    }

    let path = Path::new(target);
    if path.is_absolute() {
        return Some(normalize_path(path.to_path_buf()));
    }

    let cwd = cwd?;
    Some(normalize_path(Path::new(cwd).join(path)))
}

fn strip_matching_quotes(value: &str) -> &str {
    if value.len() >= 2 {
        let first = value.as_bytes()[0];
        let last = value.as_bytes()[value.len() - 1];
        if (first == b'\'' && last == b'\'') || (first == b'"' && last == b'"') {
            return &value[1..value.len() - 1];
        }
    }
    value
}

fn normalize_path(path: PathBuf) -> String {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        use std::path::Component;

        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized.display().to_string()
}
