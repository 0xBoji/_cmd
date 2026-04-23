use serde::Serialize;
use std::collections::VecDeque;

pub const TERMINAL_LINE_LIMIT: usize = 400;
pub const MAX_TERMINAL_SESSIONS: usize = 32;

#[derive(Clone)]
pub struct TerminalState {
    pub title: String,
    pub cwd: String,
    pub status: String,
    pub lines: VecDeque<String>,
    pub history: VecDeque<String>,
    pub pending_context_line: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TerminalSnapshot {
    pub id: usize,
    pub title: String,
    pub cwd: String,
    pub status: String,
    pub recent_lines: Vec<String>,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            title: "shell-1".to_string(),
            cwd: String::new(),
            status: "starting".to_string(),
            lines: VecDeque::new(),
            history: VecDeque::new(),
            pending_context_line: None,
        }
    }
}

#[derive(Default)]
pub struct TerminalManager {
    pub sessions: Vec<TerminalState>,
}

impl TerminalManager {
    pub fn new(count: usize) -> Self {
        let count = count.max(1).min(MAX_TERMINAL_SESSIONS);
        Self {
            sessions: (0..count)
                .map(|index| TerminalState {
                    title: format!("shell-{}", index + 1),
                    ..TerminalState::default()
                })
                .collect(),
        }
    }

    pub fn add_session(&mut self, title: impl Into<String>) -> Option<usize> {
        if self.sessions.len() >= MAX_TERMINAL_SESSIONS {
            return None;
        }
        let index = self.sessions.len();
        self.sessions.push(TerminalState {
            title: title.into(),
            ..TerminalState::default()
        });
        Some(index)
    }

    pub fn remove_session(&mut self, index: usize) -> bool {
        if self.sessions.len() <= 1 || index >= self.sessions.len() {
            return false;
        }
        self.sessions.remove(index);
        true
    }

    pub fn append_line(&mut self, session_id: usize, line: impl Into<String>) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            if session.lines.len() >= TERMINAL_LINE_LIMIT {
                session.lines.pop_front();
            }
            session.lines.push_back(line.into());
        }
    }

    pub fn clear_lines(&mut self, session_id: usize) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.lines.clear();
            session.pending_context_line = None;
        }
    }

    pub fn append_context_line(&mut self, session_id: usize, line: String) {
        self.append_line(session_id, line.clone());
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.pending_context_line = Some(line);
        }
    }

    pub fn finalize_context_line(&mut self, session_id: usize, seconds: f64) {
        let Some(session) = self.sessions.get_mut(session_id) else {
            return;
        };
        let Some(pending_line) = session.pending_context_line.take() else {
            return;
        };

        if let Some(line) = session
            .lines
            .iter_mut()
            .rev()
            .find(|line| **line == pending_line)
        {
            *line = format!("{pending_line} ({seconds:.4}s)");
        }
    }

    pub fn append_history(&mut self, session_id: usize, command: String) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.history.retain(|c| c != &command);
            if session.history.len() >= 50 {
                session.history.pop_front();
            }
            session.history.push_back(command);
        }
    }

    pub fn get_suggestion(&self, session_id: usize, input: &str) -> Option<String> {
        if input.is_empty() {
            return None;
        }
        let session = self.sessions.get(session_id)?;
        session
            .history
            .iter()
            .rev()
            .find(|cmd| cmd.starts_with(input))
            .cloned()
    }
}
