pub mod panes;
pub mod registry;
pub mod terminals;
pub mod ui;

pub use panes::{PaneNode, PaneRect, PaneTree, SplitAxis};
pub use registry::{Agent, AgentRegistry, AgentStatusSummary, Event, EventLevelSummary};
pub use terminals::{TerminalManager, TerminalSnapshot, TerminalState};
pub use ui::{AgentFilterMode, UiState, ViewMode};

use serde::Serialize;
use std::collections::VecDeque;
use crate::terminal::TerminalSize;

#[derive(Debug, Clone, Serialize)]
pub struct WebSnapshot {
    pub agents: Vec<Agent>,
    pub events: Vec<Event>,
    pub terminals: Vec<TerminalSnapshot>,
    pub total_events_received: u64,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

pub struct AppState {
    pub registry: AgentRegistry,
    pub terminals: TerminalManager,
    pub ui: UiState,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::new_with_sessions(1)
    }

    pub fn new_with_sessions(count: usize) -> Self {
        let terminals = TerminalManager::new(count);
        let selected_terminal_idx = terminals.active_session_id();
        Self {
            registry: AgentRegistry::new(),
            terminals,
            ui: UiState {
                selected_terminal_idx,
                ..UiState::new()
            },
        }
    }

    pub fn web_snapshot(&self) -> WebSnapshot {
        WebSnapshot {
            agents: self.registry.agents.values().cloned().collect(),
            events: self.registry.events.iter().take(20).cloned().collect(),
            terminals: self
                .terminals
                .sessions
                .iter()
                .enumerate()
                .map(|(id, session)| {
                    let len = session.lines.len();
                    let recent_lines = session
                        .lines
                        .iter()
                        .skip(len.saturating_sub(50))
                        .cloned()
                        .collect();
                    TerminalSnapshot {
                        id,
                        title: session.title.clone(),
                        cwd: session.cwd.clone(),
                        status: session.status.clone(),
                        recent_lines,
                    }
                })
                .collect(),
            total_events_received: self.registry.total_events_received,
            timestamp: chrono::Local::now(),
        }
    }

    pub fn add_event(&mut self, event: Event) {
        self.registry.add_event(event);
    }

    pub fn update_agent(&mut self, agent: Agent) {
        self.registry.update_agent(agent);
        self.clamp_selection();
    }

    pub fn get_recent_events(&self, agent_id: Option<&str>, limit: usize) -> Vec<&Event> {
        self.registry
            .events
            .iter()
            .filter(|event| agent_id.is_none_or(|id| event.agent_id == id))
            .take(limit)
            .collect()
    }

    pub fn get_selected_agent(&self) -> Option<&Agent> {
        self.get_selected_agent_id()
            .and_then(|id| self.registry.agents.get(&id))
    }

    pub fn visible_agent_ids(&self) -> Vec<String> {
        self.registry
            .agents
            .iter()
            .filter(|(_, agent)| self.matches_filter(agent) && self.matches_search(agent))
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn visible_agent_count(&self) -> usize {
        self.visible_agent_ids().len()
    }

    pub fn visible_agents_page(&self, page_size: usize) -> Vec<String> {
        if page_size == 0 {
            return Vec::new();
        }

        let ids = self.visible_agent_ids();
        let page = self.current_grid_page(page_size);
        let start = page * page_size;
        ids.into_iter().skip(start).take(page_size).collect()
    }

    pub fn current_grid_page(&self, page_size: usize) -> usize {
        if page_size == 0 {
            0
        } else {
            self.ui.selected_agent_idx / page_size
        }
    }

    pub fn grid_page_count(&self, page_size: usize) -> usize {
        let total = self.visible_agent_count();
        if total == 0 || page_size == 0 {
            1
        } else {
            total.div_ceil(page_size)
        }
    }

    pub fn filter_label(&self) -> &'static str {
        self.ui.filter_label()
    }

    pub fn cycle_filter_mode(&mut self) {
        self.ui.cycle_filter_mode();
        self.clamp_selection();
    }

    pub fn begin_search(&mut self) {
        self.ui.begin_search();
    }

    pub fn end_search(&mut self) {
        self.ui.end_search();
    }

    pub fn clear_search_query(&mut self) {
        self.ui.clear_search_query();
        self.clamp_selection();
    }

    pub fn set_search_query(&mut self, query: &str) {
        self.ui.search_query = query.to_string();
        self.clamp_selection();
    }

    pub fn append_terminal_line(&mut self, session_id: usize, line: impl Into<String>) {
        self.terminals.append_line(session_id, line);
    }

    pub fn clear_terminal_lines(&mut self, session_id: usize) {
        self.terminals.clear_lines(session_id);
    }

    pub fn append_terminal_context_line(&mut self, session_id: usize, line: String) {
        self.terminals.append_context_line(session_id, line);
    }

    pub fn finalize_terminal_context_line(&mut self, session_id: usize, seconds: f64) {
        self.terminals.finalize_context_line(session_id, seconds);
    }

    pub fn set_terminal_status(&mut self, session_id: usize, status: impl Into<String>) {
        if let Some(session) = self.terminals.sessions.get_mut(session_id) {
            session.status = status.into();
        }
    }

    pub fn set_terminal_last_command(&mut self, session_id: usize, command: String) {
        if let Some(session) = self.terminals.sessions.get_mut(session_id) {
            session.last_command = Some(command);
        }
    }

    pub fn set_terminal_last_exit_code(&mut self, session_id: usize, exit_code: i32) {
        if let Some(session) = self.terminals.sessions.get_mut(session_id) {
            session.last_exit_code = Some(exit_code);
        }
    }

    pub fn set_terminal_viewport_size(&mut self, session_id: usize, size: TerminalSize) -> bool {
        self.terminals.set_viewport_size(session_id, size)
    }

    pub fn set_terminal_cwd(&mut self, session_id: usize, cwd: impl Into<String>) -> bool {
        let Some(session) = self.terminals.sessions.get_mut(session_id) else {
            return false;
        };
        let cwd = cwd.into();
        if session.cwd == cwd {
            return false;
        }
        session.cwd = cwd;
        true
    }

    pub fn recent_terminal_lines(&self, session_id: usize, limit: usize) -> Vec<&str> {
        let Some(session) = self.terminals.sessions.get(session_id) else {
            return Vec::new();
        };
        let len = session.lines.len();
        session
            .lines
            .iter()
            .skip(len.saturating_sub(limit))
            .map(String::as_str)
            .collect()
    }

    pub fn append_terminal_history(&mut self, session_id: usize, command: String) {
        self.terminals.append_history(session_id, command);
    }

    pub fn seed_terminal_history<I>(&mut self, history: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.terminals.seed_history(history);
    }

    pub fn seed_directory_history<I>(&mut self, history: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.terminals.seed_directory_history(history);
    }

    pub fn record_directory_visit(&mut self, path: String) {
        self.terminals.record_directory_visit(path);
    }

    pub fn terminal_directory_history(&self) -> &VecDeque<String> {
        &self.terminals.directory_history
    }

    pub fn get_terminal_suggestion(&self, session_id: usize, input: &str) -> Option<String> {
        self.terminals.get_suggestion(session_id, input)
    }

    pub fn terminal_sessions(&self) -> &[TerminalState] {
        &self.terminals.sessions
    }

    pub fn add_terminal_session(&mut self, title: impl Into<String>) -> Option<usize> {
        let index = self.terminals.add_session(title)?;
        self.ui.selected_terminal_idx = self.terminals.active_session_id();
        Some(index)
    }

    pub fn split_selected_terminal(
        &mut self,
        title: impl Into<String>,
        axis: SplitAxis,
    ) -> Option<usize> {
        let index = self.terminals.add_session_split(title, axis)?;
        self.ui.selected_terminal_idx = self.terminals.active_session_id();
        Some(index)
    }

    pub fn remove_terminal_session(&mut self, index: usize) -> bool {
        if self.terminals.remove_session(index) {
            self.ui.selected_terminal_idx = self.terminals.active_session_id();
            true
        } else {
            false
        }
    }

    pub fn selected_terminal(&self) -> Option<&TerminalState> {
        self.terminals.sessions.get(self.terminals.active_session_id())
    }

    pub fn select_terminal_index(&mut self, index: usize) {
        if self.terminals.sessions.is_empty() {
            self.ui.selected_terminal_idx = 0;
        } else {
            let index = index.min(self.terminals.sessions.len() - 1);
            let _ = self.terminals.select_session(index);
            self.ui.selected_terminal_idx = self.terminals.active_session_id();
            self.ui.view_mode = ViewMode::Focus;
        }
    }

    pub fn focus_next_terminal(&mut self) -> bool {
        if self.terminals.focus_next_session() {
            self.ui.selected_terminal_idx = self.terminals.active_session_id();
            true
        } else {
            false
        }
    }

    pub fn focus_previous_terminal(&mut self) -> bool {
        if self.terminals.focus_previous_session() {
            self.ui.selected_terminal_idx = self.terminals.active_session_id();
            true
        } else {
            false
        }
    }

    pub fn terminal_pane_layout(&self, rect: PaneRect, gap: f32) -> Vec<(usize, PaneRect)> {
        self.terminals.pane_layout(rect, gap)
    }

    pub fn terminal_pane_layout_equal(&self, rect: PaneRect, gap: f32) -> Vec<(usize, PaneRect)> {
        self.terminals.pane_tree.layout_equal(rect, gap)
    }


    pub fn terminal_pane_tree(&self) -> &PaneTree {
        &self.terminals.pane_tree
    }

    pub fn select_visible_index(&mut self, index: usize) {
        self.ui.selected_agent_idx = index;
        self.clamp_selection();
    }

    pub fn toggle_view_mode(&mut self) {
        self.ui.toggle_view_mode();
    }

    pub fn append_search_char(&mut self, ch: char) {
        self.ui.append_search_char(ch);
        self.clamp_selection();
    }

    pub fn pop_search_char(&mut self) {
        self.ui.pop_search_char();
        self.clamp_selection();
    }

    pub fn get_agent_status_summary(&self) -> AgentStatusSummary {
        self.registry.get_status_summary()
    }

    pub fn get_event_level_summary(&self) -> EventLevelSummary {
        self.registry.get_event_summary()
    }

    pub fn tick_activity(&mut self) {
        self.registry.tick_activity();
    }

    pub fn select_next(&mut self) {
        let count = self.visible_agent_count();
        if count == 0 {
            self.ui.selected_agent_idx = 0;
            return;
        }
        self.ui.selected_agent_idx = (self.ui.selected_agent_idx + 1) % count;
    }

    pub fn select_previous(&mut self) {
        let count = self.visible_agent_count();
        if count == 0 {
            self.ui.selected_agent_idx = 0;
            return;
        }
        if self.ui.selected_agent_idx == 0 {
            self.ui.selected_agent_idx = count - 1;
        } else {
            self.ui.selected_agent_idx -= 1;
        }
    }

    pub fn select_first(&mut self) {
        self.ui.selected_agent_idx = 0;
    }

    pub fn select_last(&mut self) {
        let count = self.visible_agent_count();
        if count != 0 {
            self.ui.selected_agent_idx = count - 1;
        }
    }

    pub fn select_next_page(&mut self) {
        let count = self.visible_agent_count();
        if count != 0 {
            self.ui.selected_agent_idx = (self.ui.selected_agent_idx + 5).min(count - 1);
        }
    }

    pub fn select_previous_page(&mut self) {
        if self.visible_agent_count() != 0 {
            self.ui.selected_agent_idx = self.ui.selected_agent_idx.saturating_sub(5);
        }
    }

    pub fn get_selected_agent_id(&self) -> Option<String> {
        self.visible_agent_ids()
            .get(self.ui.selected_agent_idx)
            .cloned()
    }

    fn matches_filter(&self, agent: &Agent) -> bool {
        match self.ui.filter_mode {
            AgentFilterMode::All => true,
            AgentFilterMode::Busy => agent.status.eq_ignore_ascii_case("busy"),
            AgentFilterMode::Active => !agent.status.eq_ignore_ascii_case("offline"),
            AgentFilterMode::Offline => agent.status.eq_ignore_ascii_case("offline"),
        }
    }

    fn matches_search(&self, agent: &Agent) -> bool {
        if self.ui.search_query.trim().is_empty() {
            return true;
        }

        let query = self.ui.search_query.to_ascii_lowercase();
        let haystacks = [
            agent.id.as_str(),
            agent.project.as_str(),
            agent.role.as_str(),
            agent.branch.as_str(),
            agent.instance_name.as_str(),
        ];

        haystacks
            .iter()
            .any(|candidate| candidate.to_ascii_lowercase().contains(&query))
    }

    fn clamp_selection(&mut self) {
        let count = self.visible_agent_count();
        if count == 0 {
            self.ui.selected_agent_idx = 0;
        } else if self.ui.selected_agent_idx >= count {
            self.ui.selected_agent_idx = count - 1;
        }
    }
}
