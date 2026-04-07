mod app;
mod ui;
mod listener;

use std::{
    io::{self, Stdout},
    sync::{Arc},
    time::{Duration},
};
use tokio::sync::{Mutex, mpsc};

use anyhow::Result;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::{AppState, Event, Agent};

/// RAII Guard to ensure terminal is restored on exit, even during panics.
struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize State
    let app_state = Arc::new(Mutex::new(AppState::new()));
    
    // 2. Initialize Channels
    let (event_tx, mut event_rx) = mpsc::channel::<Event>(32);
    let (agent_tx, mut agent_rx) = mpsc::channel::<Agent>(32);

    // 3. Start Listener (Background Task)
    let listener_handle = tokio::spawn(listener::start_simulated_listener(event_tx, agent_tx));

    let mut guard = TerminalGuard::new()?;
    let mut frame_count: u64 = 0;
    
    // 5. Main Render/Event Loop
    let mut interval = tokio::time::interval(Duration::from_millis(16)); // Target ~60 FPS
    
    loop {
        interval.tick().await;
        frame_count += 1;

        // A. Filter and handle incoming events
        while let Ok(event) = event_rx.try_recv() {
            let mut state = app_state.lock().await;
            state.add_event(event);
        }

        // B. Handle incoming agent updates
        while let Ok(agent) = agent_rx.try_recv() {
            let mut state = app_state.lock().await;
            state.update_agent(agent);
        }

        // C. Handle Input (Non-blocking)
        if event::poll(Duration::from_millis(0))? {
            if let CEvent::Key(key) = event::read()? {
                let mut state = app_state.lock().await;
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) => state.should_quit = true,
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => state.should_quit = true,
                    (KeyCode::Down, _) | (KeyCode::Char('j'), _) => state.select_next(),
                    (KeyCode::Up, _) | (KeyCode::Char('k'), _) => state.select_previous(),
                    (KeyCode::PageDown, _) => state.select_next_page(),
                    (KeyCode::PageUp, _) => state.select_previous_page(),
                    (KeyCode::Home, _) => state.select_first(),
                    (KeyCode::End, _) => state.select_last(),
                    _ => {}
                }
            }
        }

        // D. Tick Activity and Render Frame
        {
            let mut state = app_state.lock().await;
            if state.should_quit {
                break;
            }
            // Only shift the sparkline every 10 frames (~160ms) for better visibility
            if frame_count % 10 == 0 {
                state.tick_activity();
            }
            guard.terminal.draw(|f| ui::render(f, &state))?;
        }
    }

    // Explicitly abort background tasks on exit
    listener_handle.abort();

    Ok(())
}
