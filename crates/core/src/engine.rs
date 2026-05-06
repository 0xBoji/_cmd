use crate::app::{Agent, AppState, Event};
use crate::history;
use crate::listener;
use crate::terminal::{self, TerminalCommand, TerminalCommandTx, TerminalEvent, TerminalSize};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

const RESIZE_DEBOUNCE: Duration = Duration::from_millis(140);

/// Actions that UI surfaces can send to the CoreEngine
#[derive(Debug, Clone)]
pub enum Action {
    /// Request the engine to spawn a new terminal process
    SpawnTerminal { cwd: PathBuf },
    /// Send a command string to a specific terminal session
    SubmitCommand { session_id: usize, command: String },
    /// Resize a terminal viewport; coalesced and flushed after drag/resize settles.
    ResizeTerminal { session_id: usize, size: TerminalSize },
    /// Persist a shell command to the shared command history store
    PersistHistory { command: String, cwd: String },
}

/// The centralized background engine that drives _CMD.
/// It manages the event loop, background listeners, and terminal PTYs.
pub struct CoreEngine {
    pub state: Arc<RwLock<AppState>>,
    pub action_tx: mpsc::UnboundedSender<Action>,
}

fn apply_terminal_event(app: &mut AppState, terminal_event: TerminalEvent) {
    match terminal_event {
        TerminalEvent::Line { session_id, line } => app.append_terminal_line(session_id, line),
        TerminalEvent::Status { session_id, status } => app.set_terminal_status(session_id, status),
        TerminalEvent::Cwd { session_id, cwd } => {
            if app.set_terminal_cwd(session_id, cwd.clone()) {
                app.record_directory_visit(cwd);
            }
        }
        TerminalEvent::Timing { session_id, seconds } => {
            app.finalize_terminal_context_line(session_id, seconds)
        }
        TerminalEvent::LastCommand { session_id, command } => {
            app.append_terminal_history(session_id, command.clone());
            app.set_terminal_last_command(session_id, command);
        }
        TerminalEvent::ExitCode {
            session_id,
            exit_code,
        } => {
            app.set_terminal_last_exit_code(session_id, exit_code);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingResize {
    size: TerminalSize,
    ready_at: Instant,
}

#[derive(Default)]
struct DeferredTerminalResizes {
    by_session: HashMap<usize, PendingResize>,
}

impl DeferredTerminalResizes {
    fn schedule(&mut self, session_id: usize, size: TerminalSize, now: Instant, delay: Duration) {
        self.by_session.insert(
            session_id,
            PendingResize {
                size,
                ready_at: now + delay,
            },
        );
    }

    fn take_ready<F>(&mut self, now: Instant, mut is_ready: F) -> Vec<(usize, TerminalSize)>
    where
        F: FnMut(usize) -> bool,
    {
        let ready_sessions = self
            .by_session
            .iter()
            .filter_map(|(session_id, pending)| {
                (pending.ready_at <= now && is_ready(*session_id)).then_some(*session_id)
            })
            .collect::<Vec<_>>();

        ready_sessions
            .into_iter()
            .filter_map(|session_id| {
                self.by_session
                    .remove(&session_id)
                    .map(|pending| (session_id, pending.size))
            })
            .collect()
    }
}

fn flush_ready_terminal_resizes(
    app: &mut AppState,
    pending: &mut DeferredTerminalResizes,
    shell_txs: &[TerminalCommandTx],
    now: Instant,
) {
    for (session_id, size) in pending.take_ready(now, |session_id| {
        app.terminal_sessions()
            .get(session_id)
            .is_some_and(|session| session.status == "ready")
    }) {
        if let Some(tx) = shell_txs.get(session_id) {
            let _ = tx.send(TerminalCommand::Resize(size));
            app.set_terminal_viewport_size(session_id, size);
        }
    }
}

impl CoreEngine {
    /// Start the engine tasks. Must be called from inside a tokio runtime context.
    pub fn spawn_background(state: Arc<RwLock<AppState>>) -> mpsc::UnboundedSender<Action> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();

        let (event_tx, mut event_rx) = mpsc::channel::<Event>(64);
        let (agent_tx, mut agent_rx) = mpsc::channel::<Agent>(64);
        let (terminal_event_tx, mut terminal_event_rx) = mpsc::unbounded_channel::<TerminalEvent>();

        // 1. Start the Demo Listener
        tokio::spawn(async move {
            let _ = listener::start_demo_listener(event_tx, agent_tx).await;
        });

        // 2. The Main Event God Loop
        let state_clone = state.clone();
        tokio::spawn(async move {
            let mut tick = time::interval(Duration::from_secs(1));
            let mut resize_tick = time::interval(Duration::from_millis(50));
            // Keep track of shell transmitters internal to the engine
            let mut shell_txs: Vec<TerminalCommandTx> = Vec::new();
            let mut deferred_resizes = DeferredTerminalResizes::default();
            let persisted_entries = history::load_entries().unwrap_or_default();
            let persisted_history = history::load_history().unwrap_or_default();
            let directory_history =
                history::directory_jump_history_from_entries(&persisted_entries);

            if !persisted_history.is_empty() || !directory_history.is_empty() {
                let mut app = state_clone.write();
                app.seed_terminal_history(persisted_history);
                app.seed_directory_history(directory_history);
            }

            loop {
                tokio::select! {
                    // --- UI ACTIONS ---
                    Some(action) = action_rx.recv() => {
                        match action {
                            Action::SpawnTerminal { cwd } => {
                                let (tx, rx) = terminal::local_shell_command_tx();
                                let session_id = shell_txs.len();
                                shell_txs.push(tx);

                                let term_event_tx = terminal_event_tx.clone();
                                tokio::spawn(async move {
                                    let _ = terminal::start_local_shell(session_id, cwd, term_event_tx, rx).await;
                                });
                            }
                            Action::SubmitCommand { session_id, command } => {
                                if let Some(tx) = shell_txs.get(session_id) {
                                    let _ = tx.send(TerminalCommand::Input(command));
                                }
                            }
                            Action::ResizeTerminal { session_id, size } => {
                                deferred_resizes.schedule(session_id, size, Instant::now(), RESIZE_DEBOUNCE);
                            }
                            Action::PersistHistory { command, cwd } => {
                                let _ = history::append_history_entry_with_cwd(&command, Some(&cwd));
                            }
                        }
                    }

                    // --- MESH EVENTS ---
                    Some(event) = event_rx.recv() => {
                        let mut app = state_clone.write();
                        app.add_event(event);
                    }
                    Some(agent) = agent_rx.recv() => {
                        let mut app = state_clone.write();
                        app.update_agent(agent);
                    }

                    // --- TERMINAL I/O ---
                    Some(terminal_event) = terminal_event_rx.recv() => {
                        let mut app = state_clone.write();
                        apply_terminal_event(&mut app, terminal_event);
                        flush_ready_terminal_resizes(
                            &mut app,
                            &mut deferred_resizes,
                            &shell_txs,
                            Instant::now(),
                        );
                    }

                    // --- PERIODIC TICK ---
                    _ = tick.tick() => {
                        let mut app = state_clone.write();
                        app.tick_activity();
                    }

                    _ = resize_tick.tick() => {
                        let mut app = state_clone.write();
                        flush_ready_terminal_resizes(
                            &mut app,
                            &mut deferred_resizes,
                            &shell_txs,
                            Instant::now(),
                        );
                    }
                }
            }
        });

        action_tx
    }
}

#[cfg(test)]
mod tests {
    use super::apply_terminal_event;
    use crate::app::AppState;
    use crate::terminal::{TerminalEvent, TerminalSize};
    use std::time::{Duration, Instant};

    #[test]
    fn apply_terminal_event_should_update_shell_integration_state() {
        let mut app = AppState::new();
        app.append_terminal_context_line(0, "/tmp/demo git:(main)".to_string());

        apply_terminal_event(
            &mut app,
            TerminalEvent::Status {
                session_id: 0,
                status: "running".to_string(),
            },
        );
        apply_terminal_event(
            &mut app,
            TerminalEvent::LastCommand {
                session_id: 0,
                command: "git status --short".to_string(),
            },
        );
        apply_terminal_event(
            &mut app,
            TerminalEvent::ExitCode {
                session_id: 0,
                exit_code: 17,
            },
        );
        apply_terminal_event(
            &mut app,
            TerminalEvent::Status {
                session_id: 0,
                status: "ready".to_string(),
            },
        );
        apply_terminal_event(
            &mut app,
            TerminalEvent::Timing {
                session_id: 0,
                seconds: 0.25,
            },
        );

        let session = &app.terminal_sessions()[0];
        assert_eq!(session.status, "ready");
        assert_eq!(session.last_command.as_deref(), Some("git status --short"));
        assert_eq!(session.last_exit_code, Some(17));
        assert_eq!(
            session.history.back().map(String::as_str),
            Some("git status --short")
        );
        assert_eq!(
            session.lines.back().map(String::as_str),
            Some("/tmp/demo git:(main) (0.2500s)")
        );
    }

    #[test]
    fn deferred_terminal_resizes_should_coalesce_until_terminal_is_ready() {
        let mut deferred = super::DeferredTerminalResizes::default();
        let started = Instant::now();

        deferred.schedule(
            0,
            TerminalSize { cols: 120, rows: 32 },
            started,
            Duration::from_millis(120),
        );
        deferred.schedule(
            0,
            TerminalSize { cols: 132, rows: 40 },
            started + Duration::from_millis(50),
            Duration::from_millis(120),
        );

        assert!(deferred
            .take_ready(started + Duration::from_millis(100), |_| true)
            .is_empty());
        assert!(deferred
            .take_ready(started + Duration::from_millis(250), |_| false)
            .is_empty());

        assert_eq!(
            deferred.take_ready(started + Duration::from_millis(250), |session_id| {
                session_id == 0
            }),
            vec![(0, TerminalSize { cols: 132, rows: 40 })]
        );
    }
}
