use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

pub const EVENT_LIMIT: usize = 100;

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AgentStatusSummary {
    pub total: usize,
    pub online: usize,
    pub busy: usize,
    pub offline: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EventLevelSummary {
    pub info: usize,
    pub warn: usize,
    pub error: usize,
    pub success: usize,
}

#[derive(Default)]
pub struct AgentRegistry {
    pub agents: BTreeMap<String, Agent>,
    pub events: VecDeque<Event>,
    pub total_events_received: u64,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: BTreeMap::new(),
            events: VecDeque::with_capacity(EVENT_LIMIT),
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
        if let Some(tokens_str) = agent.metadata.get("tokens") {
            if let Ok(tokens) = tokens_str.replace(",", "").parse::<u64>() {
                agent.tokens = tokens;
            }
        }

        if let Some(existing) = self.agents.get(&agent.id) {
            agent.activity = existing.activity.clone();
            if agent.tokens == 0 && existing.tokens > 0 {
                agent.tokens = existing.tokens;
            }
        } else if agent.activity.is_empty() {
            agent.activity = VecDeque::from(vec![0; 50]);
        }
        agent.last_seen = Local::now();
        self.agents.insert(agent.id.clone(), agent);
    }

    pub fn tick_activity(&mut self) {
        for agent in self.agents.values_mut() {
            if agent.activity.len() >= 50 {
                agent.activity.pop_front();
            }
            agent.activity.push_back(0);
        }
    }

    pub fn get_status_summary(&self) -> AgentStatusSummary {
        let mut summary = AgentStatusSummary {
            total: self.agents.len(),
            ..Default::default()
        };

        for agent in self.agents.values() {
            match agent.status.to_ascii_lowercase().as_str() {
                "busy" => {
                    summary.busy += 1;
                    summary.online += 1;
                }
                "offline" => summary.offline += 1,
                _ => summary.online += 1,
            }
        }
        summary
    }

    pub fn get_event_summary(&self) -> EventLevelSummary {
        let mut summary = EventLevelSummary::default();
        for event in &self.events {
            match event.level.to_ascii_lowercase().as_str() {
                "warn" => summary.warn += 1,
                "error" => summary.error += 1,
                "success" => summary.success += 1,
                _ => summary.info += 1,
            }
        }
        summary
    }
}
