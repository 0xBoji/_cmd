use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};
use std::process::Stdio;
use tokio::sync::mpsc;
use crate::app::{Agent, Event, EventRecord, SnapshotRecord};

/// Connects to the real-time mesh via `camp watch --json`.
pub async fn start_camp_listener(tx: mpsc::Sender<Event>, agent_tx: mpsc::Sender<Agent>) -> anyhow::Result<()> {
    let mut child = Command::new("camp")
        .arg("watch")
        .arg("--json")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to start 'camp' process: {}. Is it in your PATH?", e))?;

    let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to capture camp stdout"))?;
    let mut reader = BufReader::new(stdout).lines();

    while let Some(line) = reader.next_line().await? {
        if line.is_empty() { continue; }

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
                    if let Some(agent) = record.current {
                        // 1. Check for dynamic RAI metadata
                        let mut found_rich_event = false;
                        
                        let component = agent.metadata.get("rai_component").cloned();
                        let level = agent.metadata.get("rai_level").cloned().unwrap_or_else(|| "info".into());

                        if let Some(comp) = component {
                            let _ = tx.send(Event {
                                timestamp: chrono::Local::now(),
                                agent_id: agent.id.clone(),
                                kind: record.kind.to_uppercase(),
                                component: comp,
                                level,
                                payload: agent.metadata.get("log").cloned().unwrap_or_else(|| "No log message".into()),
                            }).await;
                            found_rich_event = true;
                        } else {
                            // Backward compatibility sniffers
                            if let Some(action) = agent.metadata.get("action") {
                                let _ = tx.send(Event {
                                    timestamp: chrono::Local::now(),
                                    agent_id: agent.id.clone(),
                                    kind: "ACTION".to_string(),
                                    component: "ACTION".to_string(),
                                    level: "info".to_string(),
                                    payload: action.clone(),
                                }).await;
                                found_rich_event = true;
                            }

                            if let Some(log) = agent.metadata.get("wasp_log") {
                                let _ = tx.send(Event {
                                    timestamp: chrono::Local::now(),
                                    agent_id: agent.id.clone(),
                                    kind: "WASP".to_string(),
                                    component: "WASP".to_string(),
                                    level: "info".to_string(),
                                    payload: log.clone(),
                                }).await;
                                found_rich_event = true;
                            }
                        }

                        // 2. Fallback to generic MESH event if nothing else
                        if !found_rich_event {
                            let event = Event {
                                timestamp: chrono::Local::now(),
                                agent_id: agent.id.clone(),
                                kind: record.kind.to_uppercase(),
                                component: "MESH".to_string(),
                                level: "info".to_string(),
                                payload: format!("Agent {} via {}", record.kind, record.origin),
                            };
                            let _ = tx.send(event).await;
                        }

                        let _ = agent_tx.send(agent).await;
                    }
                }
                "left" => {
                    if let Some(agent) = record.current {
                        let mut offline_agent = agent.clone();
                        offline_agent.status = "Offline".to_string();
                        
                        let event = Event {
                            timestamp: chrono::Local::now(),
                            agent_id: offline_agent.id.clone(),
                            kind: "LEFT".to_string(),
                            component: "MESH".to_string(),
                            level: "info".to_string(),
                            payload: format!("Agent left mesh (reason: {})", record.reason.unwrap_or_else(|| "unknown".into())),
                        };
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
