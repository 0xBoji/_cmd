use std::collections::{BTreeMap, VecDeque};
use serde::{Deserialize, Serialize};

/// Maximum number of events to retain in the buffer.
const EVENT_LIMIT: usize = 100;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub instance_name: String,
    pub role: String,
    pub project: String,
    pub branch: String,
    pub status: String,
    pub capabilities: Vec<String>,
    pub port: u16,
    pub addresses: Vec<String>,
    pub metadata: BTreeMap<String, String>,
    pub last_seen: chrono::DateTime<chrono::Local>,
    pub tokens: u64,
    pub activity: VecDeque<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub agent_id: String,
    pub kind: String,
    pub component: String,
    pub level: String,
    pub payload: String,
}

/// Matches CAMP's JSON EventRecord
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub kind: String,
    pub origin: String,
    pub reason: Option<String>,
    pub previous: Option<Agent>,
    pub current: Option<Agent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotRecord {
    pub kind: String,
    pub agents: Vec<Agent>,
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
        if let Some(agent) = self.agents.get_mut(&event.agent_id) {
            if let Some(last) = agent.activity.back_mut() {
                *last += 1;
            }
        }
        if self.events.len() >= EVENT_LIMIT {
            self.events.pop_back();
        }
        self.events.push_front(event);
    }

    pub fn update_agent(&mut self, mut agent: Agent) {
        // Extract and parse tokens from metadata if present
        if let Some(tokens_str) = agent.metadata.get("tokens") {
            if let Ok(tokens) = tokens_str.replace(",", "").parse::<u64>() {
                agent.tokens = tokens;
            }
        }

        if let Some(existing) = self.agents.get(&agent.id) {
            agent.activity = existing.activity.clone();
            // Preserve tokens if the incoming one is zero but we had one before
            if agent.tokens == 0 && existing.tokens > 0 {
                agent.tokens = existing.tokens;
            }
        } else if agent.activity.is_empty() {
            // Initialize empty activity buffer for new agent (50 points)
            agent.activity = VecDeque::from(vec![0; 50]);
        }
        agent.last_seen = chrono::Local::now();
        self.agents.insert(agent.id.clone(), agent);
    }

    pub fn get_events_for_agent(&self, agent_id: &str) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.agent_id == agent_id)
            .collect()
    }

    /// Ticks the activity buffers, shifting them to the left.
    /// Should be called on a fixed interval (e.g. 1s).
    pub fn tick_activity(&mut self) {
        for agent in self.agents.values_mut() {
            if agent.activity.len() >= 50 {
                agent.activity.pop_front();
            }
            agent.activity.push_back(0);
        }
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
