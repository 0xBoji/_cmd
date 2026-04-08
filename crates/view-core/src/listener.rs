use crate::app::{Agent, Event, EventRecord, SnapshotRecord};
use chrono::Local;
use std::collections::{BTreeMap, VecDeque};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

#[derive(Debug, Clone, PartialEq, Eq)]
struct CampWatchCommandConfig {
    args: &'static [&'static str],
    silence_stderr: bool,
}

fn build_event_from_agent_record(record: &EventRecord, agent: &Agent) -> Event {
    let component = agent
        .metadata
        .get("rai_component")
        .cloned()
        .unwrap_or_else(|| "mesh".to_string());
    let level = agent
        .metadata
        .get("rai_level")
        .cloned()
        .unwrap_or_else(|| "info".to_string());
    let payload = agent
        .metadata
        .get("log")
        .cloned()
        .unwrap_or_else(|| format!("Agent {} via {}", record.kind, record.origin));

    Event {
        timestamp: chrono::Local::now(),
        agent_id: agent.id.clone(),
        kind: record.kind.to_uppercase(),
        component,
        level,
        payload,
    }
}

pub fn demo_mode_enabled() -> bool {
    std::env::var("VIEW_DEMO")
        .map(|value| demo_mode_from_value(&value))
        .unwrap_or(false)
}

fn demo_mode_from_value(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on" | "demo"
    )
}

fn build_left_event(record: &EventRecord, agent_id: String) -> Event {
    Event {
        timestamp: chrono::Local::now(),
        agent_id,
        kind: "LEFT".to_string(),
        component: "mesh".to_string(),
        level: "info".to_string(),
        payload: format!(
            "Agent left mesh (reason: {})",
            record
                .reason
                .clone()
                .unwrap_or_else(|| "unknown".to_string())
        ),
    }
}

fn camp_watch_command_config() -> CampWatchCommandConfig {
    CampWatchCommandConfig {
        args: &["watch"],
        silence_stderr: true,
    }
}

fn build_camp_watch_command() -> Command {
    let config = camp_watch_command_config();
    let mut command = Command::new("camp");
    command.args(config.args).stdout(Stdio::piped());
    if config.silence_stderr {
        command.stderr(Stdio::null());
    }
    command
}

fn demo_agents(step: usize) -> Vec<Agent> {
    let statuses = [
        (
            "agentic-coding",
            "busy",
            "orchestrator",
            "workspace",
            "feature/session-grid",
        ),
        (
            "docs-site",
            "idle",
            "worker",
            "workspace",
            "feat/reference-pass",
        ),
        (
            "api-server",
            "busy",
            "planner",
            "workspace",
            "plan/runtime-cleanup",
        ),
        ("mobile-app", "offline", "auditor", "workspace", "main"),
        (
            "infra-terraform",
            "busy",
            "reviewer",
            "workspace",
            "fix/state-drift",
        ),
        (
            "shared-lib",
            "idle",
            "builder",
            "workspace",
            "feat/desktop-preview",
        ),
    ];

    statuses
        .iter()
        .enumerate()
        .map(|(index, (id, status, role, project, branch))| {
            let mut metadata = BTreeMap::new();
            metadata.insert(
                "tokens".to_string(),
                format!("{}", 24_000 + (index as u64 * 13_500) + (step as u64 * 750)),
            );
            metadata.insert("cwd".to_string(), format!("/Users/demo/projects/{id}"));
            metadata.insert(
                "model".to_string(),
                match index {
                    0 => "gpt-5.4".to_string(),
                    1 => "gpt-5.4-mini".to_string(),
                    2 => "gpt-5.3-codex-spark".to_string(),
                    _ => "gpt-5.4-mini".to_string(),
                },
            );
            metadata.insert(
                "last_file".to_string(),
                format!("/Users/demo/projects/{id}/{}", demo_file_name(index, step)),
            );
            metadata.insert(
                "last_tool".to_string(),
                match index {
                    0 => "Edit",
                    1 => "Search",
                    2 => "Plan",
                    3 => "Idle",
                    4 => "Review",
                    _ => "Build",
                }
                .to_string(),
            );
            metadata.insert(
                "messages".to_string(),
                format!("{}", 12 + index * 4 + (step % 3)),
            );
            metadata.insert(
                "cost".to_string(),
                format!("${:.2}", 0.18 + index as f32 * 0.09 + step as f32 * 0.01),
            );

            let activity = (0..50)
                .map(|offset| {
                    let phase = (step + offset + index * 3) % 11;
                    if *status == "offline" {
                        0
                    } else if phase > 7 {
                        4 + index as u64
                    } else if phase > 3 {
                        2 + index as u64
                    } else {
                        (index % 2) as u64
                    }
                })
                .collect::<VecDeque<_>>();

            Agent {
                id: (*id).to_string(),
                instance_name: format!("{id}.rai"),
                role: (*role).to_string(),
                project: (*project).to_string(),
                branch: (*branch).to_string(),
                status: (*status).to_string(),
                capabilities: vec![
                    "observe".to_string(),
                    "stream-json".to_string(),
                    format!("tool-{}", index + 1),
                ],
                port: 4100 + index as u16,
                addresses: vec![format!("127.0.0.1:{}", 4100 + index as u16)],
                metadata,
                last_seen: Local::now(),
                tokens: 24_000 + (index as u64 * 13_500) + (step as u64 * 750),
                activity,
            }
        })
        .collect()
}

fn demo_events(step: usize) -> Vec<Event> {
    let scripts = [
        (
            "agentic-coding",
            "shell",
            [
                ("info", "$ cargo test --workspace"),
                ("success", "Tests completed with 0 failures"),
                ("error", "Retry budget exhausted for release sync"),
            ],
        ),
        (
            "docs-site",
            "shell",
            [
                ("success", "$ pnpm dev"),
                ("info", "Preview server listening on :3000"),
                ("success", "Reference page updated cleanly"),
            ],
        ),
        (
            "api-server",
            "shell",
            [
                ("warn", "$ cargo check"),
                ("warn", "Borrow checker still unhappy in auth flow"),
                ("success", "Handler plan merged into runtime lane"),
            ],
        ),
        (
            "infra-terraform",
            "shell",
            [
                ("warn", "$ terraform plan"),
                ("info", "State drift detected in staging"),
                ("success", "Review checklist completed"),
            ],
        ),
        (
            "shared-lib",
            "shell",
            [
                ("info", "$ cargo doc --open"),
                ("success", "Desktop preview rendered successfully"),
                ("info", "Public API draft ready for review"),
            ],
        ),
    ];

    scripts
        .iter()
        .enumerate()
        .map(|(index, (agent_id, component, script))| {
            let (level, payload) = script[(step + index) % script.len()];
            Event {
                timestamp: Local::now(),
                agent_id: (*agent_id).to_string(),
                kind: "UPDATED".to_string(),
                component: (*component).to_string(),
                level: level.to_string(),
                payload: payload.to_string(),
            }
        })
        .collect()
}

fn demo_file_name(index: usize, step: usize) -> &'static str {
    let files = [
        [
            "src/main.rs",
            "src/session.rs",
            "Cargo.toml",
            "README.md",
            "src/ui.rs",
        ],
        [
            "app/routes/docs.tsx",
            "content/api.md",
            "package.json",
            "README.md",
            "app/layout.tsx",
        ],
        [
            "src/auth.rs",
            "src/server.rs",
            "src/routes.rs",
            "Cargo.toml",
            "src/lib.rs",
        ],
        [
            "infra/main.tf",
            "infra/variables.tf",
            "README.md",
            "envs/staging.tfvars",
            "modules/vpc/main.tf",
        ],
        [
            "src/lib.rs",
            "src/terminal.rs",
            "README.md",
            "src/theme.rs",
            "Cargo.toml",
        ],
    ];
    files[index % files.len()][step % files[0].len()]
}

pub async fn start_demo_listener(
    tx: mpsc::Sender<Event>,
    agent_tx: mpsc::Sender<Agent>,
) -> anyhow::Result<()> {
    let mut tick = time::interval(Duration::from_millis(900));
    for warmup in 0..4 {
        emit_demo_step(&tx, &agent_tx, warmup).await?;
    }
    let mut step = 4usize;

    loop {
        tick.tick().await;
        emit_demo_step(&tx, &agent_tx, step).await?;

        step = step.wrapping_add(1);
    }
}

async fn emit_demo_step(
    tx: &mpsc::Sender<Event>,
    agent_tx: &mpsc::Sender<Agent>,
    step: usize,
) -> anyhow::Result<()> {
    for agent in demo_agents(step) {
        if agent_tx.send(agent).await.is_err() {
            return Ok(());
        }
    }

    for event in demo_events(step) {
        if tx.send(event).await.is_err() {
            return Ok(());
        }
    }

    Ok(())
}

/// Connects to the real-time mesh via `camp watch --json`.
pub async fn start_camp_listener(
    tx: mpsc::Sender<Event>,
    agent_tx: mpsc::Sender<Agent>,
) -> anyhow::Result<()> {
    let mut child = build_camp_watch_command().spawn().map_err(|e| {
        anyhow::anyhow!("Failed to start 'camp' process: {}. Is it in your PATH?", e)
    })?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture camp stdout"))?;
    let mut reader = BufReader::new(stdout).lines();

    while let Some(line) = reader.next_line().await? {
        if line.is_empty() {
            continue;
        }

        // Try parsing as a Snapshot first (initial state)
        if let Ok(snapshot) = serde_json::from_str::<SnapshotRecord>(&line) {
            for agent in snapshot.agents {
                let _ = agent_tx.send(agent).await;
            }
            continue;
        }

        // Otherwise parse as an EventRecord
        if let Ok(record) = serde_json::from_str::<EventRecord>(&line) {
            match record.kind.as_str() {
                "joined" | "updated" => {
                    if let Some(agent) = record.current.as_ref() {
                        let event = build_event_from_agent_record(&record, agent);
                        let _ = tx.send(event).await;
                        let _ = agent_tx.send(agent.clone()).await;
                    }
                }
                "left" => {
                    if let Some(agent) = record.current.as_ref() {
                        let mut offline_agent = agent.clone();
                        offline_agent.status = "Offline".to_string();

                        let event = build_left_event(&record, offline_agent.id.clone());
                        let _ = tx.send(event).await;
                        let _ = agent_tx.send(offline_agent).await;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        build_event_from_agent_record, camp_watch_command_config, demo_agents, demo_events,
        demo_mode_from_value,
    };
    use crate::app::{Agent, EventRecord};
    use chrono::Local;
    use std::collections::{BTreeMap, VecDeque};

    fn test_agent(metadata: &[(&str, &str)]) -> Agent {
        Agent {
            id: "agent-1".to_string(),
            instance_name: "agent-1".to_string(),
            role: "executor".to_string(),
            project: "view".to_string(),
            branch: "main".to_string(),
            status: "busy".to_string(),
            capabilities: Vec::new(),
            port: 0,
            addresses: Vec::new(),
            metadata: metadata
                .iter()
                .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
                .collect::<BTreeMap<_, _>>(),
            last_seen: Local::now(),
            tokens: 0,
            activity: VecDeque::from(vec![0; 50]),
        }
    }

    fn test_record() -> EventRecord {
        EventRecord {
            kind: "updated".to_string(),
            origin: "camp".to_string(),
            reason: None,
            previous: None,
            current: None,
        }
    }

    #[test]
    fn build_event_from_agent_record_should_use_rai_component_and_level_metadata() {
        let record = test_record();
        let agent = test_agent(&[
            ("rai_component", "tick"),
            ("rai_level", "warn"),
            ("log", "Queue is backing up"),
        ]);

        let event = build_event_from_agent_record(&record, &agent);

        assert_eq!(event.component, "tick");
        assert_eq!(event.level, "warn");
        assert_eq!(event.payload, "Queue is backing up");
    }

    #[test]
    fn build_event_from_agent_record_should_default_missing_level_to_info() {
        let record = test_record();
        let agent = test_agent(&[("rai_component", "garc"), ("log", "Dispatch ready")]);

        let event = build_event_from_agent_record(&record, &agent);

        assert_eq!(event.component, "garc");
        assert_eq!(event.level, "info");
    }

    #[test]
    fn camp_watch_command_config_should_avoid_json_flag_and_silence_stderr() {
        let config = camp_watch_command_config();

        assert_eq!(config.args, &["watch"]);
        assert!(config.silence_stderr);
    }

    #[test]
    fn demo_mode_from_value_should_accept_common_truthy_inputs() {
        assert!(demo_mode_from_value("1"));
        assert!(demo_mode_from_value("true"));
        assert!(demo_mode_from_value("YES"));
        assert!(demo_mode_from_value("demo"));
        assert!(!demo_mode_from_value("0"));
        assert!(!demo_mode_from_value("off"));
    }

    #[test]
    fn demo_dataset_should_cover_multiple_agents_and_levels() {
        let agents = demo_agents(2);
        let events = demo_events(3);

        assert_eq!(agents.len(), 6);
        assert!(agents
            .iter()
            .any(|agent| agent.status.eq_ignore_ascii_case("busy")));
        assert!(agents
            .iter()
            .any(|agent| agent.status.eq_ignore_ascii_case("offline")));
        assert!(agents.iter().all(|agent| !agent.activity.is_empty()));
        assert_eq!(events.len(), 5);
        assert!(events.iter().all(|event| event.component == "shell"));
        assert!(events.iter().any(|event| event.level == "success"));
    }
}
