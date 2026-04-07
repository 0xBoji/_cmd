use std::collections::{BTreeMap, VecDeque};
use serde::{Deserialize, Serialize};

/// Maximum number of events to retain in the buffer.
const EVENT_LIMIT: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Busy,
    Offline,
}

impl AgentStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Idle => "Idle",
            Self::Busy => "Busy",
            Self::Offline => "Offline",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub role: String,
    pub status: AgentStatus,
    pub git_locked: bool,
    pub last_seen: chrono::DateTime<chrono::Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub agent_id: String,
    pub kind: String,
    pub payload: String,
}

pub struct AppState {
    pub agents: BTreeMap<String, Agent>,
    pub events: VecDeque<Event>,
    pub selected_agent_idx: usize,
    pub should_quit: bool,
    pub total_events_received: u64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            agents: BTreeMap::new(),
            events: VecDeque::with_capacity(EVENT_LIMIT),
            selected_agent_idx: 0,
            should_quit: false,
            total_events_received: 0,
        }
    }

    pub fn add_event(&mut self, event: Event) {
        self.total_events_received += 1;
        if self.events.len() >= EVENT_LIMIT {
            self.events.pop_back();
        }
        self.events.push_front(event);
    }

    pub fn update_agent(&mut self, agent: Agent) {
        self.agents.insert(agent.id.clone(), agent);
    }

    pub fn select_next(&mut self) {
        if self.agents.is_empty() {
            self.selected_agent_idx = 0;
            return;
        }
        self.selected_agent_idx = (self.selected_agent_idx + 1) % self.agents.len();
    }

    pub fn select_previous(&mut self) {
        if self.agents.is_empty() {
            self.selected_agent_idx = 0;
            return;
        }
        if self.selected_agent_idx == 0 {
            self.selected_agent_idx = self.agents.len() - 1;
        } else {
            self.selected_agent_idx -= 1;
        }
    }

    pub fn select_first(&mut self) {
        self.selected_agent_idx = 0;
    }

    pub fn select_last(&mut self) {
        if !self.agents.is_empty() {
            self.selected_agent_idx = self.agents.len() - 1;
        }
    }

    pub fn select_next_page(&mut self) {
        if !self.agents.is_empty() {
            self.selected_agent_idx = (self.selected_agent_idx + 5).min(self.agents.len() - 1);
        }
    }

    pub fn select_previous_page(&mut self) {
        if !self.agents.is_empty() {
            self.selected_agent_idx = self.selected_agent_idx.saturating_sub(5);
        }
    }

    pub fn get_selected_agent_id(&self) -> Option<String> {
        self.agents.keys().nth(self.selected_agent_idx).cloned()
    }
}
