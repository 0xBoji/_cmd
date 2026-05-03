use crate::app::{PaneRect, PaneTree, SplitAxis};
use crate::terminal::TerminalSize;
use serde::Serialize;
use std::collections::VecDeque;

pub const TERMINAL_LINE_LIMIT: usize = 400;
pub const MAX_TERMINAL_SESSIONS: usize = 32;
pub const HISTORY_LIMIT: usize = 50;
pub const DIRECTORY_HISTORY_LIMIT: usize = 256;

#[derive(Clone)]
pub struct TerminalState {
    pub title: String,
    pub cwd: String,
    pub status: String,
    pub last_command: Option<String>,
    pub last_exit_code: Option<i32>,
    pub viewport_size: Option<TerminalSize>,
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
            last_command: None,
            last_exit_code: None,
            viewport_size: None,
            lines: VecDeque::new(),
            history: VecDeque::new(),
            pending_context_line: None,
        }
    }
}

pub struct TerminalManager {
    pub sessions: Vec<TerminalState>,
    pub directory_history: VecDeque<String>,
    pub pane_tree: PaneTree,
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new(1)
    }
}

impl TerminalManager {
    pub fn new(count: usize) -> Self {
        let count = count.clamp(1, MAX_TERMINAL_SESSIONS);
        let sessions = (0..count)
            .map(|index| TerminalState {
                title: format!("shell-{}", index + 1),
                ..TerminalState::default()
            })
            .collect();
        let mut pane_tree = PaneTree::new(0);
        for session_id in 1..count {
            let _ = pane_tree.split_active(SplitAxis::Vertical, session_id);
        }

        Self {
            sessions,
            directory_history: VecDeque::new(),
            pane_tree,
        }
    }

    pub fn add_session(&mut self, title: impl Into<String>) -> Option<usize> {
        self.add_session_split(title, SplitAxis::Vertical)
    }

    pub fn add_session_split(
        &mut self,
        title: impl Into<String>,
        axis: SplitAxis,
    ) -> Option<usize> {
        if self.sessions.len() >= MAX_TERMINAL_SESSIONS {
            return None;
        }
        let index = self.sessions.len();
        let history = self
            .sessions
            .first()
            .map(|session| session.history.clone())
            .unwrap_or_default();
        self.sessions.push(TerminalState {
            title: title.into(),
            history,
            ..TerminalState::default()
        });
        let _ = self.pane_tree.split_active(axis, index);
        Some(index)
    }

    pub fn remove_session(&mut self, index: usize) -> bool {
        if self.sessions.len() <= 1 || index >= self.sessions.len() {
            return false;
        }
        if !self.pane_tree.remove_session(index) {
            return false;
        }
        self.sessions.remove(index);
        true
    }

    pub fn select_session(&mut self, index: usize) -> bool {
        self.pane_tree.set_active_session(index)
    }

    pub fn focus_next_session(&mut self) -> bool {
        self.pane_tree.focus_next()
    }

    pub fn focus_previous_session(&mut self) -> bool {
        self.pane_tree.focus_previous()
    }

    pub fn active_session_id(&self) -> usize {
        self.pane_tree.active_session_id()
    }

    pub fn pane_layout(&self, rect: PaneRect, gap: f32) -> Vec<(usize, PaneRect)> {
        self.pane_tree.layout(rect, gap)
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
            if session.history.len() >= HISTORY_LIMIT {
                session.history.pop_front();
            }
            session.history.push_back(command);
        }
    }

    pub fn set_viewport_size(&mut self, session_id: usize, size: TerminalSize) -> bool {
        let Some(session) = self.sessions.get_mut(session_id) else {
            return false;
        };
        if session.viewport_size == Some(size) {
            return false;
        }
        session.viewport_size = Some(size);
        true
    }

    pub fn seed_history<I>(&mut self, history: I)
    where
        I: IntoIterator<Item = String>,
    {
        let seeded = history.into_iter().collect::<VecDeque<_>>();
        for session in &mut self.sessions {
            session.history = seeded.clone();
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

    pub fn seed_directory_history<I>(&mut self, history: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.directory_history = history.into_iter().collect();
    }

    pub fn record_directory_visit(&mut self, path: String) {
        if path.trim().is_empty() {
            return;
        }
        if self.directory_history.len() >= DIRECTORY_HISTORY_LIMIT {
            self.directory_history.pop_front();
        }
        self.directory_history.push_back(path);
    }
}
